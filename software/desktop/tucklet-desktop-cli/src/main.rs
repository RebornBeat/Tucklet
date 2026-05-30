// Tucklet desktop CLI
// A working command-line client over the wireless /v1 API, useful on its own and
// as the reference driver for the GUI shell. The control/auth handshake (BLE)
// yields a host IP + session token; pass them here with --host/--token, or wire
// the `ble` feature to discover + handshake automatically.
//
// License: PolyForm Noncommercial 1.0.0

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::exit;
use tucklet_desktop_core::proto::{
    DataTransport, MediaItem, OriginMetadata, Platform, RadioKind,
};
use tucklet_desktop_core::{estimate, link, DataClient, TransferEngine};

struct Args {
    cmd: String,
    opts: BTreeMap<String, String>,
    positionals: Vec<String>,
}

fn parse_args(argv: &[String]) -> Option<Args> {
    let mut it = argv.iter();
    let cmd = it.next()?.clone();
    let mut opts = BTreeMap::new();
    let mut positionals = Vec::new();
    while let Some(a) = it.next() {
        if let Some(key) = a.strip_prefix("--") {
            let val = it.next().cloned().unwrap_or_default();
            opts.insert(key.to_string(), val);
        } else {
            positionals.push(a.clone());
        }
    }
    Some(Args {
        cmd,
        opts,
        positionals,
    })
}

fn need<'a>(a: &'a Args, key: &str) -> &'a str {
    a.opts.get(key).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("missing --{key}");
        exit(2)
    })
}

fn human_bytes(b: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < units.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} {}", v as u64, units[i])
    } else {
        format!("{v:.1} {}", units[i])
    }
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Some(a) => a,
        None => {
            usage();
            exit(2)
        }
    };

    match args.cmd.as_str() {
        "status" => {
            let c = client(&args);
            match c.manifest() {
                Ok(m) => println!(
                    "Tucklet: {} of {} free · {} items",
                    human_bytes(m.free_bytes),
                    human_bytes(m.total_bytes),
                    m.items.len()
                ),
                Err(e) => fail(e),
            }
        }
        "list" => {
            let c = client(&args);
            match c.manifest() {
                Ok(m) => {
                    let mut by_app: BTreeMap<String, Vec<&MediaItem>> = BTreeMap::new();
                    for it in &m.items {
                        by_app.entry(it.origin.app.clone()).or_default().push(it);
                    }
                    for (app, items) in by_app {
                        println!("{app}");
                        for it in items {
                            println!(
                                "  {}  {}  [{}]  {}",
                                it.id,
                                it.name,
                                state_label(&it.state),
                                human_bytes(it.size_bytes)
                            );
                        }
                    }
                }
                Err(e) => fail(e),
            }
        }
        "pull" => {
            let c = client(&args);
            let id = need(&args, "id");
            let out = PathBuf::from(need(&args, "out"));
            match c.download(id, &out) {
                Ok(n) => println!("pulled {} -> {} ({})", id, out.display(), human_bytes(n)),
                Err(e) => fail(e),
            }
        }
        "push" => {
            let c = client(&args);
            let file = PathBuf::from(need(&args, "file"));
            let name = args
                .opts
                .get("name")
                .cloned()
                .or_else(|| {
                    file.file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "upload.bin".into());
            let size = std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
            let item = MediaItem {
                id: name.clone(),
                name: name.clone(),
                size_bytes: size,
                mime: guess_mime(&name),
                created_at: 0,
                origin: OriginMetadata {
                    platform: Platform::Desktop,
                    app: "Computer".into(),
                    collection: "Desktop".into(),
                    album: None,
                    device_name: hostname(),
                },
                state: tucklet_desktop_core::proto::ItemState::OnPhone,
                checksum: None,
            };
            match c.upload(&file, &item) {
                Ok(()) => println!("pushed {} ({})", name, human_bytes(size)),
                Err(e) => fail(e),
            }
        }
        "estimate" => {
            let c = client(&args);
            match c.manifest() {
                Ok(m) => {
                    let ids: Vec<&str> = args.positionals.iter().map(|s| s.as_str()).collect();
                    let sizes: Vec<u64> = m
                        .items
                        .iter()
                        .filter(|it| ids.is_empty() || ids.contains(&it.id.as_str()))
                        .map(|it| it.size_bytes)
                        .collect();
                    let prof = link::profile_for(RadioKind::SingleC5, DataTransport::SoftAp);
                    let est = estimate::estimate_sizes(&sizes, prof);
                    println!(
                        "About {} for {} items ({})",
                        est.human(),
                        est.files,
                        human_bytes(est.bytes_total)
                    );
                    let _engine = TransferEngine::new(prof); // same engine the GUI uses
                }
                Err(e) => fail(e),
            }
        }
        _ => {
            usage();
            exit(2)
        }
    }
}

fn client(a: &Args) -> DataClient {
    DataClient::new(need(a, "host").to_string(), need(a, "token").to_string())
}

fn state_label(s: &tucklet_desktop_core::proto::ItemState) -> &'static str {
    use tucklet_desktop_core::proto::ItemState::*;
    match s {
        OnPhone => "On phone",
        OnTucklet => "On Tucklet",
        Temporary { .. } => "Temporary",
    }
}

fn guess_mime(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.ends_with(".mov") || lower.ends_with(".mp4") {
        "video/quicktime".into()
    } else if lower.ends_with(".png") {
        "image/png".into()
    } else if lower.ends_with(".heic") {
        "image/heic".into()
    } else {
        "image/jpeg".into()
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "Computer".into())
}

fn fail(e: impl std::fmt::Display) -> ! {
    eprintln!("error: {e}");
    exit(1)
}

fn usage() {
    eprintln!(
        "tucklet <command> --host <ip> --token <t> [opts]\n\
         commands:\n\
         \x20 status                          free/total + item count\n\
         \x20 list                            items grouped by app\n\
         \x20 pull --id <id> --out <path>     download a file\n\
         \x20 push --file <path> [--name n]   upload a file\n\
         \x20 estimate [id ...]               transfer-time estimate"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_command_opts_and_positionals() {
        let argv: Vec<String> = ["list", "--host", "192.168.4.1", "--token", "t", "id1", "id2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let a = parse_args(&argv).unwrap();
        assert_eq!(a.cmd, "list");
        assert_eq!(a.opts.get("host").unwrap(), "192.168.4.1");
        assert_eq!(a.opts.get("token").unwrap(), "t");
        assert_eq!(a.positionals, vec!["id1", "id2"]);
    }

    #[test]
    fn human_bytes_reads_naturally() {
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1536), "1.5 KB");
    }

    #[test]
    fn mime_guess() {
        assert_eq!(guess_mime("clip.MOV"), "video/quicktime");
        assert_eq!(guess_mime("p.heic"), "image/heic");
    }
}
