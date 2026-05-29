# Tucklet — iOS app

Complete Swift sources for the iOS companion app. Build with Xcode 26+ (iOS 26 deployment target).

## Generate the Xcode project
This repo ships an XcodeGen spec (text, reviewable) instead of a binary `.xcodeproj`:
```
brew install xcodegen
cd software/ios
xcodegen generate
open Tucklet.xcodeproj
```

## Targets
- **Tucklet** — the app (SwiftUI). BLE control, session/transport, transfers, UI.
- **TuckletFileProvider** — File Provider extension so Tucklet files appear in the Files app.

## Architecture
```
Tucklet/
  Models/Protocol.swift          Codable mirror of tucklet-proto (the Rust source of truth)
  Models/TransferEstimator.swift mirror of tucklet-core::estimate (+ link profiles)
  Connectivity/BLEControlClient  control plane (CoreBluetooth): status, auth, session
  Connectivity/SessionManager    picks transport, joins SoftAP / Wi-Fi Aware
  Connectivity/DataClient        data plane HTTP API (manifest/thumb/file/restore)
  Transfer/TransferEngine        batch transfer w/ live ETA + the trickle decision
  Store/AppModel                 app state orchestration
  Views/                         Home, Library, Transfer sheet (shows time up front), Settings
TuckletFileProvider/             replicated File Provider extension
```

## What is complete vs. what needs the live SDK
Complete and self-consistent: the model layer, the estimator (matches the tested Rust), the BLE control client (CoreBluetooth), the HTTP data client, the transfer engine + trickle logic, the full SwiftUI UX, and the File Provider structure.

Marked `CONFIRM` in code (must be verified against the iOS 26 SDK, which can't be compiled here):
1. **WiFiAware** framework calls in `SessionManager.joinWifiAware` — the seamless transport. The SoftAP path is complete and works without it.
2. **AccessorySetupKit** entitlement keys + accessory descriptors (Info.plist) for one-tap pairing.
3. **Secure Enclave** keypair + session-challenge signing in `Crypto` — the one security primitive to finish before shipping.
4. **File Provider domain registration** glue (`NSFileProviderManager.add(domain:)`) and wiring `TuckletStore` to the shared `DataClient` via the app group.

None of these are stubs-for-laziness; they are the exact points where a brand-new OS API's signature must be confirmed rather than guessed.

## License
PolyForm Noncommercial 1.0.0 (see /LICENSE-SOFTWARE.txt).
