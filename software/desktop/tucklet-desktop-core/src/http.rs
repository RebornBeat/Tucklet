// http.rs
// A tiny, dependency-free HTTP/1.1 client over std::net, enough for the Tucklet
// /v1 API on the local link (plain HTTP, no TLS — the charm is a private AP).
// The request builder and response-head parser are pure functions so they can
// be unit-tested without a live socket.
//
// License: PolyForm Noncommercial 1.0.0

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

#[derive(Debug)]
pub enum HttpError {
    Io(std::io::Error),
    BadStatusLine,
    Incomplete,
}
impl From<std::io::Error> for HttpError {
    fn from(e: std::io::Error) -> Self {
        HttpError::Io(e)
    }
}
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::Io(e) => write!(f, "io: {e}"),
            HttpError::BadStatusLine => write!(f, "bad status line"),
            HttpError::Incomplete => write!(f, "incomplete response"),
        }
    }
}
impl std::error::Error for HttpError {}

/// Build the raw request head (everything before the body). Pure + testable.
pub fn build_request_head(
    method: &str,
    host: &str,
    path: &str,
    headers: &[(&str, String)],
    body_len: Option<usize>,
) -> String {
    let mut s = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n");
    for (k, v) in headers {
        s.push_str(&format!("{k}: {v}\r\n"));
    }
    if let Some(len) = body_len {
        s.push_str(&format!("Content-Length: {len}\r\n"));
    }
    s.push_str("\r\n");
    s
}

/// Parse the response head from a byte buffer. Returns (status, headers, index
/// where the body begins). Pure + testable.
pub fn parse_response_head(buf: &[u8]) -> Result<(u16, Vec<(String, String)>, usize), HttpError> {
    // Find the CRLFCRLF that ends the head.
    let sep = b"\r\n\r\n";
    let end = buf
        .windows(4)
        .position(|w| w == sep)
        .ok_or(HttpError::Incomplete)?;
    let head = std::str::from_utf8(&buf[..end]).map_err(|_| HttpError::BadStatusLine)?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().ok_or(HttpError::BadStatusLine)?;
    // "HTTP/1.1 200 OK"
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .ok_or(HttpError::BadStatusLine)?;
    let mut headers = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    Ok((status, headers, end + 4))
}

fn content_length(headers: &[(String, String)]) -> Option<usize> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse().ok())
}

/// Perform a request against `host` (an "ip:port" or "ip" — port defaults 80).
pub fn request(
    host: &str,
    method: &str,
    path: &str,
    headers: &[(&str, String)],
    body: Option<&[u8]>,
) -> Result<HttpResponse, HttpError> {
    let addr = if host.contains(':') {
        host.to_string()
    } else {
        format!("{host}:80")
    };
    let mut stream = TcpStream::connect(&addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let head = build_request_head(method, host, path, headers, body.map(|b| b.len()));
    stream.write_all(head.as_bytes())?;
    if let Some(b) = body {
        stream.write_all(b)?;
    }
    stream.flush()?;

    // Read the whole response (Connection: close => read to EOF).
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;

    let (status, hdrs, body_start) = parse_response_head(&raw)?;
    let body = if let Some(len) = content_length(&hdrs) {
        raw[body_start..(body_start + len).min(raw.len())].to_vec()
    } else {
        raw[body_start..].to_vec()
    };
    Ok(HttpResponse {
        status,
        headers: hdrs,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_head_has_required_lines() {
        let h = build_request_head(
            "GET",
            "192.168.4.1",
            "/v1/manifest",
            &[("X-Tucklet-Token", "abc".to_string())],
            None,
        );
        assert!(h.starts_with("GET /v1/manifest HTTP/1.1\r\n"));
        assert!(h.contains("Host: 192.168.4.1\r\n"));
        assert!(h.contains("X-Tucklet-Token: abc\r\n"));
        assert!(h.ends_with("\r\n\r\n"));
        assert!(!h.contains("Content-Length")); // no body
    }

    #[test]
    fn request_head_includes_content_length_for_body() {
        let h = build_request_head("POST", "h", "/v1/file", &[], Some(2048));
        assert!(h.contains("Content-Length: 2048\r\n"));
    }

    #[test]
    fn parses_status_and_headers_and_body_offset() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 5\r\n\r\nhello";
        let (status, headers, start) = parse_response_head(raw).unwrap();
        assert_eq!(status, 200);
        assert_eq!(
            headers.iter().find(|(k, _)| k == "Content-Length").unwrap().1,
            "5"
        );
        assert_eq!(&raw[start..], b"hello");
    }

    #[test]
    fn parse_incomplete_is_error() {
        assert!(matches!(
            parse_response_head(b"HTTP/1.1 200 OK\r\n"),
            Err(HttpError::Incomplete)
        ));
    }
}
