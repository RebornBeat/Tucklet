# tucklet-fw — on-device firmware (ESP32-C5)

The firmware binary that runs on the Tucklet device. It is thin glue over the
two host-tested crates: `tucklet-proto` (wire types) and `tucklet-core` (all the
decision logic — estimator, state machine, trickle, session credentials,
allow-list, expiry, transport resolution).

## Build

This crate uses the Espressif Rust toolchain (it is intentionally its own
workspace, independent of the host workspace that builds proto/core on your PC):

```bash
cargo install espup ldproxy
espup install
. $HOME/export-esp.sh          # source the esp env

cd firmware/crates/tucklet-fw
# pick exactly one radio_* and one storage_* feature; transports are additive
cargo build --release --features singlec5,microsd,softap
# flash + monitor (espflash is the configured runner)
cargo run  --release --features singlec5,microsd,softap
```

Variant feature combinations mirror `docs/VARIANT_MATRIX.md`:
`singlec5|dualc5` × `microsd|emmc` × `softap [+ wifi_aware] [+ wired_usbhs]`.
`board::capabilities()` turns the enabled features into the runtime
`DeviceCapabilities` the app reads over BLE — so one firmware source serves every
board, and a build that targets the wrong combination fails at compile time
(`compile_error!` guards).

## Module map

| Module | Role |
|---|---|
| `main.rs` | two-phase init + the event loop (button, status, action queue) |
| `board.rs` | variant → `DeviceCapabilities`; the signal→GPIO pin map |
| `app.rs` | shared state; BLE message handlers; ties core logic to the glue |
| `ble.rs` | NimBLE GATT server: CAPS / STATUS / AUTH / SESSION / COMMAND |
| `wifi.rs` | SoftAP up/down per session; Wi-Fi Aware path (feature-gated) |
| `httpd.rs` | the `/v1/*` data API with per-request token auth |
| `storage.rs` | SDIO mount, manifest, file I/O, round-trip origin metadata |
| `power.rs` | MAX17048 fuel gauge, charge state, sleep policy |
| `ui.rs` | button press-pattern decode, WS2812 status colors |
| `auth.rs` | NVS-persisted allow-list + Ed25519 challenge verification + device key |

## What is verified vs. what needs bring-up

**Verified here (compiled + unit-tested on the host):**
- All of `tucklet-core` incl. the `session` credential module — **16 tests**.
- `tucklet-proto` wire types — **5 tests**.
- `ui.rs` button/LED logic — **3 tests** (compiled against the real core).
- `httpd.rs` request parsing (range / path / base64) — **3 tests**.

**Written real, but needs on-hardware / SDK confirmation (cannot be CI-built
here against the esp toolchain):**
- The esp-idf glue itself (BLE, Wi-Fi, HTTP server, NVS, I2C, GPIO).
- `CONFIRM` flags mark the genuinely version/silicon-specific spots:
  - the SDMMC slot/width/pin config for the **ESP32-C5** (`storage::mount`),
  - the exact **esp32-nimble** callback signatures for your pinned crate version,
  - **Wi-Fi Aware / NAN** support on the C5 (only if you enable `wifi_aware`),
  - the WS2812 RMT timing and the deep-sleep wake source.

These are bring-up tasks, not design unknowns. The architecture, the protocol,
the state machine, and the decision logic are settled and tested; the remaining
work is making real silicon agree with the real SDK.

## License
PolyForm Noncommercial 1.0.0 (see ../../../LICENSE-SOFTWARE.txt).
