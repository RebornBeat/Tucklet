# Schematic Plan (net-by-net) — build this in KiCad

This is the complete electrical connection plan for the finalized design (ESP32-C5 radio + USB-HS storage bridge + power + UI), covering all variants. It is written as a netlist *intent* so you can place these symbols in KiCad and wire the nets directly — your established workflow.

Fixed-function pins (USB, SDIO, I2C) are given concretely. Flexible GPIO assignments are marked "→ free C5 GPIO" because the exact ESP32-C5-WROOM-1 GPIO numbers must be taken from the module datasheet pin table and must avoid the strapping pins; do not copy ESP32-S3 numbers, the C5 map differs.

## Reference designators
- U1  ESP32-C5-WROOM-1 module (radio + MCU)
- U1b ESP32-C5 #2 (DualC5 variant only)
- U2  USB 2.0 High-Speed card-reader / mass-storage bridge IC (e.g. a Genesys/Realtek-class SD bridge)
- U3  MCP73831 Li-ion charger
- U4  MAX17048 fuel gauge (I2C)
- U5  3.3 V buck regulator
- U6  2:1 analog mux / load switch for SDIO line arbitration (or use bridge's native shared-bus mode)
- J1  USB-C receptacle (USB 2.0 wiring)
- J2  microSD push-push socket  (MicroSd variant)   — OR —
- XU7 eMMC 153-ball BGA          (Emmc variant)
- SW1 tactile button
- LED1 WS2812 RGB
- BT1 LiPo cell w/ PCM

## Power nets
- `VBUS` : J1.VBUS → U3.VIN, → ESD/TVS, → input bulk cap (10 µF). Detect-able by U1 for "wired mode".
- `VBAT` : U3.BAT ↔ BT1(+) ↔ U4.CELL+ ↔ U5.VIN. Add 10 µF near U5.
- `+3V3` : U5.VOUT → U1.3V3 (+ U1b.3V3), U2.VDD, U4.VDD, SD/eMMC VDD, LED1.VDD, pull-up rails. Decoupling 100 nF per IC + 10 µF bulk near U1.
- `GND`  : common ground pour, stitched.
- USB-C `CC1`,`CC2` : each 5.1 kΩ → GND (sink role). 

## USB data (J1 ↔ bridge as the fast wired path)
The wired fast path is the BRIDGE (U2), not the C5's USB (C5/S3 USB is Full-Speed only, ~0.5 MB/s — never use it for storage).
- `USB_DP` : J1.D+ → (ESD array) → U2.DP
- `USB_DM` : J1.D- → (ESD array) → U2.DM
- Optionally also break out the C5 native USB to test pads for firmware flashing/JTAG (not the data path).

## Storage — shared between C5 (wireless mode) and bridge (wired mode)
Arbitration: when `VBUS` present and host enumerates, U2 owns the card; otherwise U1 owns it via SDIO. Implement with U6 muxing the SD bus, OR a bridge that supports a shared/he-handoff bus. SDIO 4-bit nets:
- `SD_CLK`, `SD_CMD`(10k pull-up), `SD_D0..D3`(10k pull-ups; D3=CS in SPI fallback)
- microSD variant: nets land on J2; `SD_DET` → free C5 GPIO (card-detect)
- eMMC variant: same logical nets land on XU7 (eMMC CLK/CMD/DAT0-3[-7]); no insertion/detect needed. eMMC supports 8-bit — optional DAT4-7 for higher wired throughput if the bridge supports it.
- C5 side SDIO → free C5 GPIO group (assign contiguous per datasheet); bridge side → U2 SD pins.

## I2C (fuel gauge)
- `SDA` : U4.SDA ↔ U1.(free GPIO)  + 4.7k pull-up to +3V3
- `SCL` : U4.SCL ↔ U1.(free GPIO)  + 4.7k pull-up to +3V3
- `ALRT`: U4.ALRT → free C5 GPIO (optional low-battery interrupt)

## UI
- `BTN`  : SW1 → GND, other side → free C5 GPIO with pull-up; 100 nF debounce cap. (summon-during-pairing / long-hold factory reset)
- `LED`  : U1.(free GPIO) → LED1.DIN; LED1.VDD=+3V3, GND; 300–470Ω series on data optional; 100 nF across LED1 supply.

## Charger / battery safety
- U3.PROG : Rprog → GND sets charge current (≈ size to cell, e.g. 1k≈ ... per datasheet).
- U3.STAT : → free C5 GPIO (charging indicator to firmware).
- NTC: if cell pack exposes a thermistor, route to charger NTC or to an ADC-capable C5 GPIO for charge-temperature qualification (firmware refuses charge out of window — see POWER_THERMAL.md).

## Strapping / boot (C5)
- Keep the C5 strapping pins at correct boot levels; do not load them with peripherals that disturb boot. Identify exact strap pins from the ESP32-C5-WROOM-1 datasheet before assigning any "free GPIO" above.

## DualC5 variant additions
- U1b powered from +3V3 (verify U5 sized for two radios' peak; see POWER_THERMAL).
- U1↔U1b coordination link: a UART or SPI between the two modules for link-aggregation control (`AGG_TX`/`AGG_RX` → free GPIOs each side).
- Two antennas with maximal isolation (orthogonal placement/polarization) — the key RF risk; validate on prototype.

## Per-variant summary
| Net group | SingleC5 | DualC5 | MicroSd | Emmc |
|---|---|---|---|---|
| Radio U1 | ✓ | ✓ (+U1b, +AGG link) | – | – |
| Storage | – | – | J2 + SD_DET | XU7 (no detect; opt. 8-bit) |
| Bridge U2 + mux U6 | ✓ | ✓ | ✓ | ✓ |
| Power/UI/charger | ✓ | ✓ (bigger buck/cell) | ✓ | ✓ |

Build order in KiCad: power → U1 → storage + mux → bridge → I2C/UI → (DualC5 second radio). Export the netlist when done; that's the file we iterate from.
