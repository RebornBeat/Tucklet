# Tucklet Protocol (v0.1)

The single contract that keeps firmware and apps in sync. Two planes: **control** (BLE) and **data** (WiFi). Content is the same JSON across platforms.

## 1. Control plane — Bluetooth LE (GATT)

One custom GATT service. UUIDs below are placeholders — generate real 128-bit UUIDs before release.

| Characteristic | Props | Payload |
|---|---|---|
| `STATUS` | Read/Notify | `{ "battery": 0-100, "charging": bool, "free_mb": int, "total_mb": int, "card_present": bool, "fw": "x.y.z", "variant": "singlec5-wroom-microsd" \| ... }` |
| `AUTH` | Write/Notify | Pairing handshake (see §3) |
| `SESSION` | Write/Notify | Request a transfer session; device replies with one-time WiFi creds + token (see §4) |
| `COMMAND` | Write | Small commands: `{"op":"sleep"}`, `{"op":"factory_reset_confirm"}` |

BLE carries **only** small control messages. Never bulk data.

The `variant` field in STATUS allows the app to adjust its UI, transfer estimates, and feature visibility to match the connected device's exact hardware configuration without querying a separate endpoint. Variant identifiers follow the `gen_hardware.py` naming convention (e.g., `dualc5-mini-emmc`). See `docs/VARIANT_MATRIX.md` and `hardware/common/VARIANTS.md` for the complete variant taxonomy and `DeviceCapabilities` mapping.

## 2. Data plane — WiFi SoftAP + local HTTP

While a session is active the device hosts a SoftAP and a local HTTP/1.1 server at a fixed IP (e.g. `http://192.168.4.1`). All requests carry header `X-Tucklet-Token: <session_token>`.

| Method · Path | Purpose |
|---|---|
| `GET /v1/manifest` | List stored items as metadata only (id, name, size, type, origin, thumb_url). No file bodies. |
| `GET /v1/thumb/{id}` | Small thumbnail for gallery browsing. |
| `GET /v1/file/{id}` | Stream a full file (supports HTTP Range for resume). |
| `POST /v1/file` | Upload a file. Body includes the origin metadata (see §5). |
| `DELETE /v1/file/{id}` | Remove a file. |
| `POST /v1/restore/{id}` | Return origin metadata so the app can place the file back where it came from. |

**Virtualization principle:** the app browses `manifest` + `thumb` only. Full bytes move solely on explicit user action. This is how the gallery stays instant and the battery survives.

### Transport resolution

The app never hard-codes a transport. On connect it reads `DeviceCapabilities` (via the `variant` field and feature flags), then resolves the best available data path for the current platform and physical state. Wired USB-HS is always preferred when the device is plugged in; Wi-Fi Aware is preferred over SoftAP where both sides support it; SoftAP is the universal fallback. This logic is unit-tested in `tucklet-core::variant::usable_transports()` and shared across all three client platforms (iOS, Android, Desktop).

## 3. Pairing handshake (over `AUTH`)

1. Phone -> device: `{"phone_id": "<phone_pubkey>", "name": "Ana's iPhone"}`
2. Device: lights LED, waits for the physical button press (the authorization gesture).
3. On press: device stores `phone_id` in its allow-list, replies `{"paired": true}`.
4. On timeout/no press: `{"paired": false, "reason": "no_confirmation"}`.

Production uses a proper key exchange (X25519/Ed25519) so `phone_id` is a public key and later sessions are cryptographically authenticated, not just name-matched. See `docs/protocol/AUTH.md` for the full Ed25519 challenge-response contract, the cross-verified reference vector, and the per-platform secure storage notes.

## 4. Session establishment (over `SESSION`)

1. Phone (already in allow-list) -> device: `{"op":"start"}`
2. Device spins up SoftAP, generates a single-use SSID/password + `session_token`, replies:
   `{ "ssid": "Tucklet-AB12", "psk": "<one-time>", "ip": "192.168.4.1", "token": "<session_token>", "ttl_s": 600 }`
3. Phone joins that WiFi (Android: programmatic via `CompanionDeviceManager` + `WifiNetworkSuggestion` or `NetworkRequest`; iOS: `NEHotspotConfiguration` or Wi-Fi Aware `WiFiAwareSession` where certified) and uses the HTTP API with `X-Tucklet-Token`.
4. On completion/idle-timeout: device tears down SoftAP, invalidates `psk` and `token`.

Credentials are never static and never printed on the device. Each session uses fresh, single-use, short-TTL credentials.

## 5. Round-trip / origin metadata (the "put it back where it was" feature)

Every uploaded item stores where it came from:

```json
{
  "id": "itm_01H...",
  "name": "IMG_2087.HEIC",
  "size": 4382002,
  "type": "image/heic",
  "created_at": "2026-04-02T18:11:09Z",
  "origin": {
    "platform": "android|ios",
    "app": "Camera",
    "collection": "DCIM/Camera",
    "album": "Camara",
    "device_name": "Ana's phone"
  },
  "state": "on_tucklet"
}
```

On `POST /v1/restore/{id}` the device returns the `origin` block; the app re-inserts the file into the correct album/app location (Android and iOS differ here; firmware stays platform-neutral).

## 6. State vocabulary (UX contract — no jargon)

Three states only. Never expose the word "cache":

| Protocol value | App label | Meaning |
|---|---|---|
| `on_phone` | **On phone** | Lives on the phone only |
| `on_tucklet` | **On Tucklet** | Lives on the device only |
| `temporary` | **Temporary** | A copy pulled to the phone for a user-chosen duration; auto-removed when it expires |

For `temporary`, the app stores a user-chosen expiry (1 hour / 1 day / 1 week / keep). Expiry enforcement is the app's job; firmware only holds the canonical copy.

## 7. Variant-aware transfer estimates

The device's variant determines its wired and wireless throughput. The app uses the `variant` field from STATUS to look up the correct `LinkProfile` (`tucklet-core::link::profile_for`) so the transfer-time estimate reflects *this* unit's real speed before any bytes move.

| Variant class | Wireless (SoftAP) | Wired (GL3224 USB 3.0) | Wired (GL823 USB 2.0) |
|---|---|---|---|
| Single C5 | ~9 MB/s | ~70–100+ MB/s | ~25–35 MB/s |
| Dual C5 | ~15 MB/s | ~70–100+ MB/s | ~25–35 MB/s |
| Single E22-MINI (roadmap) | ~150 MB/s (projected) | ~70–100+ MB/s | ~25–35 MB/s |
| Dual E22-MINI (roadmap) | ~300 MB/s (projected) | ~70–100+ MB/s | ~25–35 MB/s |

The wired speed depends on which USB bridge (U2) is populated. The GL3224 (QFN-32, USB 3.0, ~70–100+ MB/s) is the primary and standard build path; the GL823 (QFN-24, USB 2.0, ~25–35 MB/s) is the size-constrained backup. The device reports its bridge capability as part of `DeviceCapabilities` so the app shows the honest wired speed. See `hardware/common/COMPONENT_SELECTION.md` for the full USB bridge alternatives matrix (GL3224, PL2732, GL823, AU6601, EZ-USB SD3).

### Pro Charm variants

Pro variants (ESP32-E22-WROOM + 2S LiPo, ~35×28×12–13 mm envelope) use the same ESP32-E22 radio as the E22-MINI Charm variants and therefore share the same wireless throughput estimates. The wired path is also identical (GL3224 USB 3.0 bridge). The Pro line is fully covered under the same source-available, non-commercial license as the Charm line (CC BY-NC-SA 4.0 for hardware, PolyForm Noncommercial 1.0.0 for software). See `hardware/PRO_LINE.md` for the Pro variant matrix and power tree.

## 8. Versioning

`fw` in STATUS and `/v1/` in the path are the version anchors. Breaking changes bump to `/v2/`; apps negotiate against `fw`.
