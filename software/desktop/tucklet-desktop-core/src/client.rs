// client.rs
// The /v1 data API client (mirrors httpd.rs / PROTOCOL.md §2), built on the
// dependency-free http module and the shared tucklet-proto types. Plain HTTP
// over the charm's local link, authenticated with the per-session token.
//
// License: PolyForm Noncommercial 1.0.0

use crate::b64;
use crate::http;
use std::fs;
use std::path::Path;
use tucklet_proto::{Manifest, MediaItem, OriginMetadata};

#[derive(Debug)]
pub enum ClientError {
    Http(http::HttpError),
    Status(u16),
    Json(String),
    Io(std::io::Error),
}
impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Http(e) => write!(f, "http: {e}"),
            ClientError::Status(s) => write!(f, "unexpected status {s}"),
            ClientError::Json(e) => write!(f, "json: {e}"),
            ClientError::Io(e) => write!(f, "io: {e}"),
        }
    }
}
impl std::error::Error for ClientError {}
impl From<http::HttpError> for ClientError {
    fn from(e: http::HttpError) -> Self {
        ClientError::Http(e)
    }
}
impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::Io(e)
    }
}

/// A connected data session: the charm's IP + the bearer token from the grant.
pub struct DataClient {
    host: String,
    token: String,
}

impl DataClient {
    pub fn new(host: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            token: token.into(),
        }
    }

    fn auth(&self) -> Vec<(&'static str, String)> {
        vec![("X-Tucklet-Token", self.token.clone())]
    }

    pub fn manifest(&self) -> Result<Manifest, ClientError> {
        let r = http::request(&self.host, "GET", "/v1/manifest", &self.auth(), None)?;
        if !r.is_success() {
            return Err(ClientError::Status(r.status));
        }
        serde_json::from_slice(&r.body).map_err(|e| ClientError::Json(e.to_string()))
    }

    pub fn thumbnail(&self, id: &str) -> Result<Option<Vec<u8>>, ClientError> {
        let r = http::request(
            &self.host,
            "GET",
            &format!("/v1/thumb/{id}"),
            &self.auth(),
            None,
        )?;
        if r.status == 404 {
            return Ok(None);
        }
        if !r.is_success() {
            return Err(ClientError::Status(r.status));
        }
        Ok(Some(r.body))
    }

    /// Download a file to `dest`. Supports the Range header for resume, though
    /// here we fetch whole files (the firmware honors Range either way).
    pub fn download(&self, id: &str, dest: &Path) -> Result<u64, ClientError> {
        let r = http::request(
            &self.host,
            "GET",
            &format!("/v1/file/{id}"),
            &self.auth(),
            None,
        )?;
        if !r.is_success() {
            return Err(ClientError::Status(r.status));
        }
        fs::write(dest, &r.body)?;
        Ok(r.body.len() as u64)
    }

    /// Upload a local file as a new item. The full MediaItem travels base64 in
    /// the X-Tucklet-Origin header (the firmware stores it + forces state).
    pub fn upload(&self, file: &Path, item: &MediaItem) -> Result<(), ClientError> {
        let body = fs::read(file)?;
        let item_json =
            serde_json::to_vec(item).map_err(|e| ClientError::Json(e.to_string()))?;
        let mut headers = self.auth();
        headers.push(("Content-Type", "application/octet-stream".to_string()));
        headers.push(("X-Tucklet-Origin", b64::encode(&item_json)));
        let r = http::request(&self.host, "POST", "/v1/file", &headers, Some(&body))?;
        if !r.is_success() {
            return Err(ClientError::Status(r.status));
        }
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), ClientError> {
        let r = http::request(
            &self.host,
            "DELETE",
            &format!("/v1/file/{id}"),
            &self.auth(),
            None,
        )?;
        if !r.is_success() {
            return Err(ClientError::Status(r.status));
        }
        Ok(())
    }

    pub fn restore_origin(&self, id: &str) -> Result<OriginMetadata, ClientError> {
        let r = http::request(
            &self.host,
            "POST",
            &format!("/v1/restore/{id}"),
            &self.auth(),
            Some(&[]),
        )?;
        if !r.is_success() {
            return Err(ClientError::Status(r.status));
        }
        serde_json::from_slice(&r.body).map_err(|e| ClientError::Json(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tucklet_proto::{ItemState, Manifest, MediaItem, OriginMetadata, Platform};

    #[test]
    fn parses_a_manifest_fixture_into_proto_types() {
        // Exactly the wire shape the firmware emits (flattened state).
        let json = r#"{
            "items": [
              {"id":"a","name":"IMG_1.heic","size_bytes":1500000,"mime":"image/heic",
               "created_at":1700000000,
               "origin":{"platform":"desktop","app":"Camera","collection":"DCIM/Camera","device_name":"Mac"},
               "state":"on_tucklet"},
              {"id":"b","name":"V.mov","size_bytes":50000000,"mime":"video/quicktime",
               "created_at":1700000100,
               "origin":{"platform":"ios","app":"Camera","collection":"DCIM/Camera","device_name":"iPhone"},
               "state":"temporary","expires_at":1700600000}
            ],
            "free_bytes": 12000000000,
            "total_bytes": 64000000000
        }"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.items.len(), 2);
        assert!(matches!(m.items[0].state, ItemState::OnTucklet));
        match m.items[1].state {
            ItemState::Temporary { expires_at } => assert_eq!(expires_at, Some(1700600000)),
            _ => panic!("expected temporary"),
        }
        assert_eq!(m.total_bytes, 64_000_000_000);
    }

    #[test]
    fn upload_item_round_trips_through_origin_header_base64() {
        let item = MediaItem {
            id: "x".into(),
            name: "p.heic".into(),
            size_bytes: 10,
            mime: "image/heic".into(),
            created_at: 1,
            origin: OriginMetadata {
                platform: Platform::Desktop,
                app: "Camera".into(),
                collection: "DCIM/Camera".into(),
                album: None,
                device_name: "Mac".into(),
            },
            state: ItemState::OnPhone,
            checksum: None,
        };
        let json = serde_json::to_vec(&item).unwrap();
        let header = b64::encode(&json);
        let decoded = b64::decode(&header).unwrap();
        let back: MediaItem = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(back.id, "x");
        assert!(matches!(back.origin.platform, Platform::Desktop));
    }
}
