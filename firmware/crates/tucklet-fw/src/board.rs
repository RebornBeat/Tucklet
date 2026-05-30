//! Board definition: turns the compile-time variant features into a runtime
//! [`DeviceCapabilities`] descriptor, and centralizes the signal->GPIO pin map.
//!
//! The GPIO numbers below are placeholders flagged `CONFIRM`: assign each to a
//! free ESP32-C5 GPIO from the **ESP32-C5-WROOM-1 datasheet**, keeping the
//! strapping pins clean. This is the same one-step reconciliation documented in
//! `hardware/variants/<variant>/PIN_MAP.md`.

use tucklet_proto::{DataTransport, DeviceCapabilities, RadioKind, StorageKind, PROTOCOL_VERSION};

/// Firmware version string surfaced in STATUS and capabilities.
pub const FIRMWARE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// eMMC capacity advertised when built with `--features emmc`. Edit per SKU.
pub const EMMC_CAPACITY_GIB: u32 = 64;

// --- Pin map (CONFIRM against ESP32-C5-WROOM-1 datasheet) ------------------
// SDIO 4-bit to the SD mux (U6) common/A side.
pub const PIN_SD_CLK: u32 = 14;
pub const PIN_SD_CMD: u32 = 15;
pub const PIN_SD_D0: u32 = 2;
pub const PIN_SD_D1: u32 = 4;
pub const PIN_SD_D2: u32 = 12;
pub const PIN_SD_D3: u32 = 13;
pub const PIN_SD_SEL: u32 = 21; // drives U6: high = USB-HS bridge owns the card
#[cfg(feature = "microsd")]
pub const PIN_SD_DET: u32 = 16; // microSD card-detect

// I2C to the MAX17048 fuel gauge.
pub const PIN_I2C_SDA: u32 = 8;
pub const PIN_I2C_SCL: u32 = 9;
pub const PIN_GAUGE_ALRT: u32 = 10;

// UI + charger.
pub const PIN_BUTTON: u32 = 5;
pub const PIN_LED: u32 = 6; // WS2812 data
pub const PIN_CHG_STAT: u32 = 7;

// USB VBUS-present sense (decides wired vs wireless storage ownership).
pub const PIN_VBUS_SENSE: u32 = 18;

/// Build the capability descriptor for THIS firmware build from its features.
///
/// Compile-time guards ensure exactly one radio_* and one storage_* feature.
pub fn capabilities(device_id: String) -> DeviceCapabilities {
    let radio = radio_kind();
    let storage = storage_kind();
    let transports = transports();
    DeviceCapabilities {
        protocol_version: PROTOCOL_VERSION,
        firmware_version: FIRMWARE_VERSION.into(),
        radio,
        storage,
        transports,
        device_id,
    }
}

#[cfg(all(feature = "singlec5", not(feature = "dualc5")))]
fn radio_kind() -> RadioKind {
    RadioKind::SingleC5
}
#[cfg(all(feature = "dualc5", not(feature = "singlec5")))]
fn radio_kind() -> RadioKind {
    RadioKind::DualC5
}
#[cfg(any(
    all(feature = "singlec5", feature = "dualc5"),
    all(not(feature = "singlec5"), not(feature = "dualc5"))
))]
compile_error!("select exactly one radio feature: `singlec5` or `dualc5`");

#[cfg(all(feature = "microsd", not(feature = "emmc")))]
fn storage_kind() -> StorageKind {
    StorageKind::MicroSd
}
#[cfg(all(feature = "emmc", not(feature = "microsd")))]
fn storage_kind() -> StorageKind {
    StorageKind::Emmc { capacity_gib: EMMC_CAPACITY_GIB }
}
#[cfg(any(
    all(feature = "microsd", feature = "emmc"),
    all(not(feature = "microsd"), not(feature = "emmc"))
))]
compile_error!("select exactly one storage feature: `microsd` or `emmc`");

fn transports() -> Vec<DataTransport> {
    let mut t = Vec::new();
    // Wi-Fi Aware first (seamless) if built in; SoftAP is the universal baseline.
    #[cfg(feature = "wifi_aware")]
    t.push(DataTransport::WifiAware);
    #[cfg(feature = "softap")]
    t.push(DataTransport::SoftAp);
    #[cfg(feature = "wired_usbhs")]
    t.push(DataTransport::WiredUsbHs);
    // Guarantee at least SoftAP so a misconfigured build still has a data path.
    if t.is_empty() {
        t.push(DataTransport::SoftAp);
    }
    t
}
