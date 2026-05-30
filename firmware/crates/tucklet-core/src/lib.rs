//! # tucklet-core
//!
//! All the device logic that has nothing to do with a specific radio, OS, or
//! peripheral. Pure functions and small state machines, shared by the firmware
//! and (conceptually mirrored by) the apps. `no_std + alloc`.
//!
//! Modules:
//!   * [`estimate`]  — transfer-time estimation (the headline UX feature).
//!   * [`link`]      — realistic throughput profiles per device variant.
//!   * [`state`]     — the device runtime state machine.
//!   * [`trickle`]   — the background "drip" scheduler decision.
//!   * [`expiry`]    — Temporary-copy lifetime logic.
//!   * [`allowlist`] — paired-phone trust set.
//!   * [`variant`]   — resolve which transports are usable for a variant+platform.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use tucklet_proto::{DataTransport, Platform, RadioKind};

// ===========================================================================
// link — realistic throughput profiles (bytes/sec), grounded in measured data
// ===========================================================================
pub mod link {
    use super::*;

    /// A throughput model: sustained bytes/sec plus a fixed per-file overhead
    /// (handshake, metadata, filesystem) in seconds. Real transfers are
    /// dominated by per-file overhead when files are small, which is why this
    /// model is per-file, not a flat average.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct LinkProfile {
        pub sustained_bps: u64,
        pub per_file_overhead_ms: u32,
    }

    impl LinkProfile {
        pub const fn new(sustained_bps: u64, per_file_overhead_ms: u32) -> Self {
            Self { sustained_bps, per_file_overhead_ms }
        }
    }

    // Numbers below come from the measured reality in docs/TRANSFER_PERFORMANCE.md.
    // They are deliberately conservative (the "you will actually get this" end).

    /// Single ESP32-C5 over 5 GHz at close range: ~9 MB/s sustained.
    pub const C5_WIRELESS: LinkProfile = LinkProfile::new(9_000_000, 40);
    /// Dual-C5 aggregated (experimental): ~15 MB/s sustained.
    pub const DUAL_C5_WIRELESS: LinkProfile = LinkProfile::new(15_000_000, 40);
    /// Original ESP32-S3 (fallback radio): ~4 MB/s sustained.
    pub const S3_WIRELESS: LinkProfile = LinkProfile::new(4_000_000, 50);
    /// USB 2.0 High-Speed wired bridge: ~30 MB/s sustained.
    pub const WIRED_USB_HS: LinkProfile = LinkProfile::new(30_000_000, 8);

    /// Pick a conservative profile for a (radio, transport) pair.
    pub fn profile_for(radio: RadioKind, transport: DataTransport) -> LinkProfile {
        match transport {
            DataTransport::WiredUsbHs => WIRED_USB_HS,
            DataTransport::SoftAp | DataTransport::WifiAware => match radio {
                RadioKind::SingleC5 => C5_WIRELESS,
                RadioKind::DualC5 => DUAL_C5_WIRELESS,
            },
        }
    }
}

// ===========================================================================
// estimate — transfer-time estimation (the most-requested feature)
// ===========================================================================
pub mod estimate {
    use super::link::LinkProfile;
    use alloc::vec::Vec;

    /// Result of estimating a transfer.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Estimate {
        /// Whole seconds, rounded up (never tell the user "0s" for real work).
        pub seconds: u32,
        pub bytes_total: u64,
        pub files: u32,
    }

    impl Estimate {
        /// A short human label: "12s", "3 min", "1 hr 4 min".
        #[cfg(feature = "std")]
        pub fn human(&self) -> alloc::string::String {
            human_seconds(self.seconds)
        }
    }

    /// Estimate time to move a set of file sizes over a given link profile.
    ///
    /// time = sum_over_files( size / sustained + per_file_overhead )
    ///
    /// Per-file overhead is what makes "30 small photos" feel different from
    /// "one big video of the same total size" — and is exactly why the naive
    /// `total_bytes / speed` estimate lies to users.
    pub fn estimate_sizes(sizes: &[u64], link: LinkProfile) -> Estimate {
        let files = sizes.len() as u32;
        let bytes_total: u64 = sizes.iter().copied().sum();

        // Transfer time in milliseconds, computed in u128 to avoid overflow.
        let bps = link.sustained_bps.max(1) as u128;
        let mut ms: u128 = 0;
        for &s in sizes {
            // size / bps seconds  ->  *1000 ms
            ms += (s as u128 * 1000) / bps;
            ms += link.per_file_overhead_ms as u128;
        }

        let seconds = ((ms + 999) / 1000) as u64; // round up to whole seconds
        Estimate {
            seconds: seconds.min(u32::MAX as u64) as u32,
            bytes_total,
            files,
        }
    }

    /// Convenience: estimate from a Vec (e.g. mapped from `TransferItem`s).
    pub fn estimate_items(sizes: Vec<u64>, link: LinkProfile) -> Estimate {
        estimate_sizes(&sizes, link)
    }

    /// Recompute remaining time mid-transfer from measured throughput, so the
    /// live ETA reflects reality instead of the original guess.
    pub fn eta_seconds(bytes_remaining: u64, measured_bps: u64) -> u32 {
        if measured_bps == 0 {
            return u32::MAX;
        }
        let s = (bytes_remaining + measured_bps - 1) / measured_bps; // ceil
        s.min(u32::MAX as u64) as u32
    }

    #[cfg(feature = "std")]
    pub fn human_seconds(total: u32) -> alloc::string::String {
        use alloc::format;
        if total < 60 {
            format!("{total}s")
        } else if total < 3600 {
            let m = total / 60;
            let s = total % 60;
            if s == 0 { format!("{m} min") } else { format!("{m} min {s}s") }
        } else {
            let h = total / 3600;
            let m = (total % 3600) / 60;
            if m == 0 { format!("{h} hr") } else { format!("{h} hr {m} min") }
        }
    }
}

// ===========================================================================
// state — device runtime state machine
// ===========================================================================
pub mod state {
    /// Runtime states (see docs/FINAL_REVIEW.md §3).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DeviceState {
        /// Radios off, deep sleep. Wakes on button or BLE low-duty advertise window.
        Asleep,
        /// Low-duty BLE advertising; discoverable to paired phones (and, during a
        /// pairing window, to new ones awaiting the physical button press).
        Advertising,
        /// BLE connected + authenticated. STATUS notifications flow. No WiFi yet.
        Connected,
        /// High-speed data path up (SoftAP/Aware or wired). Files moving.
        Transferring,
    }

    /// Events that drive transitions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Event {
        ButtonShortPress,
        AuthenticatedPhoneConnected,
        SessionStarted,
        TransferComplete,
        IdleTimeout,
        PhoneDisconnected,
    }

    impl DeviceState {
        /// Pure transition function. Unknown (state,event) pairs are no-ops,
        /// which keeps the machine total and panic-free on firmware.
        pub fn next(self, ev: Event) -> DeviceState {
            use DeviceState::*;
            use Event::*;
            match (self, ev) {
                (Asleep, ButtonShortPress) => Advertising,
                (Advertising, AuthenticatedPhoneConnected) => Connected,
                (Advertising, IdleTimeout) => Asleep,
                (Connected, SessionStarted) => Transferring,
                (Connected, IdleTimeout) => Asleep,
                (Connected, PhoneDisconnected) => Advertising,
                (Transferring, TransferComplete) => Connected,
                (Transferring, PhoneDisconnected) => Advertising,
                (Transferring, IdleTimeout) => Connected,
                // Any other combination: stay put.
                (s, _) => s,
            }
        }
    }
}

// ===========================================================================
// trickle — background drip scheduler decision
// ===========================================================================
pub mod trickle {
    /// Live conditions used to decide whether to run a low-power background sync.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Conditions {
        pub phone_in_range: bool,
        pub phone_idle: bool,
        pub charging: bool,
        pub battery_percent: u8,
        pub pending_items: u32,
    }

    /// The trickle decision. Solving slow bandwidth with time, not speed:
    /// drip new items whenever it's cheap to do so, so the big transfer never
    /// has to happen.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Decision {
        pub should_run: bool,
        /// How many items to move in this drip window (small, to stay low-power).
        pub batch_size: u32,
    }

    /// Minimum battery to trickle on battery power (don't drain the charm).
    pub const MIN_BATTERY_ON_BATTERY: u8 = 30;

    pub fn decide(c: Conditions) -> Decision {
        if c.pending_items == 0 || !c.phone_in_range {
            return Decision { should_run: false, batch_size: 0 };
        }
        // Always fine to trickle while charging; otherwise require idle + healthy battery.
        let ok = c.charging || (c.phone_idle && c.battery_percent >= MIN_BATTERY_ON_BATTERY);
        if !ok {
            return Decision { should_run: false, batch_size: 0 };
        }
        // Larger drips while charging (power is free), small drips on battery.
        let batch_size = if c.charging { 25 } else { 5 };
        Decision { should_run: true, batch_size: batch_size.min(c.pending_items) }
    }
}

// ===========================================================================
// expiry — Temporary-copy lifetime logic
// ===========================================================================
pub mod expiry {
    use tucklet_proto::{EpochSeconds, ItemState};

    /// Is this item a Temporary copy that has passed its expiry as of `now`?
    pub fn is_expired(state: &ItemState, now: EpochSeconds) -> bool {
        match state {
            ItemState::Temporary { expires_at: Some(t) } => now >= *t,
            _ => false,
        }
    }

    /// Seconds until expiry (None = no expiry / not temporary / already expired-handled).
    pub fn seconds_until_expiry(state: &ItemState, now: EpochSeconds) -> Option<i64> {
        match state {
            ItemState::Temporary { expires_at: Some(t) } => Some((*t - now).max(0)),
            _ => None,
        }
    }
}

// ===========================================================================
// allowlist — paired-phone trust set
// ===========================================================================
pub mod allowlist {
    use alloc::vec::Vec;
    use alloc::string::String;

    /// A trusted phone: its long-term public key and a friendly name.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TrustedPhone {
        pub pubkey: String,
        pub name: String,
    }

    /// In-memory model of the device allow-list. The firmware persists this in
    /// NVS; this struct holds the logic so it can be unit-tested off-device.
    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub struct AllowList {
        phones: Vec<TrustedPhone>,
    }

    impl AllowList {
        pub fn new() -> Self { Self { phones: Vec::new() } }

        /// Enroll a phone (idempotent on pubkey). Returns true if newly added.
        pub fn enroll(&mut self, pubkey: String, name: String) -> bool {
            if self.contains(&pubkey) {
                // Update the friendly name but report "already known".
                if let Some(p) = self.phones.iter_mut().find(|p| p.pubkey == pubkey) {
                    p.name = name;
                }
                return false;
            }
            self.phones.push(TrustedPhone { pubkey, name });
            true
        }

        pub fn contains(&self, pubkey: &str) -> bool {
            self.phones.iter().any(|p| p.pubkey == pubkey)
        }

        /// Revoke a phone. Returns true if it was present.
        pub fn revoke(&mut self, pubkey: &str) -> bool {
            let before = self.phones.len();
            self.phones.retain(|p| p.pubkey != pubkey);
            self.phones.len() != before
        }

        /// Wipe everything (factory reset / long button hold).
        pub fn clear(&mut self) { self.phones.clear(); }

        pub fn len(&self) -> usize { self.phones.len() }
        pub fn is_empty(&self) -> bool { self.phones.is_empty() }
        pub fn phones(&self) -> &[TrustedPhone] { &self.phones }
    }
}

// ===========================================================================
// variant — which transports are actually usable for a build + platform
// ===========================================================================
pub mod variant {
    use super::*;
    use alloc::vec::Vec;
    use tucklet_proto::DeviceCapabilities;

    /// Filter a device's advertised transports down to those the given client
    /// platform can really use right now, ordered best-first.
    ///
    /// Rationale (see ADR-002/003):
    ///   * Wi-Fi Aware needs platform support (iOS 26+/Android 8+) AND a
    ///     certified/NAN-capable build; conservatively gated here.
    ///   * SoftAP works on every platform.
    ///   * Wired works on desktop trivially; on phones only via the USB-C bridge.
    pub fn usable_transports(
        caps: &DeviceCapabilities,
        platform: Platform,
    ) -> Vec<DataTransport> {
        let mut out: Vec<DataTransport> = Vec::new();

        // Seamless wireless first, if both sides support it.
        if caps.supports(DataTransport::WifiAware) && platform_supports_aware(platform) {
            out.push(DataTransport::WifiAware);
        }
        // Universal wireless fallback.
        if caps.supports(DataTransport::SoftAp) {
            out.push(DataTransport::SoftAp);
        }
        // Wired fast path.
        if caps.supports(DataTransport::WiredUsbHs) {
            out.push(DataTransport::WiredUsbHs);
        }
        out
    }

    fn platform_supports_aware(platform: Platform) -> bool {
        match platform {
            // Assumed minimum platform versions handled by the app at runtime;
            // here we only express "the platform CAN do Aware in principle".
            Platform::Android | Platform::Ios => true,
            Platform::Desktop => false,
        }
    }
}

// ===========================================================================
// session — single-use transport credentials + TTL (pure, host-testable)
// ===========================================================================
pub mod session {
    use alloc::string::String;
    use alloc::format;
    use tucklet_proto::{DataTransport, EpochSeconds, SessionGrant};

    /// Default lifetime of a transfer session's credentials, in seconds.
    pub const DEFAULT_TTL_S: u32 = 600;

    /// A live transfer session. The firmware mints one per authorized transfer,
    /// hands the `SessionGrant` to the phone over BLE, brings up the data path,
    /// and tears everything down when the session expires or completes.
    ///
    /// Credentials are single-use and short-lived: a captured SoftAP password
    /// is worthless once the session ends (see PROTOCOL §4).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Session {
        pub token: String,
        pub ssid_or_service: String,
        pub psk: String,
        pub ip: String,
        pub transport: DataTransport,
        pub issued_at: EpochSeconds,
        pub ttl_s: u32,
    }

    impl Session {
        /// Mint a new session. `rand16` supplies 16 random bytes (from the
        /// hardware RNG on-device); kept as a parameter so this is pure and
        /// testable. The IP is fixed for SoftAP (the device's AP gateway).
        pub fn mint(
            transport: DataTransport,
            now: EpochSeconds,
            rand16: [u8; 16],
            ttl_s: u32,
        ) -> Self {
            let hex = hex16(&rand16);
            // SSID is short + human-recognizable; PSK + token are full entropy.
            let suffix = &hex[..4];
            let ssid_or_service = match transport {
                DataTransport::WifiAware => format!("tucklet.{suffix}"),
                _ => format!("Tucklet-{suffix}"),
            };
            Session {
                token: format!("tk_{hex}"),
                ssid_or_service,
                psk: format!("{hex}{hex}")[..16].into(), // 16-char one-time PSK
                ip: String::from("192.168.4.1"),
                transport,
                issued_at: now,
                ttl_s,
            }
        }

        /// Is the session still valid at `now`?
        pub fn is_valid(&self, now: EpochSeconds) -> bool {
            now >= self.issued_at && (now - self.issued_at) < self.ttl_s as i64
        }

        /// Seconds left before expiry (0 once expired).
        pub fn remaining_s(&self, now: EpochSeconds) -> u32 {
            let elapsed = (now - self.issued_at).max(0);
            (self.ttl_s as i64 - elapsed).max(0) as u32
        }

        /// Does a presented bearer token match this session (and is it valid)?
        pub fn authorize(&self, presented_token: &str, now: EpochSeconds) -> bool {
            self.is_valid(now) && constant_time_eq(self.token.as_bytes(), presented_token.as_bytes())
        }

        /// The wire grant handed to the phone over BLE.
        pub fn grant(&self) -> SessionGrant {
            SessionGrant {
                ssid_or_service: self.ssid_or_service.clone(),
                psk: self.psk.clone(),
                ip: self.ip.clone(),
                token: self.token.clone(),
                ttl_seconds: self.remaining_s(self.issued_at), // == ttl at issue
            }
        }
    }

    fn hex16(b: &[u8; 16]) -> String {
        let mut s = String::with_capacity(32);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for &byte in b.iter() {
            s.push(HEX[(byte >> 4) as usize] as char);
            s.push(HEX[(byte & 0x0f) as usize] as char);
        }
        s
    }

    /// Length-independent-ish constant-time compare (avoids early-exit timing
    /// leaks on the token check). Both sides are short fixed tokens.
    fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut diff = 0u8;
        for i in 0..a.len() {
            diff |= a[i] ^ b[i];
        }
        diff == 0
    }
}

// ===========================================================================
// tests
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    // ---- estimate ----------------------------------------------------------
    #[test]
    fn estimate_one_big_video_is_dominated_by_bytes() {
        // 500 MB over 9 MB/s ~= 55.6s, plus one 40ms overhead -> 56s (rounded up).
        let est = estimate::estimate_sizes(&[500_000_000], link::C5_WIRELESS);
        assert_eq!(est.files, 1);
        assert_eq!(est.bytes_total, 500_000_000);
        assert!((55..=57).contains(&est.seconds), "got {}", est.seconds);
    }

    #[test]
    fn estimate_many_small_photos_includes_per_file_overhead() {
        // 30 photos x 4 MB = 120 MB. Pure bytes: 120e6/9e6 ~= 13.3s.
        // Plus 30 x 40ms overhead = 1.2s -> ~15s total. The overhead matters.
        let sizes: Vec<u64> = vec![4_000_000; 30];
        let est = estimate::estimate_sizes(&sizes, link::C5_WIRELESS);
        assert_eq!(est.files, 30);
        assert!((14..=16).contains(&est.seconds), "got {}", est.seconds);
    }

    #[test]
    fn wired_is_much_faster_than_wireless_for_the_same_job() {
        let sizes: Vec<u64> = vec![4_000_000; 30];
        let wireless = estimate::estimate_sizes(&sizes, link::C5_WIRELESS);
        let wired = estimate::estimate_sizes(&sizes, link::WIRED_USB_HS);
        assert!(wired.seconds < wireless.seconds);
    }

    #[test]
    fn live_eta_uses_measured_throughput() {
        // 90 MB remaining at a measured 9 MB/s -> 10s.
        assert_eq!(estimate::eta_seconds(90_000_000, 9_000_000), 10);
        assert_eq!(estimate::eta_seconds(1, 0), u32::MAX);
    }

    #[cfg(feature = "std")]
    #[test]
    fn human_labels_read_naturally() {
        assert_eq!(estimate::human_seconds(15), "15s");
        assert_eq!(estimate::human_seconds(60), "1 min");
        assert_eq!(estimate::human_seconds(75), "1 min 15s");
        assert_eq!(estimate::human_seconds(3600), "1 hr");
        assert_eq!(estimate::human_seconds(3840), "1 hr 4 min");
    }

    // ---- state machine -----------------------------------------------------
    #[test]
    fn state_machine_happy_path() {
        use state::{DeviceState::*, Event::*};
        let mut s = Asleep;
        s = s.next(ButtonShortPress);
        assert_eq!(s, Advertising);
        s = s.next(AuthenticatedPhoneConnected);
        assert_eq!(s, Connected);
        s = s.next(SessionStarted);
        assert_eq!(s, Transferring);
        s = s.next(TransferComplete);
        assert_eq!(s, Connected);
        s = s.next(IdleTimeout);
        assert_eq!(s, Asleep);
    }

    #[test]
    fn state_machine_ignores_impossible_events() {
        use state::{DeviceState::*, Event::*};
        assert_eq!(Asleep.next(TransferComplete), Asleep);
        assert_eq!(Transferring.next(ButtonShortPress), Transferring);
    }

    // ---- trickle -----------------------------------------------------------
    #[test]
    fn trickle_runs_bigger_batches_while_charging() {
        let d = trickle::decide(trickle::Conditions {
            phone_in_range: true,
            phone_idle: false,
            charging: true,
            battery_percent: 50,
            pending_items: 100,
        });
        assert!(d.should_run);
        assert_eq!(d.batch_size, 25);
    }

    #[test]
    fn trickle_skips_when_low_battery_and_not_charging() {
        let d = trickle::decide(trickle::Conditions {
            phone_in_range: true,
            phone_idle: true,
            charging: false,
            battery_percent: 20,
            pending_items: 100,
        });
        assert!(!d.should_run);
    }

    #[test]
    fn trickle_skips_when_nothing_pending_or_out_of_range() {
        let base = trickle::Conditions {
            phone_in_range: true, phone_idle: true, charging: true,
            battery_percent: 90, pending_items: 0,
        };
        assert!(!trickle::decide(base).should_run);
        let oo = trickle::Conditions { phone_in_range: false, pending_items: 5, ..base };
        assert!(!trickle::decide(oo).should_run);
    }

    // ---- expiry ------------------------------------------------------------
    #[test]
    fn temporary_expiry_logic() {
        use tucklet_proto::ItemState;
        let s = ItemState::Temporary { expires_at: Some(1000) };
        assert!(!expiry::is_expired(&s, 999));
        assert!(expiry::is_expired(&s, 1000));
        assert_eq!(expiry::seconds_until_expiry(&s, 600), Some(400));
        let keep = ItemState::Temporary { expires_at: None };
        assert!(!expiry::is_expired(&keep, i64::MAX));
        assert_eq!(expiry::seconds_until_expiry(&ItemState::OnPhone, 0), None);
    }

    // ---- allowlist ---------------------------------------------------------
    #[test]
    fn allowlist_enroll_revoke_clear() {
        use allowlist::AllowList;
        let mut al = AllowList::new();
        assert!(al.enroll("pkA".into(), "Ana".into()));
        assert!(!al.enroll("pkA".into(), "Ana 2".into())); // already known
        assert_eq!(al.phones()[0].name, "Ana 2"); // name updated
        assert!(al.contains("pkA"));
        assert!(al.enroll("pkB".into(), "Bo".into()));
        assert_eq!(al.len(), 2);
        assert!(al.revoke("pkA"));
        assert!(!al.revoke("pkA"));
        assert_eq!(al.len(), 1);
        al.clear();
        assert!(al.is_empty());
    }

    // ---- variant resolution ------------------------------------------------
    #[test]
    fn variant_transport_resolution_per_platform() {
        use tucklet_proto::{DeviceCapabilities, PROTOCOL_VERSION, StorageKind};
        let caps = DeviceCapabilities {
            protocol_version: PROTOCOL_VERSION,
            firmware_version: "0.1.0".into(),
            radio: RadioKind::SingleC5,
            storage: StorageKind::MicroSd,
            transports: vec![
                DataTransport::SoftAp,
                DataTransport::WifiAware,
                DataTransport::WiredUsbHs,
            ],
            device_id: "TCK-1".into(),
        };
        // iOS: Aware first, then SoftAP, then wired.
        let ios = variant::usable_transports(&caps, Platform::Ios);
        assert_eq!(ios[0], DataTransport::WifiAware);
        assert!(ios.contains(&DataTransport::SoftAp));
        // Desktop: no Aware; SoftAP + wired.
        let desk = variant::usable_transports(&caps, Platform::Desktop);
        assert!(!desk.contains(&DataTransport::WifiAware));
        assert!(desk.contains(&DataTransport::WiredUsbHs));
    }

    // ---- session credentials ----------------------------------------------
    #[test]
    fn session_mint_and_validity_window() {
        use session::Session;
        use tucklet_proto::DataTransport;
        let r = [0xABu8; 16];
        let s = Session::mint(DataTransport::SoftAp, 1000, r, session::DEFAULT_TTL_S);
        assert!(s.ssid_or_service.starts_with("Tucklet-"));
        assert_eq!(s.psk.len(), 16);
        assert!(s.token.starts_with("tk_"));
        // valid at issue, valid just before expiry, invalid at/after expiry
        assert!(s.is_valid(1000));
        assert!(s.is_valid(1000 + session::DEFAULT_TTL_S as i64 - 1));
        assert!(!s.is_valid(1000 + session::DEFAULT_TTL_S as i64));
        // not valid "before" it was issued (clock skew guard)
        assert!(!s.is_valid(999));
        assert_eq!(s.remaining_s(1000), session::DEFAULT_TTL_S);
        assert_eq!(s.remaining_s(1000 + 100), session::DEFAULT_TTL_S - 100);
        assert_eq!(s.remaining_s(1_000_000), 0);
    }

    #[test]
    fn session_token_authorization_is_exact() {
        use session::Session;
        use tucklet_proto::DataTransport;
        let s = Session::mint(DataTransport::SoftAp, 0, [1u8; 16], 600);
        assert!(s.authorize(&s.token, 10));
        assert!(!s.authorize("tk_wrong", 10));
        assert!(!s.authorize(&s.token, 10_000)); // expired
        // grant mirrors the session
        let g = s.grant();
        assert_eq!(g.token, s.token);
        assert_eq!(g.ip, "192.168.4.1");
        assert_eq!(g.ttl_seconds, 600);
    }

    #[test]
    fn session_aware_uses_service_name() {
        use session::Session;
        use tucklet_proto::DataTransport;
        let s = Session::mint(DataTransport::WifiAware, 0, [0x5Au8; 16], 600);
        assert!(s.ssid_or_service.starts_with("tucklet."));
    }
}
