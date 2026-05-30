// tucklet-desktop-core
// The desktop client's hardware-independent core: the /v1 HTTP data client, the
// transfer engine (built on the shared, tested tucklet-core estimator), the auth
// seam, and the BLE discovery seam. Reuses tucklet-proto + tucklet-core so the
// desktop speaks byte-identical wire format to the firmware and the iOS/Android
// apps.
//
// On the WIRED path the device mounts as a plain USB Mass Storage drive — no app
// required. This crate powers the WIRELESS path (and a future GUI/CLI).
//
// License: PolyForm Noncommercial 1.0.0

pub mod auth;
pub mod b64;
pub mod client;
pub mod discovery;
pub mod http;
pub mod transfer;

// Re-export the shared types so consumers depend on one crate.
pub use tucklet_core::{allowlist, estimate, link, trickle};
pub use tucklet_crypto;
pub use tucklet_proto as proto;

pub use client::{ClientError, DataClient};
pub use discovery::{DeviceHandle, Discovery, NullDiscovery, SERVICE_UUID};
pub use transfer::{Progress, TransferEngine};

/// Pick the conservative link profile for a connected device + transport.
pub fn link_profile_for(
    radio: proto::RadioKind,
    transport: proto::DataTransport,
) -> link::LinkProfile {
    link::profile_for(radio, transport)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexports_resolve_and_profile_picks_wired_fastest() {
        let wired = link_profile_for(proto::RadioKind::SingleC5, proto::DataTransport::WiredUsbHs);
        let wireless = link_profile_for(proto::RadioKind::SingleC5, proto::DataTransport::SoftAp);
        assert!(wired.sustained_bps > wireless.sustained_bps);
    }
}
