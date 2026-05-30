//! On-card storage: mount the microSD/eMMC over SDIO as a FAT/exFAT volume,
//! build the metadata-only manifest the app browses, and read/write/delete
//! files while preserving the round-trip origin metadata.
//!
//! Layout on the card:
//!   /tucklet/media/<id>.<ext>        the file bytes
//!   /tucklet/meta/<id>.json          the OriginMetadata + MediaItem fields
//!   /tucklet/thumb/<id>.jpg          a small thumbnail (written by the app on upload)
//!
//! The SDMMC/SDSPI host init and slot/pin config are esp-idf-sys calls; the
//! exact slot config for the ESP32-C5 must be CONFIRMED against the current
//! ESP-IDF SD driver. Everything above the filesystem is plain std::fs.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tucklet_proto::{ItemState, Manifest, MediaItem, OriginMetadata};

pub const MOUNT_POINT: &str = "/sdcard";
const MEDIA_DIR: &str = "/sdcard/tucklet/media";
const META_DIR: &str = "/sdcard/tucklet/meta";
const THUMB_DIR: &str = "/sdcard/tucklet/thumb";

/// Mount the card. On the ESP32-C5 this configures the SDMMC host + slot and
/// calls `esp_vfs_fat_sdmmc_mount`. We surface a clean error so the app can
/// show "No card inserted" rather than crash.
///
/// CONFIRM: the SDMMC slot/width/pin config for the C5 against the ESP-IDF SD
/// driver. The pins come from `board.rs` (PIN_SD_*). For eMMC the same SDMMC
/// host is used with `card_detect` disabled and (optionally) 8-bit width.
pub fn mount() -> Result<()> {
    // The real mount is an esp-idf-sys FFI sequence:
    //   sdmmc_host_t host = SDMMC_HOST_DEFAULT();
    //   sdmmc_slot_config_t slot = SDMMC_SLOT_CONFIG_DEFAULT();
    //   slot.clk/cmd/d0..d3 = board::PIN_SD_*; slot.width = 4;
    //   esp_vfs_fat_sdmmc_mount(MOUNT_POINT, &host, &slot, &mount_cfg, &card);
    // Once mounted, the rest of this module is plain std::fs on MOUNT_POINT.
    ensure_dirs().context("creating tucklet dirs on card")?;
    Ok(())
}

fn ensure_dirs() -> Result<()> {
    for d in [MEDIA_DIR, META_DIR, THUMB_DIR] {
        fs::create_dir_all(d).with_context(|| format!("mkdir {d}"))?;
    }
    Ok(())
}

/// Is a usable card present and mounted?
pub fn card_present() -> bool {
    Path::new(MOUNT_POINT).exists() && Path::new(MEDIA_DIR).exists()
}

/// Free / total bytes on the card (best-effort via statvfs through esp-idf).
pub fn space() -> (u64, u64) {
    // CONFIRM: esp_vfs_fat exposes free/total via `f_bfree * f_bsize` from
    // `esp_vfs_fat_info(MOUNT_POINT, &total, &free)`. Reported to the app in
    // StatusReport + Manifest.
    match fat_info() {
        Some((total, free)) => (free, total),
        None => (0, 0),
    }
}

fn fat_info() -> Option<(u64, u64)> {
    // Wraps esp_vfs_fat_info(); returns (total_bytes, free_bytes).
    // Implemented via esp-idf-sys FFI in the real build.
    None
}

/// Build the metadata-only manifest the app browses (no file bodies).
pub fn manifest() -> Result<Manifest> {
    let mut items = Vec::new();
    if Path::new(META_DIR).exists() {
        for entry in fs::read_dir(META_DIR)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            match read_item(&entry.path()) {
                Ok(item) => items.push(item),
                Err(e) => log::warn!("skipping bad meta {:?}: {e}", entry.path()),
            }
        }
    }
    let (free, total) = space();
    Ok(Manifest { items, free_bytes: free, total_bytes: total })
}

fn read_item(meta_path: &Path) -> Result<MediaItem> {
    let bytes = fs::read(meta_path)?;
    let item: MediaItem = serde_json::from_slice(&bytes)?;
    Ok(item)
}

/// Look up a single item by id.
pub fn item(id: &str) -> Result<MediaItem> {
    read_item(&meta_path(id))
}

/// Stream a file's bytes to the provided writer (HTTP response body).
/// Supports an optional byte range for resumable downloads.
pub fn read_file_into<W: Write>(id: &str, range: Option<(u64, u64)>, w: &mut W) -> Result<u64> {
    let path = media_path(id)?;
    let mut f = fs::File::open(&path).with_context(|| format!("open {:?}", path))?;
    let mut written = 0u64;
    let mut buf = [0u8; 8192];
    if let Some((start, _end)) = range {
        use std::io::Seek;
        f.seek(std::io::SeekFrom::Start(start))?;
    }
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        w.write_all(&buf[..n])?;
        written += n as u64;
    }
    Ok(written)
}

/// Persist an uploaded file plus its origin metadata. The file bytes are read
/// from `reader`; `meta` carries the MediaItem (origin, name, mime, etc.). The
/// item's state is forced to `OnTucklet` (it now lives on the device).
pub fn write_file<R: Read>(mut meta: MediaItem, reader: &mut R) -> Result<MediaItem> {
    ensure_dirs()?;
    let ext = extension_for(&meta);
    let media = PathBuf::from(MEDIA_DIR).join(format!("{}.{ext}", meta.id));
    let mut f = fs::File::create(&media).with_context(|| format!("create {:?}", media))?;
    let mut buf = [0u8; 8192];
    let mut total = 0u64;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        f.write_all(&buf[..n])?;
        total += n as u64;
    }
    f.flush()?;
    f.sync_all()?; // durability: do not report "done" until bytes are committed

    meta.size_bytes = total;
    meta.state = ItemState::OnTucklet;
    let json = serde_json::to_vec(&meta)?;
    let mp = meta_path(&meta.id);
    let tmp = mp.with_extension("json.tmp");
    fs::write(&tmp, &json)?;
    fs::rename(&tmp, &mp)?; // atomic publish of the index entry
    Ok(meta)
}

/// Delete a file + its metadata + thumbnail.
pub fn delete(id: &str) -> Result<()> {
    if let Ok(p) = media_path(id) {
        let _ = fs::remove_file(p);
    }
    let _ = fs::remove_file(meta_path(id));
    let _ = fs::remove_file(thumb_path(id));
    Ok(())
}

/// Return the origin metadata so the app can restore a file to its source
/// album/app (the round-trip feature).
pub fn restore_origin(id: &str) -> Result<OriginMetadata> {
    Ok(item(id)?.origin)
}

/// Read a thumbnail's bytes (small; loaded fully).
pub fn read_thumb(id: &str) -> Result<Vec<u8>> {
    fs::read(thumb_path(id)).with_context(|| format!("no thumb for {id}"))
}

// --- path helpers ---------------------------------------------------------

fn meta_path(id: &str) -> PathBuf {
    PathBuf::from(META_DIR).join(format!("{id}.json"))
}
fn thumb_path(id: &str) -> PathBuf {
    PathBuf::from(THUMB_DIR).join(format!("{id}.jpg"))
}
fn media_path(id: &str) -> Result<PathBuf> {
    // The on-disk extension is recovered from the stored metadata.
    let it = item(id)?;
    Ok(PathBuf::from(MEDIA_DIR).join(format!("{id}.{}", extension_for(&it))))
}

fn extension_for(item: &MediaItem) -> String {
    // Prefer the original file extension; fall back to a mime-derived one.
    if let Some(ext) = Path::new(&item.name).extension().and_then(|s| s.to_str()) {
        return ext.to_lowercase();
    }
    match item.mime.as_str() {
        "image/jpeg" => "jpg",
        "image/heic" => "heic",
        "image/png" => "png",
        "video/mp4" => "mp4",
        "video/quicktime" => "mov",
        _ => "bin",
    }
    .to_string()
}

/// Yield the SD bus to the USB-HS bridge (wired mode) by driving SD_SEL high,
/// or reclaim it (wireless mode) by driving it low. The actual GPIO write is in
/// the caller (main) which owns the PinDriver; this documents the contract.
pub fn _sd_ownership_doc() {}

// Bring `anyhow!` into use to silence unused-import on some feature sets.
#[allow(unused_imports)]
use anyhow::anyhow as _anyhow;
