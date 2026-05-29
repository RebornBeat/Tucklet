//! # tucklet-proto
//!
//! The single source of truth for everything that crosses the wire between the
//! Tucklet device, its firmware, and the companion apps (Android / iOS / desktop).
//!
//! Two planes (see `docs/protocol/PROTOCOL.md`):
//!   * Control plane — small JSON messages over BLE GATT.
//!   * Data plane    — file metadata + transfers over the local HTTP API
//!                     (carried by WiFi SoftAP or Wi-Fi Aware).
//!
//! This crate is `no_std + alloc` so the exact same types compile into firmware
//! and into the apps. Enable the default `std` feature for host builds.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Protocol version. Apps negotiate against the device's reported value.
pub const PROTOCOL_VERSION: u16 = 1;

/// Epoch seconds (UTC). Kept as a plain integer so the type is no_std friendly
/// and identical across Rust firmware, Kotlin, and Swift.
pub type EpochSeconds = i64;

// ---------------------------------------------------------------------------
// Device variants — the full build matrix
// ---------------------------------------------------------------------------

/// Which radio configuration the device was built with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadioKind {
    /// Single ESP32-C5 (dual-band Wi-Fi 6 + BLE). The recommended v1.
    SingleC5,
    /// Two ESP32-C5 radios with link aggregation (experimental, higher speed).
    DualC5,
}

/// Which storage the device was built with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageKind {
    /// User-swappable microSD card.
    MicroSd,
    /// Soldered eMMC of the given capacity in gibibytes.
    Emmc { capacity_gib: u32 },
}

/// Which high-speed wireless data transports the firmware build supports.
/// SoftAP is always present; Wi-Fi Aware is the optional seamless upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataTransport {
    /// WiFi SoftAP + local HTTP. Works on every phone; the universal baseline.
    SoftAp,
    /// Wi-Fi Aware (NAN) data path. Seamless (no "join network" prompt) where
    /// the chipset + platform + certification allow.
    WifiAware,
    /// USB 2.0 High-Speed mass-storage via the wired bridge (~20-40 MB/s).
    WiredUsbHs,
}

/// The full capability descriptor a device advertises so every app knows
/// exactly what this physical unit can do. Covers the entire variant matrix:
/// radio (single/dual) x storage (microSD/eMMC) x transports (SoftAP/Aware/wired).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub protocol_version: u16,
    pub firmware_version: String,
    pub radio: RadioKind,
    pub storage: StorageKind,
    /// All data transports this firmware build supports, best first.
    pub transports: Vec<DataTransport>,
    /// Hardware serial / DID (public identifier; never a secret).
    pub device_id: String,
}

impl DeviceCapabilities {
    pub fn supports(&self, t: DataTransport) -> bool {
        self.transports.iter().any(|x| *x == t)
    }

    /// The best wireless transport available, preferring the seamless Wi-Fi
    /// Aware path over SoftAP. Returns `None` if only wired is present.
    pub fn best_wireless(&self) -> Option<DataTransport> {
        if self.supports(DataTransport::WifiAware) {
            Some(DataTransport::WifiAware)
        } else if self.supports(DataTransport::SoftAp) {
            Some(DataTransport::SoftAp)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Storage state (the plain-language UX contract — never the word "cache")
// ---------------------------------------------------------------------------

/// Where an item currently lives, in the exact vocabulary the UI shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum ItemState {
    /// "On phone" — exists on the phone only.
    OnPhone,
    /// "On Tucklet" — exists on the device only.
    OnTucklet,
    /// "Temporary" — a copy pulled to the phone that auto-removes at `expires_at`.
    /// `expires_at: None` means "keep" (no expiry).
    Temporary { expires_at: Option<EpochSeconds> },
}

/// How long a user wants a Temporary copy to live. Maps to the picker:
/// 1 hour / 1 day / 1 week / Keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporaryPolicy {
    OneHour,
    OneDay,
    OneWeek,
    Keep,
}

impl TemporaryPolicy {
    /// Seconds of lifetime, or `None` for Keep.
    pub fn lifetime_seconds(self) -> Option<i64> {
        match self {
            TemporaryPolicy::OneHour => Some(3_600),
            TemporaryPolicy::OneDay => Some(86_400),
            TemporaryPolicy::OneWeek => Some(604_800),
            TemporaryPolicy::Keep => None,
        }
    }

    /// Resolve to a concrete `ItemState::Temporary` from "now".
    pub fn resolve(self, now: EpochSeconds) -> ItemState {
        ItemState::Temporary {
            expires_at: self.lifetime_seconds().map(|s| now + s),
        }
    }
}

// ---------------------------------------------------------------------------
// Origin metadata (the round-trip "put it back exactly where it came from")
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Android,
    Ios,
    Desktop,
}

/// Remembers where a file came from so a restore lands it back in the same
/// album/app rather than an orphaned folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OriginMetadata {
    pub platform: Platform,
    /// User-facing bucket: "Camera", "Screenshots", "WhatsApp", ...
    pub app: String,
    /// Platform location hint used on restore, e.g. "DCIM/Camera".
    pub collection: String,
    /// Album / collection name if any.
    pub album: Option<String>,
    pub device_name: String,
}

// ---------------------------------------------------------------------------
// Media items + manifest (browse metadata only; bodies move on demand)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub mime: String,
    pub created_at: EpochSeconds,
    pub origin: OriginMetadata,
    #[serde(flatten)]
    pub state: ItemState,
    /// Content hash (e.g. blake3) for dedup / skip-already-transferred.
    pub checksum: Option<String>,
}

impl MediaItem {
    pub fn is_image(&self) -> bool {
        self.mime.starts_with("image/")
    }
    pub fn is_video(&self) -> bool {
        self.mime.starts_with("video/")
    }
}

/// The result of `GET /v1/manifest`: metadata for everything stored, plus space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub items: Vec<MediaItem>,
    pub free_bytes: u64,
    pub total_bytes: u64,
}

// ---------------------------------------------------------------------------
// Control plane (BLE) messages
// ---------------------------------------------------------------------------

/// `STATUS` characteristic payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusReport {
    pub battery_percent: u8,
    pub charging: bool,
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub card_present: bool,
    pub firmware_version: String,
}

/// Phone -> device on the `AUTH` characteristic (first-time pairing only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairRequest {
    /// Phone's long-term public key (X25519), hex-encoded.
    pub phone_pubkey: String,
    pub phone_name: String,
}

/// Device -> phone result of pairing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairResponse {
    pub paired: bool,
    /// Present when paired: the device's public key for mutual auth.
    pub device_pubkey: Option<String>,
    pub reason: Option<String>,
}

/// Phone -> device on the `SESSION` characteristic to open a transfer session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRequest {
    /// Signature over the device-issued nonce, proving key possession.
    pub challenge_signature: String,
    /// Which transport the app wants to use (must be in capabilities).
    pub transport: DataTransport,
}

/// Device -> phone: single-use credentials for the data plane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionGrant {
    /// For SoftAP: the one-time SSID. For Wi-Fi Aware: the service name.
    pub ssid_or_service: String,
    /// One-time pre-shared key (SoftAP) or pairing passphrase (Aware).
    pub psk: String,
    /// Local IP of the device's HTTP server, e.g. "192.168.4.1".
    pub ip: String,
    /// Bearer token for every HTTP request (`X-Tucklet-Token`).
    pub token: String,
    /// Seconds until the credentials expire.
    pub ttl_seconds: u32,
}

/// Small fire-and-forget commands on the `COMMAND` characteristic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum Command {
    Sleep,
    FactoryResetConfirm,
    /// Ask the device to begin a low-power background trickle if items are pending.
    BeginTrickle,
}

// ---------------------------------------------------------------------------
// Transfers (the part the user feels most: speed + ETA)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferKind {
    /// Phone -> Tucklet (offload, free up the phone).
    Offload,
    /// Tucklet -> phone (load a copy onto the phone).
    Load,
}

/// How the user wants a transfer executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferMode {
    /// Move everything now, foreground, with a progress bar + ETA.
    Batch,
    /// Low-power background drip whenever the device is near + idle/charging.
    Trickle,
}

/// One item queued in a transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferItem {
    pub id: String,
    pub size_bytes: u64,
    pub mime: String,
}

/// A complete transfer request from the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferRequest {
    pub kind: TransferKind,
    pub mode: TransferMode,
    pub items: Vec<TransferItem>,
    /// For Load transfers, the Temporary policy to apply to the resulting copies.
    pub temporary_policy: Option<TemporaryPolicy>,
}

/// A live progress update streamed back to the app during a Batch transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferProgress {
    pub items_total: u32,
    pub items_done: u32,
    pub bytes_total: u64,
    pub bytes_done: u64,
    /// Best current estimate of seconds remaining (see tucklet-core estimator).
    pub eta_seconds: u32,
    /// Measured instantaneous throughput in bytes/sec.
    pub throughput_bps: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn sample_origin() -> OriginMetadata {
        OriginMetadata {
            platform: Platform::Ios,
            app: "Camera".into(),
            collection: "DCIM/Camera".into(),
            album: Some("Summer".into()),
            device_name: "Ana's iPhone".into(),
        }
    }

    #[test]
    fn item_state_roundtrips_through_json() {
        let states = vec![
            ItemState::OnPhone,
            ItemState::OnTucklet,
            ItemState::Temporary { expires_at: Some(1_900_000_000) },
            ItemState::Temporary { expires_at: None },
        ];
        for s in states {
            let j = serde_json::to_string(&s).unwrap();
            let back: ItemState = serde_json::from_str(&j).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn temporary_policy_resolves_correctly() {
        let now = 1_000_000;
        assert_eq!(
            TemporaryPolicy::OneHour.resolve(now),
            ItemState::Temporary { expires_at: Some(now + 3_600) }
        );
        assert_eq!(
            TemporaryPolicy::Keep.resolve(now),
            ItemState::Temporary { expires_at: None }
        );
    }

    #[test]
    fn capabilities_pick_best_wireless() {
        let mut caps = DeviceCapabilities {
            protocol_version: PROTOCOL_VERSION,
            firmware_version: "0.1.0".into(),
            radio: RadioKind::SingleC5,
            storage: StorageKind::Emmc { capacity_gib: 128 },
            transports: vec![DataTransport::SoftAp, DataTransport::WiredUsbHs],
            device_id: "TCK-0001".into(),
        };
        assert_eq!(caps.best_wireless(), Some(DataTransport::SoftAp));
        caps.transports.push(DataTransport::WifiAware);
        assert_eq!(caps.best_wireless(), Some(DataTransport::WifiAware));
        assert!(caps.supports(DataTransport::WiredUsbHs));
    }

    #[test]
    fn media_item_full_roundtrip() {
        let item = MediaItem {
            id: "itm_1".into(),
            name: "IMG_2087.HEIC".into(),
            size_bytes: 4_382_002,
            mime: "image/heic".into(),
            created_at: 1_900_000_000,
            origin: sample_origin(),
            state: ItemState::OnTucklet,
            checksum: Some("blake3:abc".into()),
        };
        assert!(item.is_image());
        assert!(!item.is_video());
        let j = serde_json::to_string(&item).unwrap();
        let back: MediaItem = serde_json::from_str(&j).unwrap();
        assert_eq!(item, back);
    }

    #[test]
    fn storage_kind_encodes_capacity() {
        let s = StorageKind::Emmc { capacity_gib: 64 };
        let j = serde_json::to_string(&s).unwrap();
        assert!(j.contains("64"));
        let back: StorageKind = serde_json::from_str(&j).unwrap();
        assert_eq!(s, back);
    }
}
