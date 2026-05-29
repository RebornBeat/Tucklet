# ADR-002: Connectivity — BLE control plane + WiFi data plane (NOT Bluetooth-only)

**Status:** Accepted · **Date:** 2026-05

## Context

The original product vision was "no WiFi, just Bluetooth / AirDrop." This decision record exists because that vision, taken literally, is physically impossible, and the whole product depends on understanding why.

## The hard constraint

Bluetooth throughput is far too low for storage:

| Link | Realistic throughput | Time for one 500 MB video |
|---|---|---|
| Bluetooth LE 5.x | ~0.1–0.25 MB/s | ~35–80 minutes |
| Bluetooth Classic | ~0.2–0.3 MB/s | ~25–40 minutes |
| WiFi (ESP32 SoftAP, TCP) | ~2.5–5 MB/s | ~100–200 seconds |
| USB-C (USB 2.0 HS wired) | ~20–40 MB/s | ~15–25 seconds |

"AirDrop" is not a Bluetooth feature. AirDrop uses Bluetooth only to *discover* the peer, then transfers the files over **WiFi Direct**. So "AirDrop tech" already proves the rule: control over BLE, bulk data over WiFi. It is also Apple-proprietary and unavailable to third-party hardware.

## Decision

- **Control plane = Bluetooth LE.** Discovery, the button-press authorization handshake, battery %, free-space query, wake/sleep, and small commands. Cheap, low-power, always-on-demand.
- **Data plane = WiFi SoftAP.** The device hosts a private WiFi access point **only while a transfer session is active**, then tears it down to save power. Files move over a local HTTP/socket API on that AP.
- **Wired fallback = USB-C.** For users who want maximum speed or who are charging anyway, USB-C carries data too.

This preserves everything the user actually wanted: no permanent data cable, button-gated security, low idle power. The only correction is that the *data radio* must be WiFi-class.

## Consequences

- The SoC must have **both BLE and WiFi**. This is exactly why the ESP32-S3 was chosen (see ADR-004 / hardware).
- WiFi SoftAP draws meaningful current; it must be **off by default** and only spun up for transfer sessions, driven by BLE commands. Battery sizing accounts for transfer-time WiFi draw, not continuous.
- The phone must join the device's SoftAP. On Android this is programmatic; on iOS it requires NEHotspotConfiguration (see ADR-003).
- Security spans both planes: BLE authorizes the session and hands the phone a one-time WiFi credential + session token (see PROTOCOL.md). The SoftAP password is never static or printed on the device.
