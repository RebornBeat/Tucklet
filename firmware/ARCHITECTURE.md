# Firmware Architecture (Rust, ESP32-S3)

## Toolchain choice: esp-idf (std) Rust

Two Rust paths exist for ESP32-S3:
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
- `auth` — allow-list in NVS, pairing handshake, (production) X25519 key exchange.
- `session` — single-use SoftAP credential + token generation/expiry.
- `storage` — exFAT over SDIO; manifest build; origin-metadata sidecar (`.tucklet.json` per item or a single index).
- `power` — fuel-gauge read, charge state, sleep policy.
- `ui` — button press-pattern decode (short/hold), LED state mapping.

## Security notes
- SoftAP credentials and session tokens are single-use and short-TTL (PROTOCOL §4).
- Allow-list entries are phone public keys; clear via long-hold factory reset.
- No static secrets, nothing printed on the device.

## Honest status
This is a **scaffold**: the structure, types, and state machine are real and correct, but the device-specific bring-up (SDIO timing, WiFi current tuning, exFAT integration, real key exchange) must be done and tested on actual hardware. It is not a flashed, validated binary.
