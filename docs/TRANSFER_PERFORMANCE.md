# Transfer Performance & Layer-1 Wireless Architecture

This is the document that decides whether the product is good. Most wireless drives in history died on slow transfers + clunky apps (SanDisk Connect, Gnarbox, WD My Passport Wireless, the Seagate units — a graveyard of discontinued products). Tucklet targets exactly that weak spot, so the speed story has to be honest and engineered, not hoped for.

## Units (the thing that confused everyone)
Networks are quoted in mega**bits** (Mbps); files in mega**bytes** (MB/s). Divide Mbps by 8 to get MB/s. "30–70 Mbps" = "~4–9 MB/s". This doc uses **MB/s** because that's what a user feels.

## The two layers
- **Layer 1 — wireless data path.** The everyday path. Must be invisible and low-power. This is the focus.
- **Layer 2 — wired fast path.** A plug-in fallback for bulk. Optional, but it fits the dimensions (see below), so it stays in.

## The hard ceiling of the ESP32 family
Espressif's own number is "up to 20 Mbit/s TCP, 30 Mbit/s UDP over the air"; measured real-world results scatter from ~7–22 Mbit/s down to far less. Every ESP chip is a single-stream (1x1), 20–40 MHz radio, so it is physically capped around ~9 MB/s. The ESP32-S3's USB is Full-Speed only (12 Mbps ceiling) and its own "wireless disk" demo measured 540 KB/s read / 350 KB/s write — so the ESP chip itself can NEVER be the fast wired path.

## Layer-1 options (verified spectrum)

| Option | Class | Real throughput | Size | ~Cost (radio) | Battery | Fits charm? |
|---|---|---|---|---|---|---|
| ESP32-S3 | MCU | ~2–5 MB/s, 2.4GHz only | 18×25.5mm | ~$2.75 | weeks | Yes |
| **ESP32-C5** | MCU | **~4–9 MB/s**, +5GHz WiFi6 | ~same as S3 | ~$2–3 | weeks | Yes — RECOMMENDED |
| Dual ESP32-C5 (aggregation) | MCU x2, experimental | ~13–16 MB/s | 2 modules + 2 ant | ~$5–6 | days | Tight but plausible |
| MT7628 + 802.11ac + Linux | Router SoC | ~6–18+ MB/s dual-band | SoC+DDR+flash | ~$10–20 | big cell | No (FileHub-sized) |
| 2x2 ac/ax module + Linux SoM | App processor | ~20–50+ MB/s | larger | ~$15–30+ | big cell | No |

**Conclusion:** to exceed ~9 MB/s you either run two radios or jump to a Linux router SoC — which is literally what a FileHub is, and why a FileHub is a brick. "More Layer-1 power" and "invisible charm" pull in opposite directions. The C5 is the chosen balance.

### Decision: ESP32-C5
Dual-band Wi-Fi 6 (adds 5GHz, less congested), ~2x the S3's wireless, BLE retained for control, charm-sized, low power, in mass production. Newer/less mature tooling than S3 and single-core — acceptable. (If Wi-Fi Aware/NAN on the C5 proves unsupported, the original ESP32 has confirmed NAN; settle during bring-up. SoftAP works on C5 regardless.)

## The form factor is a hidden Layer-1 advantage
Because the charm sits ON the phone, the radio link is centimeters long → near-perfect SNR → the radio sustains its TOP modulation rate → you reliably land at the HIGH end (~9 MB/s), not the low end (~4). Competing drives sit across the room and suffer; Tucklet never does. The UX form factor also maximizes the cheap radio's throughput.

## Experimental Layer-1 boosters (unproven, flagged honestly)
1. **Dual-C5 link aggregation** (app-layer striping or MPTCP): ~13–16 MB/s. Hard part = RF isolation of two 5GHz antennas in ~32mm (orthogonal placement + polarization mitigates; not guaranteed until prototyped). Best "beyond C5" lever that might still fit the charm. Treat as v2.
2. **Magnetic snap-on contact pins** (MagSafe-style): full wired speed with snap-on feel. Catch: phones expose no back-side data pins, only the charge port — needs a companion case or snap-over-USB-C. Accessory path.
3. **60 GHz / 802.11ad**: gigabits at cm range (ideal for a charm), but no phone exposes it to third parties and modules are power-hungry/costly. Dream, not buildable now.
4. **Trickle-sync (THE winning lever, not hardware):** continuously back up new photos whenever the charm is near the phone and idle/charging, so the big transfer NEVER happens and raw throughput stops mattering for the everyday case. Cheap radio + always-present + patient background sync beats a fast radio used occasionally. Strong on Android; partial on iOS (background-networking limits). This is the design philosophy that makes a 9 MB/s charm feel better than an 18 MB/s brick.

## Layer 2 — wired fast path (kept, because it fits)
A dedicated USB 2.0 High-Speed card-reader/bridge IC (~$1–2, ~5mm QFN) owns the microSD/eMMC when plugged in, bypassing the ESP entirely → ~20–40 MB/s. Storage is shared by mode arbitration: wireless mode = ESP owns the card; plugged-in mode = bridge owns it; never both at once. On desktop this also makes the device a plain USB drive (easiest platform). Negligible size impact.

## Dimensions — reconfirmed for all options
- **C5 + USB-HS bridge:** bridge is negligible; comfortably inside the AirTag envelope (~32–35mm). Confirmed.
- **Dual-C5:** board area fits; constraint is two-antenna isolation in 32mm + slightly bigger battery. Borderline-plausible — validate on prototype before finalizing enclosure.
- **Linux router SoC:** DDR + SoC + flash + bigger battery exceed the charm envelope → deck-of-cards size. Only if speed is chosen over form factor.
- Battery is the thickness driver in every case, not the logic.

## Headline numbers to design and market around
- Everyday wireless (C5, on-phone, top-of-range): **~9 MB/s** — fine for photos/short clips, invisible via trickle-sync.
- Bulk wired (USB-HS bridge): **~20–40 MB/s** — full library in minutes.
- Do NOT market the ESP USB path; it is ~0.5 MB/s and unused.
