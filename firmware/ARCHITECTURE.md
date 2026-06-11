# Firmware Architecture (Rust, ESP32-C5)

> **Scope:** This document covers the firmware architecture for the **ESP32-C5** production line (WROOM and MINI form factors, single and dual radio). The ESP32-E22 (Roadmap) and Pro (E22-WROOM + 2S) lines are documented separately in `hardware/PRO_LINE.md`. Their architecture will extend this foundation when the hardware is released.

## Toolchain choice: esp-idf (std) Rust

Two Rust paths exist for ESP32-C5:
- `esp-hal` — bare-metal `no_std`. Maximum control, but you implement/port WiFi, TCP, TLS, FAT/exFAT, HTTP server yourself. Heavy lift.
- **`esp-idf-svc` / `esp-idf-hal` (std)** — Rust on top of Espressif's IDF. WiFi, SoftAP, an HTTP server, a FAT/SD layer, and NVS key storage are already available. **We use this** — it gets a working device to "files move over WiFi" in a fraction of the time, which matters for a first product.

## Crate layout (Cargo workspace)

```
firmware/
├── Cargo.toml                # workspace
└── crates/
    ├── tucklet-fw/            # the binary that runs on the device
    ├── tucklet-proto/         # shared protocol types (serde structs from PROTOCOL.md)
    └── tucklet-core/          # device logic: state machine, allow-list, sessions
```

`tucklet-proto` is `no_std + serde` so the **same types can be reused by the apps** (via a thin codegen/FFI or by mirroring), keeping firmware and app in lockstep with PROTOCOL.md.

## Runtime state machine

```
            press (short)
  ASLEEP ─────────────────▶ ADVERTISING ──(authorized phone connects)──▶ CONNECTED
    ▲                            │                                          │
    │ inactivity timeout         │ timeout / no auth                        │ session start
    └────────────────────────────┴──────────────────────────────────◀──────┤
                                                                            ▼
                                                                     TRANSFERRING (SoftAP up)
                                                                            │ done / idle
                                                                            ▼
                                                                        CONNECTED ──▶ ASLEEP
```

- **ASLEEP:** radios off, deep sleep, button wake configured.
- **ADVERTISING:** BLE only, short window, LED breathing. New phones require the physical press to enter the allow-list.
- **CONNECTED:** BLE link up, STATUS notifications flowing (battery/free space). No WiFi.
- **TRANSFERRING:** SoftAP + HTTP server up, single-use creds issued, files move. Torn down on idle/complete.

## Modules (in tucklet-core)

- `state` — the machine above.
- `auth` — allow-list in NVS, pairing handshake, X25519 key exchange (production).
- `session` — single-use SoftAP credential + token generation/expiry.
- `storage` — exFAT over SDIO; manifest build; origin-metadata sidecar (`.tucklet.json` per item or a single index).
- `power` — fuel-gauge read, charge state, sleep policy.
- `ui` — button press-pattern decode (short/hold), LED state mapping.
- `agg` — **SPI Master/Slave** driver for the dual-radio aggregation link (dual-C5 variants only).

## Dual-Radio AGG Link — SPI Architecture

The dual ESP32-C5 variants use a dedicated **SPI bus** for inter-processor communication, replacing the legacy UART approach. SPI unlocks 12–15 MB/s sustained aggregated throughput, a significant improvement over the ~8–10 MB/s UART bottleneck.

### SPI AGG Link Pins

| Signal | Direction (U1 → U1B) | Notes |
|---|---|---|
| `AGG_CLK` | U1 output → U1B input | SPI clock, driven by primary radio |
| `AGG_CS` | U1 output → U1B input | Chip select, active low |
| `AGG_MOSI` | U1 output → U1B input | Data from primary to secondary |
| `AGG_MISO` | U1 input ← U1B output | Data from secondary to primary |

U1 (primary) is the SPI master; U1B (secondary) is the SPI slave. The firmware on U1 coordinates link aggregation — striping data across both Wi-Fi radios for higher combined throughput.

### Firmware Responsibilities

1. **Initialization:** On dual variants, U1 initializes the SPI master, U1B initializes the SPI slave.
2. **Synchronization:** U1 asserts `AGG_CS` to start a transaction, clocks data on `AGG_MOSI`, and reads `AGG_MISO`.
3. **Packet format:** The AGG link uses a simple framed protocol:
   - Header: `[0xAA, 0x55, cmd, len_hi, len_lo]`
   - Payload: `[data...]`
   - Footer: `[checksum]`
4. **Throughput:** Target is 12–15 MB/s sustained. This is limited by SPI clock speed and the secondary radio's ability to keep up with the data stream.

### Why SPI over UART

| Attribute | UART | SPI |
|---|---|---|
| Wires | 2 (TX, RX) | 4 (CLK, CS, MOSI, MISO) |
| Max throughput | ~8–10 MB/s | ~12–15 MB/s |
| Flow control | Software (XON/XOFF) | Hardware (CS) |
| Latency | Higher | Lower |
| Pin count | 2 | 4 |

The 4-wire SPI link is justified by the 50–87% throughput improvement. The additional 2 pins are available on the ESP32-C5 in both WROOM and MINI form factors.

## Variant handling

Compile-time Cargo features select the hardware reality; everything else is runtime:

```
[features]
# Chip selection
chip_c5 = []
chip_e22 = []

# Form Factor
form_wroom = []
form_mini  = []

# Radio Count (derived or explicit)
radio_single = []
radio_dual   = []

# Storage
storage_microsd = []
storage_emmc    = []

# Transport
transport_softap = []        # always on
transport_wifi_aware = []    # optional
transport_wired_usbhs = []   # optional (the bridge IC is populated)

# AGG Link
agg_spi = []                 # dual variants use SPI; single doesn't need it
```

At boot the firmware assembles a `DeviceCapabilities` from its enabled features and advertises it. `tucklet-core::variant::usable_transports()` (unit-tested) then narrows that to what the connected client can use.

## Storage ownership — the shared-bus architecture

The SDIO storage bus is shared between the radio (wireless path) and the USB-HS bridge (wired path) via a hardware mux (U6). The firmware controls this mux via the `SD_SEL` GPIO:

- **SD_SEL = LOW** (default): Radio (U1) owns the storage. Firmware reads/writes via SDIO.
- **SD_SEL = HIGH** (bridge owns): USB-HS bridge (U2) owns the storage. The bridge appears as a USB mass storage device when plugged in.

**Firmware rule:** Never access the storage bus when `SD_SEL` is HIGH (USB plugged in). The bridge is autonomous and the firmware must not interfere. Similarly, the bridge must not access the bus when `SD_SEL` is LOW (wireless mode).

### Mux timing

1. **USB plug-in detected** (VBUS present, or firmware detects USB connect event).
2. Firmware flushes any pending writes, unmounts the filesystem, and drives `SD_SEL` HIGH.
3. Bridge takes ownership. Computer sees a USB mass storage device.
4. **USB unplug detected** (VBUS falls).
5. Bridge releases the bus. Firmware drives `SD_SEL` LOW.
6. Firmware re-initializes SDIO, remounts the filesystem.

This handoff is seamless from the user's perspective: unplug the USB cable and the charm is immediately available for wireless transfers again.

### Per-variant mux considerations

| Variant | Mux | Notes |
|---|---|---|
| **All single C5** | TS3A-class (primary) | Standard SDIO mux. Validate bandwidth on prototype. |
| **All single C5** | TMUX1574 (alternative) | Upgrade path if SDIO timing issues observed. |
| **All single C5** | TXS02612 (alternative) | SD-specific with level shifting. |
| **All dual C5** | TS3A-class (primary) | Same mux as single, but firmware must also coordinate AGG link state. |
| **All dual C5** | TMUX1574 (alternative) | Recommended upgrade for dual variants due to higher SDIO clock rates. |

## Security notes

- SoftAP credentials and session tokens are single-use and short-TTL (PROTOCOL §4).
- Allow-list entries are phone public keys; clear via long-hold factory reset.
- No static secrets, nothing printed on the device.

## Power management

### Charge + transfer simultaneous operation

The BQ25896 (primary charger) provides a **power-path** output (`SYS` pin) that powers the system directly from USB while the battery charges. This means:

1. **USB plugged in, transferring:** System runs from USB power via `SYS`. Battery charges in parallel. Radio can operate at full power.
2. **USB plugged in, idle:** System runs from USB power. Battery charges.
3. **USB unplugged:** System runs from battery via buck regulator (U5).

The MCP73831 (backup charger) does **not** have power-path. If this alternative is used, the system runs from battery even when USB is plugged in. For single-C5 variants where simultaneous charge+transfer is rare, this is acceptable. For dual-C5 variants, the BQ25896 is strongly recommended.

### Charge current sizing

The BQ25896 sets charge current via I2C (or PROG resistor fallback). For a 120–200 mAh LiPo:
- **Charge current:** 500 mA (safe for small cells, charges in ~20–40 min).
- **Input current limit:** 900 mA (allows headroom for system load while charging).

The MCP73831 sets charge current via PROG resistor only (no I2C). R3 = 2 kΩ sets ~500 mA.

### I2C bus sharing

The BQ25896 shares the I2C bus with the MAX17048 (fuel gauge). Both are I2C slaves with different addresses. The firmware uses a shared I2C bus driver and addresses each device independently. Only one set of pull-ups (R4, R5) is needed.

## Honest status

This is a **scaffold**: the structure, types, and state machine are real and correct, but the device-specific bring-up (SDIO timing, WiFi current tuning, exFAT integration, real key exchange) must be done and tested on actual hardware. It is not a flashed, validated binary.

The following items require on-hardware validation:
- SDIO timing with the TS3A-class mux (upgrade to TMUX1574 if issues)
- GL3224-OEM bridge handoff timing (USB plug/unplug transitions)
- BQ25896 I2C control and charge current calibration for the specific cell
- Dual-radio SPI AGG link throughput and RF isolation
- Thermal management under sustained transfer (especially dual variants)
- exFAT filesystem integrity during power loss mid-transfer
