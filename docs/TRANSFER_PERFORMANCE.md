# Transfer Performance & Layer-1 Wireless Architecture

This is the document that decides whether the product is good. Most wireless drives in history died on slow transfers + clunky apps (SanDisk Connect, Gnarbox, WD My Passport Wireless, the Seagate units — a graveyard of discontinued products). Tucklet targets exactly that weak spot, so the speed story has to be honest and engineered, not hoped for.

## Units (the thing that confused everyone)
Networks are quoted in mega**bits** (Mbps); files in mega**bytes** (MB/s). Divide Mbps by 8 to get MB/s. "30–70 Mbps" = "~4–9 MB/s". This doc uses **MB/s** because that's what a user feels.

## The two layers
- **Layer 1 — wireless data path.** The everyday path. Must be invisible and low-power. This is the focus.
- **Layer 2 — wired fast path.** A plug-in fallback for bulk. Optional, but it fits the dimensions (see below), so it stays in.

## The hard ceiling of the ESP32 family
Espressif's own number is "up to 20 Mbit/s TCP, 30 Mbit/s UDP over the air"; measured real-world results scatter from ~7–22 Mbit/s down to far less. Most ESP chips are single-stream (1x1), 20–40 MHz radio, physically capped around ~9 MB/s. The ESP32-S3's USB is Full-Speed only (12 Mbps ceiling) and its own "wireless disk" demo measured 540 KB/s read / 350 KB/s write — so the ESP chip itself can NEVER be the fast wired path.

## Layer-1 options (verified spectrum)

| Option | Class | Real throughput | Size | ~Cost (radio) | Battery | Fits charm? |
|---|---|---|---|---|---|---|
| ESP32-S3 | MCU | ~2–5 MB/s, 2.4GHz only | 18×25.5mm | ~$2.75 | weeks | Yes (but slow) |
| **ESP32-C5** | MCU | **~4–9 MB/s**, +5GHz WiFi6 | ~18×27.5mm | ~$2–3 | weeks | **Yes — CURRENT STANDARD** |
| **ESP32-E22-MINI-1** | MCU (Hi-Perf) | **~20–40+ MB/s** (Projected) | ~15×21mm (Projected) | ~$4–5 | days | **Yes — ROADMAP** |
| Dual ESP32-C5 (aggregation) | MCU x2, experimental | ~13–16 MB/s | 2 modules + 2 ant | ~$5–6 | days | Tight but plausible |
| MT7628 + 802.11ac + Linux | Router SoC | ~6–18+ MB/s dual-band | SoC+DDR+flash | ~$10–20 | big cell | No (FileHub-sized) |
| 2x2 ac/ax module + Linux SoM | App processor | ~20–50+ MB/s | larger | ~$15–30+ | big cell | No |

**Conclusion:** The market bifurcates. You either accept the physical limits of a "Charm" sized device, or you build a brick. Tucklet chooses the Charm. The **ESP32-C5** is the balance for today. The **ESP32-E22-MINI-1** is the roadmap for a "Super-Charm" that maintains the form factor but breaks the throughput ceiling.

### Decision: ESP32-C5 (Standard) & ESP32-E22-MINI-1 (Roadmap)
**Standard (C5):** Dual-band Wi-Fi 6 (adds 5GHz, less congested), ~2x the S3's wireless, BLE retained for control, charm-sized, low power, in mass production.
**Roadmap (E22):** Wi-Fi 6E (Tri-band) offers the potential for significant throughput gains (2x2 MIMO, 160MHz channels) inside a projected Mini form factor. This allows a "Pro Charm" variant without breaking the dimensions. Note: The E22 requires significantly more power (larger battery) and thermal management.

## The form factor is a hidden Layer-1 advantage
Because the charm sits ON the phone, the radio link is centimeters long → near-perfect SNR → the radio sustains its TOP modulation rate → you reliably land at the HIGH end (~9 MB/s for C5, higher for E22), not the low end (~4). Competing drives sit across the room and suffer; Tucklet never does. The UX form factor also maximizes the cheap radio's throughput.

**The "Charm Advantage":**
This proximity advantage is our primary defense against obsolescence. While a FileHub across the room struggles with interference, Tucklet's "on-phone" position gives it a pristine RF environment. This means a C5 in a Charm often outperforms a faster chip in a brick, simply due to link budget.

## Experimental Layer-1 boosters (unproven, flagged honestly)
1. **Dual-C5 link aggregation** (app-layer striping or MPTCP): ~13–16 MB/s. Hard part = RF isolation of two 5GHz antennas in ~32mm (orthogonal placement + polarization mitigates; not guaranteed until prototyped). Best "beyond C5" lever that fits the current chip availability. Treat as v2.
2. **ESP32-E22 Upgrade:** Swapping the C5 for an E22-MINI (when available) is the most direct path to speed. It fits the same footprint philosophy (Mini) but moves to 2x2 Wi-Fi 6E. This is the preferred "v2" hardware upgrade path over aggregation complexity.
3. **Magnetic snap-on contact pins** (MagSafe-style): full wired speed with snap-on feel. Catch: phones expose no back-side data pins, only the charge port — needs a companion case or snap-over-USB-C. Accessory path.
4. **60 GHz / 802.11ad**: gigabits at cm range (ideal for a charm), but no phone exposes it to third parties and modules are power-hungry/costly. Dream, not buildable now.
5. **Trickle-sync (THE winning lever, not hardware):** continuously back up new photos whenever the charm is near the phone and idle/charging, so the big transfer NEVER happens and raw throughput stops mattering for the everyday case. Cheap radio + always-present + patient background sync beats a fast radio used occasionally. Strong on Android; partial on iOS (background-networking limits). This is the design philosophy that makes a 9 MB/s charm feel better than an 18 MB/s brick.

## Layer 2 — wired fast path (kept, because it fits)
A dedicated USB 2.0 High-Speed card-reader/bridge IC (~$1–2, ~5mm QFN) owns the microSD/eMMC when plugged in, bypassing the ESP entirely → ~20–40 MB/s. Storage is shared by mode arbitration: wireless mode = ESP owns the card; plugged-in mode = bridge owns it; never both at once. On desktop this also makes the device a plain USB drive (easiest platform). Negligible size impact.

## Dimensions — reconfirmed for all options
- **C5 + USB-HS bridge:** bridge is negligible; comfortably inside the AirTag envelope (~32–35mm). Confirmed.
- **E22-MINI (Roadmap):** Projected Mini dimensions (~15x21mm) fit the Charm envelope perfectly. Requires larger battery (thickness increase) but keeps the "clip-on" identity.
- **Dual-C5:** board area fits; constraint is two-antenna isolation in 32mm + slightly bigger battery. Borderline-plausible — validate on prototype before finalizing enclosure.
- **Linux router SoC:** DDR + SoC + flash + bigger battery exceed the charm envelope → deck-of-cards size. Only if speed is chosen over form factor. **Rejected.**
- Battery is the thickness driver in every case, not the logic.

## Headline numbers to design and market around
- **Standard Wireless (C5, on-phone):** **~9 MB/s** — fine for photos/short clips, invisible via trickle-sync.
- **Roadmap Wireless (E22, on-phone):** **~20–40 MB/s** — High-performance transfers in the same form factor.
- **Bulk wired (USB-HS bridge):** **~20–40 MB/s** — full library in minutes.
- **Do NOT market the ESP USB path;** it is ~0.5 MB/s and unused.
