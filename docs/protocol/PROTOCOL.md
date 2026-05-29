# Tucklet Protocol (v0.1)

The single contract that keeps firmware and apps in sync. Two planes: **control** (BLE) and **data** (WiFi). Content is the same JSON across platforms.

## 1. Control plane — Bluetooth LE (GATT)

One custom GATT service. UUIDs below are placeholders — generate real 128-bit UUIDs before release.

| Characteristic | Props | Payload |
|---|---|---|
| `STATUS` | Read/Notify | `{ "battery": 0-100, "charging": bool, "free_mb": int, "total_mb": int, "card_present": bool, "fw": "x.y.z" }` |
| `AUTH` | Write/Notify | Pairing handshake (see §3) |
| `SESSION` | Write/Notify | Request a transfer session; device replies with one-time WiFi creds + token (see §4) |
| `COMMAND` | Write | Small commands: `{"op":"sleep"}`, `{"op":"factory_reset_confirm"}` |

BLE carries **only** small control messages. Never bulk data.

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

## 3. Pairing handshake (over `AUTH`)

1. Phone -> device: `{"phone_id": "<phone_pubkey>", "name": "Ana's iPhone"}`
2. Device: lights LED, waits for the physical button press (the authorization gesture).
3. On press: device stores `phone_id` in its allow-list, replies `{"paired": true}`.
4. On timeout/no press: `{"paired": false, "reason": "no_confirmation"}`.

Production should use a proper key exchange (e.g. X25519) so `phone_id` is a public key and later sessions are cryptographically authenticated, not just name-matched.

## 4. Session establishment (over `SESSION`)

1. Phone (already in allow-list) -> device: `{"op":"start"}`
2. Device spins up SoftAP, generates a single-use SSID/password + `session_token`, replies:
   `{ "ssid": "Tucklet-AB12", "psk": "<one-time>", "ip": "192.168.4.1", "token": "<session_token>", "ttl_s": 600 }`
3. Phone joins that WiFi (Android: programmatic; iOS: NEHotspotConfiguration) and uses the HTTP API with `X-Tucklet-Token`.
4. On completion/idle-timeout: device tears down SoftAP, invalidates `psk` and `token`.

Credentials are never static and never printed on the device.

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

## 7. Versioning

`fw` in STATUS and `/v1/` in the path are the version anchors. Breaking changes bump to `/v2/`; apps negotiate against `fw`.
