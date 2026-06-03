# Component Selection (shared across all variants)

This is the shared parts list and rationale for every Tucklet board. The
variant-specific BOMs (`variants/<name>/BOM.csv`) are generated from this same
model in `gen_hardware.py` and are the authoritative per-board cost; this file
explains *why* each part is chosen.

> Costs are order-of-magnitude at a few-hundred-unit volume. Verify live
> availability on LCSC / Digi-Key / Mouser before committing.

## 1. SoC / radio — ESP32-C5 & ESP32-E22 Families (`U1`, and `U1B` on dual)

We offer two distinct product tiers via the radio selection, each available in Standard (WROOM) and Compact (Mini) form factors.

### Tier 1: ESP32-C5 (The "Charm" Baseline)
- **Performance:** Dual-band **Wi-Fi 6 (2.4 + 5 GHz)** + **BLE**. The 5 GHz band is less congested and roughly doubles the older ESP32-S3's wireless throughput — ~9 MB/s at the close range a phone-mounted charm enjoys.
- **Form Factors:**
    - **ESP32-C5-WROOM-1:** Standard footprint (**18.0 x 27.5 mm**). Larger PCB antenna area for optimal range.
    - **ESP32-C5-MINI-1:** Ultra-compact (**15.4 x 21.3 mm**). Same silicon performance, optimized for tightest enclosure fit.
- **Rationale:** BLE is the always-on control/auth plane; Wi-Fi is the on-demand data plane. Using the **pre-certified module** (not a bare chip) avoids bare-die RF layout and substantially reduces the FCC/CE burden.
- **Why C5 over S3:** the S3 is 2.4 GHz-only (~2–5 MB/s). For a product whose whole pitch is "fast enough to be invisible," the 5 GHz band matters.
- **Cost:** ~$2.60 (WROOM) / ~$2.40 (MINI).

### Tier 2: ESP32-E22 (The "High-Performance" Pro Line)
- **Performance:** Tri-band **Wi-Fi 6E (2.4, 5, 6 GHz)** + **BLE 5.4**. Features a dual-core RISC-V processor @ 500 MHz. Theoretical throughput exceeds 150 MB/s (device I/O limited).
- **Form Factors:**
    - **ESP32-E22-MINI-1 (Projected):** Ultra-compact footprint. **Primary target for the "Pro Charm" line.** Fits the Charm form factor with integrated PCB antenna.
    - **ESP32-E22-WROOM / M.2:** **Excluded.** Standard/M.2 modules require external IPEX antennas (reliability risk in a sealed charm) and larger board area, violating the "Charm" product identity.
- **Rationale:** Targets users with extreme data density or congested 6 GHz environments. This chip is typically a Radio Co-Processor (RCP) but is utilized here as the main application processor to leverage its massive wireless bandwidth.
- **Constraints:** Significantly higher power draw (~2A peak) and heat generation compared to the C5. Requires the "Pro" power subsystem (Section 7/8).
- **Cost:** ~$4.20 (MINI, estimated).

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
  (size to the cell). `STAT` → a GPIO so firmware knows charge state.
- ~$0.50.

## 6. Fuel gauge — MAX17048 (`U4`)
- I²C model-gauge so the app shows a **real** battery percentage, not a guess.
  `ALRT` → a GPIO for a low-battery interrupt.
- ~$0.80.

## 7. Regulator — 3.3 V buck (`U5`)
- Sized for the **Wi-Fi TX peak**, not the average — under-sizing causes mid-transfer brownout resets, the #1 ESP bring-up bug.
- **Tier 1 (C5):**
    - **Single Radio:** ~500 mA bursts. Standard 600 mA+ buck (~$0.40).
    - **Dual Radio:** ~1 A peak. Larger buck (~$0.55).
- **Tier 2 (E22):**
    - **All E22 Variants:** Dual-core 500 MHz + Wi-Fi 6E draws ~2 A peak. Requires a high-current buck regulator (e.g., TPS62840-class) (~$1.20).
- **Thermal Note:** On E22 boards, the regulator will dissipate noticeable heat. Ensure copper pour connection.

## 8. Battery — LiPo (`BT1`)
- Final mAh set **after measuring real transfer current** on a prototype.
- **Tier 1 (C5):** 120–200 mAh + PCM. ~$2.00. Fits the "Charm" envelope.
- **Tier 2 (E22):** 300–500 mAh + PCM. ~$3.50. Required to sustain high-throughput sessions. May dictate a slightly larger "Pro" enclosure or denser cell technology.

## 9. UI — button + RGB LED (`SW1`, `LED1`, `R6`)
- One tactile button (pair / factory-reset via press patterns) with a 10 kΩ
  pull-up and firmware debounce. One WS2812 addressable RGB for status.
- ~$0.30.

## 10. Passives — SD/I²C pull-ups, decoupling, bulk (`R3–R11`, `C_GRP`)
- SDIO CMD + data pull-ups (10 kΩ), I²C pull-ups (4.7 kΩ), PROG resistor,
  100 nF per IC rail, 10 µF bulk near U1/U5, USB/charger input caps.
- ~$0.80 grouped (expand `C_GRP` to individual C refs during layout).

## Shared electronics subtotal
Roughly **$9.5–$10.5** (C5 baseline) to **$14.00+** (E22 baseline) before the storage device, PCB, and enclosure — see each `variants/<name>/BOM.csv` for the exact, authoritative totals. The `singlec5-microsd` baseline lands at **~$13.66 full BOM**, inside the sub-$15 target; E22 variants rise from there (performance costs money — expected for the "Pro" line).

## Not produced here (and why)
- **Gerbers / pick-and-place:** outputs of PCB layout + DFM, not schematic
  capture. Cannot be honestly faked.
- **FCC/CE certification:** required (radios). The pre-certified module reduces
  but does not eliminate it. Budget time + cost before sale.
