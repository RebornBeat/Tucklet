# Tucklet — Desktop

Two parts:

1. **No app needed for wired use.** Plug the Tucklet in and it mounts as a plain
   USB Mass Storage drive — your file manager opens it like any disk. (That's the
   firmware's `wired_usbhs` path; nothing to install.)
2. **A wireless companion app** for backing up and pulling photos over the air,
   speaking the same `/v1` API as the firmware and the iOS/Android apps.

## Layout

```
desktop/
├── tucklet-desktop-core/   Rust lib: /v1 HTTP client, transfer engine, auth +
│                           BLE-discovery seams. Reuses tucklet-proto + tucklet-core.
├── tucklet-desktop-cli/    `tucklet` CLI (status/list/pull/push/estimate).
└── ui/                     Tauri v2 + React (CRA + Tailwind) GUI shell.
```

## Build & run

**Core + CLI** (no system libs needed):
```bash
cd software/desktop
cargo build
cargo test
# usage (host + token come from the BLE session handshake):
./target/debug/tucklet status   --host 192.168.4.1 --token <session-token>
./target/debug/tucklet list      --host 192.168.4.1 --token <token>
./target/debug/tucklet pull --id <id> --out photo.heic --host ... --token ...
./target/debug/tucklet push --file photo.heic --host ... --token ...
./target/debug/tucklet estimate  --host ... --token ...
```

Real BLE discovery (optional, needs the platform Bluetooth stack):
```bash
cargo build -p tucklet-desktop-core --features ble
```

**GUI** (Tauri + React):
```bash
cd software/desktop/ui
npm install
npm run tauri dev      # or: npm run tauri build
```

## Verified vs. needs-confirming

**Verified here** (compiled + tested with cargo 1.96, reusing the real shared
crates):
- `tucklet-desktop-core` — **16 tests**: the dependency-free HTTP/1.1 request
  builder + response parser; base64 (RFC 4648 vectors + round-trip); the `/v1`
  `DataClient` parsing a real flattened-state manifest fixture and round-tripping
  the base64 origin header; the transfer engine matching the core estimator
  (30×4 MB ≈ 15 s) and its batch/progress/stop-on-error behavior; profile pick.
- `tucklet-desktop-cli` — **3 tests** (arg parser, byte formatting, mime guess),
  and an **end-to-end socket smoke test**: the built `tucklet` binary fetched and
  printed a live manifest from a local HTTP server over a real TCP connection.

- **Integration test** (`tests/integration_device.rs`) — a mock Tucklet built
  from the *same shared crates the firmware uses* (allow-list + crypto + proto)
  is driven by the real client through enroll → nonce → sign → verify → token
  → manifest/push/pull/delete over real sockets, plus an impostor-key rejection.

**Written real, needs your environment to confirm** (marked `CONFIRM`):
- The crypto seam (`auth.rs`) — **implemented + tested**: a shared, host-tested
  `tucklet-crypto` (Ed25519) creates/loads the identity and signs the nonce; a
  unit test signs and verifies exactly as the firmware does. CONFIRM: the secret
  currently persists to a file under the config dir — move it to the OS keychain
  (macOS Keychain / Windows DPAPI / libsecret) before shipping.
- BLE discovery (`discovery.rs`, `--features ble`) — btleplug against the real
  adapter; service-UUID advertisement + OS Bluetooth permissions.
- The Tauri GUI (`ui/`) — not built in this sandbox (the `tauri` crate needs the
  platform webview + a Node toolchain). Its commands are thin wrappers over the
  verified core; build it with `npm run tauri`.

The BLE service UUID and `/v1` shapes match the firmware.

## License
PolyForm Noncommercial 1.0.0 (see `/LICENSE-SOFTWARE.txt`).
