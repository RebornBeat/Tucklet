# Tucklet — Final Architecture Review (pre-firmware)

This document consolidates and, where noted, **supersedes** the earlier ADRs after verifying current (2026) platform facts. Read this as the source of truth; the ADRs remain for history.

## 1. Connectivity — a layered strategy (supersedes ADR-002's two-plane model)

Three radios, used in layers. The app abstracts the transport so firmware and UX don't care which one moved the bytes.

| Layer | Radio | Role | Status today |
|---|---|---|---|
| Discovery + control + auth | **BLE** | Find the device, authenticate, battery %, free space, wake the data path | Works on every phone now |
| Primary data path (target) | **Wi-Fi Aware (NAN)** | Peer-to-peer WiFi-speed transfer, no access point, no "join network" prompt | iOS 26 has a WiFiAware framework; needs a CERTIFIED accessory + a SoC that supports NAN (see risks) |
| Fallback data path (guaranteed) | **WiFi SoftAP** | Device hosts a private AP; phone joins; files move over local HTTP | Works on every phone now; on iOS shows a one-time "join network?" prompt |

**Build order:** SoftAP first (universal, no certification). Add Wi-Fi Aware as the premium, prompt-free path once certified.

### Wi-Fi Aware — what it is and why it matters
Wi-Fi Aware / NAN is the "between BLE and WiFi" technology: BLE-style discovery (devices see each other by service name) with WiFi-class throughput, and no access point or internet required. It is the same class of tech AirDrop uses. It is the cleanest possible transport for this product.

### Risks to settle EARLY (do not assume)
1. **iOS requires a Wi-Fi Aware *certified* accessory.** Apple's `WiFiAware` framework (iOS 26) talks to certified accessories — not arbitrary NAN devices. Wi-Fi Alliance certification is a real cost/step before the clean iOS path works. SoftAP needs no certification.
2. **ESP32-C5 NAN support is new.** The ESP32-C5 is a newer chip. While Espressif lists Wi-Fi 6 features, NAN (Neighbor Awareness Networking) support must be validated on the current ESP-IDF for C5. If Wi-Fi Aware is must-have on day one, validate C5 NAN support early. SoftAP works on the C5 regardless.

## 2. iOS reality — can we do "everything"? (supersedes ADR-003)

Much closer to yes than six months ago, but three hard limits remain. None are solvable by "more engineering effort" — they are deliberate OS boundaries.

| Capability | iOS status |
|---|---|---|
| One-time pairing, seamless | **Yes** — AccessorySetupKit (iOS 18) gives "bring close, tap to pair." Full AirPods-grade *proximity* parity is currently **EU-only** (DMA); the base flow is worldwide. |
| High-speed transfer, no AP prompt | **Yes via Wi-Fi Aware** (certified). Otherwise SoftAP works everywhere with a one-time join prompt. |
| Files appear like local files | **Yes** — File Provider extension (shows in Files app). |
| Auto-connect after first pairing | **Yes** when the app is active/recently backgrounded (CoreBluetooth background modes). |
| Fully autonomous background sync, app never opened, phone locked for days | **No** — iOS hard-restricts arbitrary background networking. Android allows this; iOS does not. |

**Decision unchanged: Android-first, iOS strong-second.** The iOS experience in 2026 is genuinely good; just don't promise fully-silent always-on background sync on iOS.

## 3. Security + invisibility model (supersedes ADR-004's "press to summon")

The product must feel invisible: **one button press ever** (first-time pairing), then zero presses forever. This is the AirPods model and it is both more secure and better UX than press-every-time.

### Enrollment (first time, per phone)
1. User presses the button once -> device enters a short pairing window.
2. Device and phone exchange public keys (X25519/Ed25519). Phone's public key is stored in the device allow-list.
3. Pairing on iOS is presented via AccessorySetupKit ("bring close, tap").

### Daily use (every time after — no button)
1. Device does **low-duty BLE advertising** with a rotating/random address (not trackable). Average current is tens-to-hundreds of microamps, so a ~150 mAh cell lasts weeks; deep sleep between extends this.
2. The app sees it and silently connects.
3. **Challenge-response:** device sends a nonce; phone signs it with its private key; device verifies against the allow-list. No button, cannot be spoofed (attacker lacks the key), cannot be silently enrolled (new phones still need the physical press).
4. If a transfer is requested, the high-speed path (Wi-Fi Aware or SoftAP) is brought up on demand, then torn down.

### The button's only jobs
- First-time authorize (short press during pairing).
- Factory reset / wipe allow-list (long hold).

### Revocation
- App-side "forget this Tucklet" and device-side factory reset both clear trust. Single-use, short-TTL credentials for every SoftAP session mean a captured password is worthless later.

## 4. UX principles (the non-technical-user contract)
- Plain-language state only: **On phone / On Tucklet / Temporary**. Never the word "cache."
- **Temporary** = user picks the lifespan (1 hour / 1 day / 1 week / keep). The app enforces expiry.
- **Round-trip metadata:** files remember their origin (album/app) so "put it back" restores them exactly.
- **Per-app organization** by default (Camera, Screenshots, WhatsApp…), with an optional friendly folder view for power users — never forced.
- Real battery %, real free space, real transfer progress. No guessing, no mystery usage.

## 5. SoC decision — The "Charm Strategy"

The hardware strategy is now aligned exclusively around the **"Charm" form factor** (AirTag-class, < 35mm width). Oversized "Pro" or "Backpack" variants are excluded to preserve product identity.

### Production Hardware (Default)
**SoC:** **ESP32-C5** (RISC-V, Wi-Fi 6, BLE 5.0/5.4, Thread/Zigbee).
*   **Form Factors:**
    *   `ESP32-C5-WROOM-1` (Standard, 18 x 27.5mm).
    *   `ESP32-C5-MINI-1` (Compact, 15.4 x 21.3mm).
*   **Rationale:**
    *   **Dual-Band Wi-Fi 6:** Supports 5 GHz. This is the critical feature that doubles real-world throughput (~9 MB/s) compared to older 2.4 GHz chips (S3/C3), fulfilling the "invisible transfer" promise.
    *   **Form Factor:** The C5-MINI-1 enables the smallest possible "Charm" designs, fitting easily into the 32mm enclosure class.
    *   **Integration:** Integrated PCB antenna (WROOM/MINI) eliminates the reliability risks and assembly complexity of external IPEX cables/ceramic antennas found in M.2 modules.
    *   **Power:** Highly efficient, suitable for 120-200mAh batteries.
*   **Status:** **Production Ready.**

### Roadmap Hardware (Future/Projected)
**SoC:** **ESP32-E22** (Wi-Fi 6E, Tri-band, High Performance).
*   **Form Factor:** `ESP32-E22-MINI-1` (Projected/Planned).
*   **Rationale:** The E22 brings Wi-Fi 6E (6 GHz band) and extreme throughput (150+ Mbps theoretical), but requires significantly more power and heat management.
*   **Strategy:** We hold the **Design IP** for an E22-MINI variant. This design will be released as the "Super-Charm" **only when** Espressif releases the `ESP32-E22-MINI-1` module (with integrated antenna). Current E22 M.2 modules are rejected due to size/antenna constraints.

### Deprecated Hardware
*   **ESP32-S3:** Replaced by ESP32-C5. The S3 lacks 5 GHz Wi-Fi, which is a hard requirement for the Tucklet performance targets. It also lacks the mature Thread/Zigbee integration of the C5.
*   **ESP32-E22 M.2 (22x30mm):** Rejected. Requires external antenna and forces a "Puck/Backpack" form factor, which dilutes the "Charm" brand identity.

## 6. Open items before firmware
- [x] ~~Confirm Wi-Fi Aware target: SoftAP-only v1, or NAN from the start?~~ -> **Decision:** SoftAP is the guaranteed Layer 1 path. NAN is a stretch goal for iOS seamless UX.
- [x] ~~SoC Selection~~ -> **Decision:** ESP32-C5 (Production), ESP32-E22-MINI (Roadmap).
- [ ] Decide final name (working: **Tucklet**).
- [ ] Pick the launch capacities for the eMMC variant (see hardware/common/VARIANTS.md).
- [ ] Measure real transfer-current on a C5 dev board to size the battery (Peak TX current sizing).
- [ ] Budget Wi-Fi Alliance certification (only if pursuing the iOS Wi-Fi Aware path) and FCC/CE (always, due to radios).
