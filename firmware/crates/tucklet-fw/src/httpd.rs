//! Data plane HTTP server. Serves the metadata-only manifest, thumbnails, full
//! files (with range support), uploads (with origin metadata), deletes, and the
//! restore-origin lookup. Every request must carry the session bearer token in
//! `X-Tucklet-Token`; the token is validated against the live session.
//!
//! Endpoints (see docs/protocol/PROTOCOL.md §2):
//!   GET    /v1/manifest
//!   GET    /v1/thumb/{id}
//!   GET    /v1/file/{id}        (supports Range)
//!   POST   /v1/file             (body = bytes; X-Tucklet-Origin = base64 MediaItem)
//!   DELETE /v1/file/{id}
//!   POST   /v1/restore/{id}

use anyhow::Result;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tucklet_core::session::Session;

use crate::storage;

/// Shared handle to the currently-valid session. `None` when no transfer
/// session is active (server rejects everything).
pub type SharedSession = Arc<Mutex<Option<Session>>>;

/// A monotonic clock (seconds) used to validate session TTL. On-device this is
/// wired to the RTC; injected so handlers stay testable in principle.
pub type NowFn = Arc<dyn Fn() -> i64 + Send + Sync>;

const TOKEN_HEADER: &str = "X-Tucklet-Token";
const ORIGIN_HEADER: &str = "X-Tucklet-Origin";

/// Start the HTTP server and register all routes. The returned server must be
/// kept alive for as long as the data path is up.
pub fn start(session: SharedSession, now: NowFn) -> Result<EspHttpServer<'static>> {
    let mut server = EspHttpServer::new(&HttpConfig {
        stack_size: 10_240,
        ..Default::default()
    })?;

    // --- GET /v1/manifest --------------------------------------------------
    {
        let session = session.clone();
        let now = now.clone();
        server.fn_handler("/v1/manifest", Method::Get, move |req| {
            if !authorized(&req, &session, &now) {
                return reject(req);
            }
            let manifest = storage::manifest().unwrap_or_else(|_| tucklet_proto::Manifest {
                items: Vec::new(),
                free_bytes: 0,
                total_bytes: 0,
            });
            let body = serde_json::to_vec(&manifest)?;
            let mut resp = req.into_response(200, Some("OK"), &json_headers())?;
            resp.write_all(&body)?;
            Ok(())
        })?;
    }

    // --- GET /v1/thumb/{id} ------------------------------------------------
    {
        let session = session.clone();
        let now = now.clone();
        server.fn_handler("/v1/thumb/*", Method::Get, move |req| {
            if !authorized(&req, &session, &now) {
                return reject(req);
            }
            let id = last_path_segment(req.uri());
            match storage::read_thumb(&id) {
                Ok(bytes) => {
                    let mut resp = req.into_response(
                        200,
                        Some("OK"),
                        &[("Content-Type", "image/jpeg")],
                    )?;
                    resp.write_all(&bytes)?;
                }
                Err(_) => {
                    req.into_response(404, Some("Not Found"), &[])?;
                }
            }
            Ok(())
        })?;
    }

    // --- GET /v1/file/{id} (with Range) ------------------------------------
    {
        let session = session.clone();
        let now = now.clone();
        server.fn_handler("/v1/file/*", Method::Get, move |req| {
            if !authorized(&req, &session, &now) {
                return reject(req);
            }
            let id = last_path_segment(req.uri());
            let range = parse_range(header(&req, "Range").as_deref());
            let item = match storage::item(&id) {
                Ok(it) => it,
                Err(_) => {
                    req.into_response(404, Some("Not Found"), &[])?;
                    return Ok(());
                }
            };
            let headers = [
                ("Content-Type", item.mime.as_str()),
                ("Accept-Ranges", "bytes"),
            ];
            let status = if range.is_some() { 206 } else { 200 };
            let mut resp = req.into_response(status, None, &headers)?;
            storage::read_file_into(&id, range, &mut resp)?;
            Ok(())
        })?;
    }

    // --- POST /v1/file -----------------------------------------------------
    {
        let session = session.clone();
        let now = now.clone();
        server.fn_handler("/v1/file", Method::Post, move |mut req| {
            if !authorized(&req, &session, &now) {
                return reject(req);
            }
            // Origin metadata + MediaItem fields travel base64-encoded in a
            // header so the body is pure bytes (streamable, large-file safe).
            let origin_b64 = header(&req, ORIGIN_HEADER).unwrap_or_default();
            let meta_json = base64_decode(&origin_b64).unwrap_or_default();
            let meta: tucklet_proto::MediaItem = match serde_json::from_slice(&meta_json) {
                Ok(m) => m,
                Err(_) => {
                    req.into_response(400, Some("Bad origin metadata"), &[])?;
                    return Ok(());
                }
            };
            // Stream the request body straight to the card.
            let mut reader = BodyReader { req: &mut req };
            let stored = storage::write_file(meta, &mut reader)?;
            let body = serde_json::to_vec(&stored)?;
            let mut resp = req.into_response(201, Some("Created"), &json_headers())?;
            resp.write_all(&body)?;
            Ok(())
        })?;
    }

    // --- DELETE /v1/file/{id} ----------------------------------------------
    {
        let session = session.clone();
        let now = now.clone();
        server.fn_handler("/v1/file/*", Method::Delete, move |req| {
            if !authorized(&req, &session, &now) {
                return reject(req);
            }
            let id = last_path_segment(req.uri());
            let _ = storage::delete(&id);
            req.into_response(204, Some("No Content"), &[])?;
            Ok(())
        })?;
    }

    // --- POST /v1/restore/{id} --------------------------------------------
    {
        let session = session.clone();
        let now = now.clone();
        server.fn_handler("/v1/restore/*", Method::Post, move |req| {
            if !authorized(&req, &session, &now) {
                return reject(req);
            }
            let id = last_path_segment(req.uri());
            match storage::restore_origin(&id) {
                Ok(origin) => {
                    let body = serde_json::to_vec(&origin)?;
                    let mut resp = req.into_response(200, Some("OK"), &json_headers())?;
                    resp.write_all(&body)?;
                }
                Err(_) => {
                    req.into_response(404, Some("Not Found"), &[])?;
                }
            }
            Ok(())
        })?;
    }

    Ok(server)
}

// --- helpers --------------------------------------------------------------

fn json_headers() -> [(&'static str, &'static str); 1] {
    [("Content-Type", "application/json")]
}

/// Validate the bearer token against the live session and its TTL.
fn authorized<C>(
    req: &esp_idf_svc::http::server::Request<C>,
    session: &SharedSession,
    now: &NowFn,
) -> bool
where
    C: esp_idf_svc::http::server::Connection,
{
    let token = match header(req, TOKEN_HEADER) {
        Some(t) => t,
        None => return false,
    };
    let guard = session.lock().unwrap();
    match guard.as_ref() {
        Some(s) => s.authorize(&token, now()),
        None => false,
    }
}

fn reject<C>(req: esp_idf_svc::http::server::Request<C>) -> Result<()>
where
    C: esp_idf_svc::http::server::Connection,
{
    req.into_response(401, Some("Unauthorized"), &[])?;
    Ok(())
}

fn header<C>(req: &esp_idf_svc::http::server::Request<C>, name: &str) -> Option<String>
where
    C: esp_idf_svc::http::server::Connection,
{
    req.header(name).map(|s| s.to_string())
}

/// The last path segment, percent-decoding is not needed for our opaque ids.
fn last_path_segment(uri: &str) -> String {
    let path = uri.split('?').next().unwrap_or(uri);
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string()
}

/// Parse a single-range `Range: bytes=start-end` header into (start, end).
fn parse_range(h: Option<&str>) -> Option<(u64, u64)> {
    let h = h?;
    let spec = h.strip_prefix("bytes=")?;
    let mut parts = spec.splitn(2, '-');
    let start: u64 = parts.next()?.trim().parse().ok()?;
    let end = parts
        .next()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(u64::MAX);
    Some((start, end))
}

/// Minimal base64 decode (standard alphabet, no external dep) for the origin
/// header. Returns empty on malformed input.
fn base64_decode(s: &str) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes: Vec<u8> = s.bytes().filter(|&b| b != b'=' && !b.is_ascii_whitespace()).collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks(4) {
        let mut buf = [0u8; 4];
        let mut n = 0;
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = val(b)?;
            n += 1;
        }
        if n >= 2 {
            out.push((buf[0] << 2) | (buf[1] >> 4));
        }
        if n >= 3 {
            out.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if n >= 4 {
            out.push((buf[2] << 6) | buf[3]);
        }
    }
    Some(out)
}

/// Adapter so the request body implements std::io::Read for storage::write_file.
struct BodyReader<'a, 'b, C: esp_idf_svc::http::server::Connection> {
    req: &'a mut esp_idf_svc::http::server::Request<&'b mut C>,
}

impl<'a, 'b, C: esp_idf_svc::http::server::Connection> std::io::Read for BodyReader<'a, 'b, C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Read::read(self.req, buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_parsing() {
        assert_eq!(parse_range(Some("bytes=0-99")), Some((0, 99)));
        assert_eq!(parse_range(Some("bytes=500-")), Some((500, u64::MAX)));
        assert_eq!(parse_range(None), None);
        assert_eq!(parse_range(Some("garbage")), None);
    }

    #[test]
    fn last_segment() {
        assert_eq!(last_path_segment("/v1/file/abc123"), "abc123");
        assert_eq!(last_path_segment("/v1/thumb/x?y=1"), "x");
    }

    #[test]
    fn base64_roundtrip_known() {
        // "Tucklet" -> "VHVja2xldA=="
        assert_eq!(base64_decode("VHVja2xldA=="), Some(b"Tucklet".to_vec()));
    }
}
