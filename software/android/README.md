# Tucklet — Android app

The Android companion app (Kotlin + Jetpack Compose), speaking the same protocol
as the firmware (`docs/protocol/PROTOCOL.md`) and following the same UX
(`docs/UX_SPEC.md`). Android is the most seamless platform for this device: it can
join the charm's Wi-Fi programmatically and run real background backup.

## Build

```bash
cd software/android
# needs Android SDK 35 + JDK 17; open in Android Studio or:
./gradlew :app:assembleDebug      # (add the Gradle wrapper, or open in Studio)
```
`minSdk 29`, `targetSdk 35`, `compileSdk 35`.

## What the app does (full UX-spec coverage)

- **Onboarding / pairing** (`pairing/PairingManager.kt`, `ui/MainActivity.kt`) —
  CompanionDeviceManager shows a system chooser of nearby Tucklets (filtered by
  the BLE service UUID); one tap associates it (kept across launches, allows
  background BLE). Cryptographic enrollment then happens over BLE.
- **Home / Library / Settings** (`ui/Screens.kt`) — presence + battery + space;
  a unified library of **On phone** (MediaStore) and **On Tucklet** (manifest)
  items grouped by source app, with real thumbnails for both; multi-select with
  context-aware "Free up space" / "Get a copy"; item detail with restore / back
  up / delete; an Undo banner; honest "Forget this Tucklet".
- **Transfer** — estimated time shown up front (the verified estimator), keep-time
  picker for "Get a copy", and the actual export→upload / download→save work.
- **Programmatic Wi-Fi** (`wifi/WifiConnector.kt`) — joins the charm's one-time
  SoftAP via `WifiNetworkSpecifier` and pins HTTP to that `Network`. No trip to
  Settings (one-time system approval only) — the seamless path iOS can't match.
- **Real background trickle** (`trickle/TrickleWorker.kt`) — a periodic
  WorkManager job drips new photos to the charm when conditions are good, using
  the tested `Trickle.decide`. This genuinely runs in the background, unlike iOS.

## Source map

| Area | File |
|---|---|
| Wire types (mirror of `tucklet-proto`) | `protocol/Protocol.kt` |
| Estimator + trickle (mirror of `tucklet-core`) | `core/TransferEstimator.kt` |
| BLE control (GATT central) | `ble/BleControlClient.kt` |
| Programmatic Wi-Fi join | `wifi/WifiConnector.kt` |
| Data session + HTTP | `net/SessionManager.kt`, `net/DataClient.kt` |
| Photo library bridge (MediaStore) | `photos/PhotoSource.kt` |
| Pairing (CompanionDeviceManager) | `pairing/PairingManager.kt` |
| Background trickle (WorkManager) | `trickle/TrickleWorker.kt` |
| Orchestrator (shared by UI + worker) | `store/AppRepository.kt`, `store/TuckletGraph.kt` |
| ViewModel | `store/AppViewModel.kt` |
| UI (Compose) | `ui/Theme.kt`, `ui/MainActivity.kt`, `ui/Screens.kt` |

## Verified vs. needs-confirming

**Verified here** (compiled with kotlinc 2.0.0 and unit-tested):
- `core/TransferEstimator.kt` — estimator + trickle + link profiles: **14
  assertions** mirroring the Rust `tucklet-core` test values exactly (500 MB
  video 55–57 s; 30×4 MB photos 14–16 s; ETA; human labels; trickle batch 25/5;
  profile resolution).
- `protocol/Protocol.kt` — compiles with the serialization plugin and **round-
  trips the wire format**: snake_case keys, the flattened `state`/`expires_at`
  (matching the firmware's `#[serde(flatten)]`), and the `micro_sd` /
  `{"emmc":{"capacity_gib":N}}` `StorageKind` encoding.

**Written real, needs the Android SDK + device to confirm** (marked `CONFIRM`):
- The crypto seam (`store/AppRepository.kt` `Crypto`) — **implemented**: a
  BouncyCastle Ed25519 key is generated on first run and sealed in
  EncryptedSharedPreferences; its public key is enrolled at pairing and each
  session nonce is signed with it. Verified cross-stack: BouncyCastle produces
  byte-identical signatures to the firmware's ed25519-dalek for the same
  key+nonce, so a phone-signed challenge verifies on the device. CONFIRM on
  device: at-rest key storage (EncryptedSharedPreferences, or Keystore/StrongBox
  where Ed25519 is supported).
- GATT write types / notification timing (`ble/BleControlClient.kt`).
- CompanionDeviceManager result extras across API levels (`ui/MainActivity.kt`).
- `WifiNetworkSpecifier` approval behavior for per-session SSIDs (`wifi/`).
- MediaStore delete consent on Android 11+ (`photos/PhotoSource.kt`).

The GATT UUIDs match the firmware `ble.rs`.

## License
PolyForm Noncommercial 1.0.0 (see `/LICENSE-SOFTWARE.txt`).
