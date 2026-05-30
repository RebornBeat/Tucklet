# Tucklet — iOS app

The iOS companion app. Calm, keepsake-feel UI (see `docs/UX_SPEC.md`), built to
the same protocol the firmware speaks (`docs/protocol/PROTOCOL.md`). Build with
Xcode 26+ (iOS 26 deployment target).

## Generate the Xcode project

```bash
brew install xcodegen
cd software/ios
xcodegen generate
open Tucklet.xcodeproj
```

## What the app does (full UX-spec coverage)

- **Onboarding / pairing** (`Sources/Pairing.swift`) — one-tap AccessorySetupKit
  flow ("bring close, tap to connect"), shown until paired. After that the charm
  is invisible: silent BLE reconnect + a cryptographic challenge-response.
- **Home** (`Views/Screens.swift`) — presence, battery, free/total space, a
  plain-language backup summary, and an Undo banner after a backup.
- **Library** — one unified list of **On phone** (camera roll via PhotoKit) and
  **On Tucklet** (device manifest) items, grouped by source app, with real
  thumbnails for both. Multi-select; "Free up space" and "Get a copy" actions
  appear based on what's selected.
- **Item detail** — large thumbnail, plain state ("On phone" / "On Tucklet" /
  "Temporary"), origin, and the right action: back up, put back on phone
  (round-trip restore to the origin album), or delete.
- **Transfer sheet** — the honest **estimated time shown up front**, Now vs
  Automatically, and for "Get a copy" a keep-time picker (defaults to the
  Settings value). Live ETA recomputed from measured throughput.
- **Trickle backup** (`Sources/TrickleScheduler.swift`) — a `BGProcessingTask`
  drips new photos to the charm in the background, plus foreground auto mode.
- **Settings** — auto-backup + only-while-charging toggles, default keep time,
  device storage/battery/firmware, Undo last backup, and "Forget this Tucklet"
  (with an honest note that fully erasing this phone from the charm needs a
  device factory reset).

## Source map

| Area | Files |
|---|---|
| Wire types (mirror of `tucklet-proto`) | `Models/Protocol.swift` |
| Estimator (mirror of `tucklet-core`) | `Models/TransferEstimator.swift` |
| BLE control | `Connectivity/BLEControlClient.swift` |
| Data session + HTTP | `Connectivity/SessionManager.swift`, `Connectivity/DataClient.swift` |
| Photo library bridge | `Sources/PhotoKitSource.swift` |
| Pairing / onboarding | `Sources/Pairing.swift` |
| Background trickle | `Sources/TrickleScheduler.swift` |
| Orchestrator | `Store/AppModel.swift` |
| Transfers | `Transfer/TransferEngine.swift` |
| UI | `Views/TuckletApp.swift`, `Views/Screens.swift` |
| Files app integration | `TuckletFileProvider/FileProviderExtension.swift` |

## Honesty: what needs confirming on-device

This app is written real against the actual SDKs but cannot be compiled on this
machine (no Apple toolchain). The pure transfer-time logic is verified in
`tucklet-core` (Rust, tested). The spots to confirm in Xcode against the iOS 26
SDK are marked `CONFIRM` in-source:

1. **Crypto seam — implemented.** A CryptoKit `Curve25519.Signing` (Ed25519)
   key is created on first run and sealed in the Keychain
   (`kSecAttrAccessibleWhenUnlockedThisDeviceOnly`); its public key is enrolled
   at pairing and each session nonce is signed with it. This is the same RFC 8032
   standard the firmware verifies — cross-checked: BouncyCastle and ed25519-dalek
   produce byte-identical signatures, and CryptoKit follows the same deterministic
   spec. CONFIRM on device: Keychain accessibility/entitlement and that the signed
   nonce verifies against the enrolled key end-to-end.
2. **AccessorySetupKit** (`Pairing.swift`) — exact `ASDiscoveryDescriptor`
   property names and picker-completion shape on the iOS 26 SDK.
3. **Wi-Fi Aware** entitlement/usage keys (`project.yml`) — confirm against the
   iOS 26 `WiFiAware` framework; SoftAP is the guaranteed fallback.
4. **Background limits** — iOS won't run unbounded background networking; trickle
   is best-effort (system-scheduled) plus foreground auto. Android does this more
   freely (next platform).

The BLE GATT UUIDs in `BLEControlClient.swift` match the firmware `ble.rs`
service/characteristic UUIDs (SVC `F0CC0001…`).

## License
PolyForm Noncommercial 1.0.0 (see `/LICENSE-SOFTWARE.txt`).
