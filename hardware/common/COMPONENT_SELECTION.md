# Component Selection (shared across all variants)

This is the shared parts list and rationale for every Tucklet board. The
variant-specific BOMs (`variants/<name>/BOM.csv`) are generated from this same
model in `gen_hardware.py` and are the authoritative per-board cost; this file
explains *why* each part is chosen.

> Costs are order-of-magnitude at a few-hundred-unit volume. Verify live
> availability on LCSC / Digi-Key / Mouser before committing.

## 1. SoC / radio — ESP32-C5-WROOM-1  (`U1`, and `U1B` on dual)
- Dual-band **Wi-Fi 6 (2.4 + 5 GHz)** + **BLE** on one module. The 5 GHz band is
  less congested and roughly doubles the older ESP32-S3's wireless throughput —
  ~9 MB/s at the close range a phone-mounted charm enjoys.
- BLE is the always-on control/auth plane; Wi-Fi is the on-demand data plane.
- Using the **pre-certified module** (not a bare chip) avoids bare-die RF layout
  and substantially reduces the FCC/CE burden.
- SDIO for fast storage access; mature-enough esp-idf support.
- **Why C5 over S3:** the S3 is 2.4 GHz-only (~2–5 MB/s). For a product whose
  whole pitch is "fast enough to be invisible," the 5 GHz band matters. (If
  Wi-Fi Aware/NAN on the C5 proves unsupported during bring-up, the original
  ESP32 has confirmed NAN; SoftAP works on the C5 regardless. See
  `../../docs/FINAL_REVIEW.md`.)
- ~$2.60/module.

## 2. USB-HS storage bridge — generic USB2.0 High-Speed SD/eMMC bridge (`U2`)
- The fast **wired** path (~20–40 MB/s). It owns the storage directly when the
  device is plugged in, **bypassing the radio entirely**.
- Critical: the ESP32's *own* USB is Full-Speed only (~0.5 MB/s in practice), so
  the chip can never be the fast wired path. A dedicated bridge is what makes
  "plug in for a full-library dump in minutes" real.
- ~$1.60 (e.g. GL3224-class / RTS5306-class — confirm the part + its SD
  shared-bus/handoff behavior during bring-up).

## 3. Storage bus mux — SD 2:1 (`U6`)
- Arbitrates the single SDIO bus between the radio (A-side) and the bridge
  (B-side). `SD_SEL` (driven by USB-VBUS-present) selects the owner; never both.
- ~$0.45 (TS3A-class), or fold into a bridge that natively supports a shared bus.

## 4. USB-C — receptacle + protection (`J1`, `D1`, `R1`, `R2`)
- USB-C receptacle (USB 2.0 wiring) for charging + the wired data path.
- ESD array on D+/D−/VBUS; two 5.1 kΩ CC pulldowns (required for a USB-C sink).
- ~$0.30 + ~$0.22 protection/passives.

## 5. Battery charger — MCP73831 (`U3`)
- Single-cell Li-ion/LiPo linear charger from VBUS; `PROG` resistor sets current
  (size to the cell). `STAT` → a C5 GPIO so firmware knows charge state.
- ~$0.50.

## 6. Fuel gauge — MAX17048 (`U4`)
- I²C model-gauge so the app shows a **real** battery percentage, not a guess.
  `ALRT` → a C5 GPIO for a low-battery interrupt.
- ~$0.80.

## 7. Regulator — 3.3 V buck (`U5`)
- Sized for the **Wi-Fi TX peak (~500 mA bursts)**, not the average —
  under-sizing causes mid-transfer brownout resets, the #1 ESP bring-up bug.
- Dual-radio boards use a larger buck (≥1 A) for two simultaneous radios.
- ~$0.40 (single) / ~$0.55 (dual).

## 8. Battery — LiPo ~120–200 mAh + protection (`BT1`)
- Final mAh set **after measuring real 5 GHz transfer current** on a prototype.
  Dual-radio needs a larger cell. Always with a protection circuit (PCM).
- ~$2.00.

## 9. UI — button + RGB LED (`SW1`, `LED1`, `R6`)
- One tactile button (pair / factory-reset via press patterns) with a 10 kΩ
  pull-up and firmware debounce. One WS2812 addressable RGB for status.
- ~$0.30.

## 10. Passives — SD/I²C pull-ups, decoupling, bulk (`R3–R11`, `C_GRP`)
- SDIO CMD + data pull-ups (10 kΩ), I²C pull-ups (4.7 kΩ), PROG resistor,
  100 nF per IC rail, 10 µF bulk near U1/U5, USB/charger input caps.
- ~$0.80 grouped (expand `C_GRP` to individual C refs during layout).

## Shared electronics subtotal
Roughly **$9.5–$10.5** before the storage device, PCB, and enclosure — see each
`variants/<name>/BOM.csv` for the exact, authoritative totals. The
`singlec5-microsd` baseline lands at **~$13.66 full BOM**, inside the sub-$15
target; eMMC and dual-radio variants rise from there (flash and the second radio
cost money — that's expected and is exactly why both lines exist).

## Not produced here (and why)
- **Gerbers / pick-and-place:** outputs of PCB layout + DFM, not schematic
  capture. Cannot be honestly faked.
- **FCC/CE certification:** required (radios). The pre-certified module reduces
  but does not eliminate it. Budget time + cost before sale.
