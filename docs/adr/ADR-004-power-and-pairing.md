# ADR-004: Power, SoC, and the button-gated pairing/auth model

**Status:** Accepted · **Date:** 2026-05

## SoC

**ESP32-S3** (module, e.g. ESP32-S3-WROOM-1 with PSRAM). Rationale:
- Has **both WiFi and BLE 5** on one cheap chip (~$2.5 as a module) — directly satisfies ADR-002.
- Mature Rust support via `esp-hal` (no_std) and `esp-idf-hal`/`esp-idf-svc` (std, with WiFi + filesystem + TLS already integrated). We use the **std / esp-idf** path because WiFi + SoftAP + an HTTP server + a filesystem are far less work there. (See firmware/ARCHITECTURE.md.)
- SDIO peripheral for fast microSD access.

## Power & battery

- **Battery:** small LiPo, ~120–200 mAh, sized so idle (BLE-advertise-on-demand only) lasts weeks and a realistic photo-transfer session is comfortable. Final size set after measuring real WiFi-transfer current draw on the prototype.
- **Charger / PMIC:** a USB-C Li-ion charger IC (e.g. an MCP73831-class or integrated PMIC) + battery fuel-gauge so the app can show a *real* percentage, not a guess.
- **Idle discipline:** the device is **fully silent** when not summoned — no background advertising, no scanning. This is both a battery and a security property.

## The pairing / auth model (the security heart of the product)

The device does nothing until physically touched. The flow:

1. **Asleep:** radios off. Nothing is discoverable. Battery sips microamps.
2. **Press button once → "summon":** device begins BLE advertising for a short window (e.g. 30 s) and lights the status LED.
3. **Phone sees it, requests connection.** First time only: the device shows it wants to pair; **the user presses the button again to authorize** that specific phone. The phone's identity (a public key) is stored in an allow-list. This is the "press again to auth" you described.
4. **Known phone, later:** a single press summons it; an already-authorized phone connects without a second press (configurable — you can require press-to-auth every time for higher security).
5. **Transfer:** over BLE the phone receives a **one-time WiFi credential + session token**; the device spins up its SoftAP; files move over WiFi; on completion or timeout the SoftAP is torn down and the credential is invalidated.
6. **Sleep:** after inactivity, everything powers down again.

### Why this is secure-by-default
- No always-on advertising means no passive tracking/enumeration of the device.
- Physical-press authorization for new devices means a remote attacker cannot pair without being physically present at the device.
- Per-session, single-use WiFi credentials mean a captured SoftAP password is worthless later.
- The allow-list can be cleared (factory reset) by a long button hold.

## Consequences
- One tactile button serves three roles: summon (short press), authorize (press during pairing), factory-reset (long hold). The app explains these; the hardware needs only one button + one RGB/status LED.
- Firmware needs a small persisted key store (the allow-list) in flash/NVS.
