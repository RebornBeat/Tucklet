// integration_device.rs
// Cross-component integration test. A host-side MOCK Tucklet built from the SAME
// shared crates the firmware uses (tucklet-core allow-list, tucklet-crypto verify,
// tucklet-proto wire types) is driven by the REAL desktop client through the full
// chain: enroll -> nonce -> sign -> verify -> token -> /v1 manifest/push/pull/delete.
//
// The only stand-in is the transport for the auth message (HTTP here, BLE on
// device): the bytes signed and verified are identical, so this exercises the
// genuine crypto handshake + protocol end to end.
//
// License: PolyForm Noncommercial 1.0.0

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use tucklet_desktop_core::allowlist::AllowList;
use tucklet_desktop_core::auth::Identity;
use tucklet_desktop_core::proto::{Manifest, MediaItem};
use tucklet_desktop_core::{b64, http, tucklet_crypto, DataClient};

/// Shared state of the mock device.
struct Device {
    allow: AllowList,
    nonce: [u8; 32],
    token: Mutex<Option<String>>,
    store: Mutex<Vec<(MediaItem, Vec<u8>)>>,
    free_bytes: u64,
    total_bytes: u64,
}

fn read_request(stream: &mut TcpStream) -> Option<(String, String, HashMap<String, String>, Vec<u8>)> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];
    // Read until we have the header terminator.
    let head_end = loop {
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
        let n = stream.read(&mut tmp).ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&tmp[..n]);
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let mut lines = head.split("\r\n");
    let request_line = lines.next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    let want = headers
        .get("content-length")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = buf[head_end..].to_vec();
    while body.len() < want {
        let n = stream.read(&mut tmp).ok()?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
    }
    Some((method, path, headers, body))
}

fn respond(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
    let reason = match status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "OK",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

fn handle(dev: &Arc<Device>, stream: &mut TcpStream) {
    let Some((method, path, headers, body)) = read_request(stream) else { return };

    // --- auth plane (stands in for BLE) ---
    if method == "GET" && path == "/auth/nonce" {
        let json = format!("{{\"nonce\":\"{}\"}}", hex_encode(&dev.nonce));
        respond(stream, 200, "application/json", json.as_bytes());
        return;
    }
    if method == "POST" && path == "/auth/session" {
        let pubkey = headers.get("x-tucklet-pubkey").cloned().unwrap_or_default();
        let sig = headers.get("x-tucklet-sig").cloned().unwrap_or_default();
        let ok = dev.allow.contains(&pubkey)
            && tucklet_crypto::verify_challenge(&pubkey, &dev.nonce, &sig).unwrap_or(false);
        if ok {
            let token = "session-token-abc".to_string();
            *dev.token.lock().unwrap() = Some(token.clone());
            respond(
                stream,
                200,
                "application/json",
                format!("{{\"token\":\"{token}\"}}").as_bytes(),
            );
        } else {
            respond(stream, 401, "text/plain", b"bad challenge");
        }
        return;
    }

    // --- data plane (token-gated) ---
    let token_ok = match (&*dev.token.lock().unwrap(), headers.get("x-tucklet-token")) {
        (Some(t), Some(h)) => t == h,
        _ => false,
    };
    if !token_ok {
        respond(stream, 401, "text/plain", b"no token");
        return;
    }

    if method == "GET" && path == "/v1/manifest" {
        let items: Vec<MediaItem> = dev.store.lock().unwrap().iter().map(|(m, _)| m.clone()).collect();
        let manifest = Manifest {
            items,
            free_bytes: dev.free_bytes,
            total_bytes: dev.total_bytes,
        };
        let json = serde_json::to_vec(&manifest).unwrap();
        respond(stream, 200, "application/json", &json);
        return;
    }
    if method == "POST" && path == "/v1/file" {
        let origin_b64 = headers.get("x-tucklet-origin").cloned().unwrap_or_default();
        let item: MediaItem =
            serde_json::from_slice(&b64::decode(&origin_b64).unwrap()).unwrap();
        dev.store.lock().unwrap().push((item, body));
        respond(stream, 200, "text/plain", b"stored");
        return;
    }
    if method == "GET" && path.starts_with("/v1/file/") {
        let id = path.trim_start_matches("/v1/file/");
        let found = dev
            .store
            .lock()
            .unwrap()
            .iter()
            .find(|(m, _)| m.id == id)
            .map(|(_, bytes)| bytes.clone());
        match found {
            Some(bytes) => respond(stream, 200, "application/octet-stream", &bytes),
            None => respond(stream, 404, "text/plain", b"not found"),
        }
        return;
    }
    if method == "DELETE" && path.starts_with("/v1/file/") {
        let id = path.trim_start_matches("/v1/file/").to_string();
        dev.store.lock().unwrap().retain(|(m, _)| m.id != id);
        respond(stream, 200, "text/plain", b"deleted");
        return;
    }
    respond(stream, 404, "text/plain", b"unknown");
}

fn hex_encode(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Spawn the mock device; returns its host string ("127.0.0.1:PORT").
fn spawn_device(enrolled_pubkey: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let host = listener.local_addr().unwrap().to_string();
    let mut allow = AllowList::new();
    allow.enroll(enrolled_pubkey.to_string(), "Test Desktop".to_string());
    let dev = Arc::new(Device {
        allow,
        nonce: core::array::from_fn(|i| (i as u8) ^ 0x5A),
        token: Mutex::new(None),
        store: Mutex::new(Vec::new()),
        free_bytes: 60_000_000_000,
        total_bytes: 64_000_000_000,
    });
    thread::spawn(move || {
        for conn in listener.incoming() {
            if let Ok(mut stream) = conn {
                handle(&dev, &mut stream);
            }
        }
    });
    host
}

fn auth_handshake(host: &str, id: &Identity) -> Option<String> {
    // GET nonce
    let r = http::request(host, "GET", "/auth/nonce", &[], None).ok()?;
    let nonce_hex = String::from_utf8_lossy(&r.body);
    let nonce_hex = nonce_hex
        .split("\"nonce\":\"")
        .nth(1)?
        .split('"')
        .next()?
        .to_string();
    let nonce = hex_decode(&nonce_hex)?;
    // sign + POST session
    let sig = id.sign_challenge(&nonce);
    let headers = [
        ("X-Tucklet-Pubkey", id.public_key_hex()),
        ("X-Tucklet-Sig", sig),
    ];
    let r = http::request(host, "POST", "/auth/session", &headers, Some(&[])).ok()?;
    if !r.is_success() {
        return None;
    }
    let body = String::from_utf8_lossy(&r.body);
    Some(body.split("\"token\":\"").nth(1)?.split('"').next()?.to_string())
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok())
        .collect()
}

#[test]
fn full_enroll_handshake_and_data_cycle() {
    // The desktop machine's real identity (shared crypto).
    let key_path = std::env::temp_dir().join(format!("itest-id-{}.key", std::process::id()));
    let _ = std::fs::remove_file(&key_path);
    let id = Identity::load_or_create_at(&key_path).unwrap();

    // Enroll (simulates the one-time button-press pairing) + start the device.
    let host = spawn_device(&id.public_key_hex());

    // Auth handshake with the REAL signer; the device verifies with tucklet-crypto.
    let token = auth_handshake(&host, &id).expect("handshake should succeed");
    let client = DataClient::new(host.clone(), token);

    // Empty to start.
    let m0 = client.manifest().unwrap();
    assert_eq!(m0.items.len(), 0);
    assert_eq!(m0.total_bytes, 64_000_000_000);

    // Push a file with its MediaItem (origin header round-trips through base64).
    let tmp_file = std::env::temp_dir().join(format!("itest-photo-{}.bin", std::process::id()));
    let payload = b"the actual photo bytes \xff\x00\xfemeow";
    std::fs::write(&tmp_file, payload).unwrap();
    let item = MediaItem {
        id: "photo-1".into(),
        name: "IMG_0001.heic".into(),
        size_bytes: payload.len() as u64,
        mime: "image/heic".into(),
        created_at: 1_700_000_000,
        origin: tucklet_desktop_core::proto::OriginMetadata {
            platform: tucklet_desktop_core::proto::Platform::Desktop,
            app: "Computer".into(),
            collection: "Desktop".into(),
            album: None,
            device_name: "Test Mac".into(),
        },
        state: tucklet_desktop_core::proto::ItemState::OnPhone,
        checksum: None,
    };
    client.upload(&tmp_file, &item).unwrap();

    // Manifest now shows it, with the flattened state intact over the wire.
    let m1 = client.manifest().unwrap();
    assert_eq!(m1.items.len(), 1);
    assert_eq!(m1.items[0].id, "photo-1");
    assert!(matches!(
        m1.items[0].state,
        tucklet_desktop_core::proto::ItemState::OnPhone
    ));

    // Pull it back; bytes must match exactly.
    let out = std::env::temp_dir().join(format!("itest-pulled-{}.bin", std::process::id()));
    let n = client.download("photo-1", &out).unwrap();
    assert_eq!(n, payload.len() as u64);
    assert_eq!(std::fs::read(&out).unwrap(), payload);

    // Delete; manifest empties.
    client.delete("photo-1").unwrap();
    assert_eq!(client.manifest().unwrap().items.len(), 0);

    let _ = std::fs::remove_file(&tmp_file);
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&key_path);
}

#[test]
fn wrong_signature_is_rejected_and_grants_no_token() {
    // A device that trusts the REAL key...
    let real = {
        let p = std::env::temp_dir().join(format!("itest-real-{}.key", std::process::id()));
        let _ = std::fs::remove_file(&p);
        Identity::load_or_create_at(&p).unwrap()
    };
    let host = spawn_device(&real.public_key_hex());

    // ...but an IMPOSTOR (different key) tries to authenticate. The allow-list
    // miss + signature mismatch must both block it.
    let impostor = {
        let p = std::env::temp_dir().join(format!("itest-imp-{}.key", std::process::id()));
        let _ = std::fs::remove_file(&p);
        Identity::load_or_create_at(&p).unwrap()
    };
    assert!(auth_handshake(&host, &impostor).is_none());

    // With no valid token, the data plane is closed.
    let client = DataClient::new(host, "forged-token".to_string());
    assert!(client.manifest().is_err());
}
