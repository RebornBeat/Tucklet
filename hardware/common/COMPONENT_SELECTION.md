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
    - **ESP32-E22-WROOM / M.2:** **Excluded from Charm line.** Standard/M.2 modules require external IPEX antennas (reliability risk in a sealed charm) and larger board area, violating the "Charm" product identity. This module is the basis for the separate, source-available, non-commercial **Pro Line**.
- **Rationale:** Targets users with extreme data density or congested 6 GHz environments. This chip is typically a Radio Co-Processor (RCP) but is utilized here as the main application processor to leverage its massive wireless bandwidth.
- **Constraints:** Significantly higher power draw (~2A peak) and heat generation compared to the C5. Requires the "Pro" power subsystem (Section 7/8).
- **Cost:** ~$4.20 (MINI, estimated).

## 2. USB-HS storage bridge — GL3224-OEM (`U2`)
- The fast **wired** path (~70–100+ MB/s). It owns the storage directly when the
  device is plugged in, **bypassing the radio entirely**.
- Critical: the ESP32's *own* USB is Full-Speed only (~0.5 MB/s in practice), so
  the chip can never be the fast wired path. A dedicated bridge is what makes
  "plug in for a full-library dump in minutes" real.
- **Primary Part:** **GL3224-OEM** — QFN-32 (5×5×0.8 mm), USB 3.0 (5 Gbps).
  Single-LUN variant. JBOD support. Firmware-upgradable. Pin mapping must be verified from the GL3224-OEM datasheet. USB 3.0 SuperSpeed differential pairs (SS_TX_P/N, SS_RX_P/N) may be NC in USB 2.0 fallback.
- **Alternative Packages:**
    - **GL3224-OEM QFN-48:** 7×7×0.8 mm. Dual-LUN. Tighter fit in eMMC-only envelopes. Use QFN-32 as default for Charm.
- ~$1.80.

### Alternative USB Bridges
- **PL2732 (Backup 1):** QFN-32 (5×5×0.8 mm), USB 3.0. Prolific. Excellent eMMC HS200 support. Firmware initialization may differ. Equivalent performance (~70–100+ MB/s). ~$1.70.
- **GL823 (Backup 2, Size-Constrained):** QFN-24 (4×4×0.8 mm), USB 2.0 (480 Mbps). Smallest footprint. Lowest power. **~25–35 MB/s wired.** Correct 24-pin part with full 4-bit SDIO support. ~$1.60.
- **AU6601 (Backup 3):** QFN-48 (7×7×0.8 mm), USB 3.0. Alcor Micro. Larger footprint. Last resort if GL3224/PL2732 unavailable.
- **Infineon EZ-USB SD3 (Roadmap):** BGA-56 (~7×7 mm), USB 5 Gbps. Highest performance. Requires BGA assembly.

> **Critical Distinction:** GL823 (24-pin, full SDIO) ≠ GL823K (16-pin, limited). The GL823K cannot route full 4-bit SDIO and must be avoided for Tucklet.

## 3. Storage bus mux — SD 2:1 (`U6`)
- Arbitrates the single SDIO bus between the radio (A-side) and the bridge
  (B-side). `SD_SEL` (driven by USB-VBUS-present) selects the owner; never both.
- **Primary Part:** **TS3A-class** — QFN-20 (~4×4×0.8 mm). Sufficient bandwidth for SDIO. Pinout varies by exact part number; verify against your chosen mux's datasheet.
- ~$0.45.

### Alternative SD Muxes
- **TMUX1574 (Signal Integrity Upgrade):** QFN-24 (~4×4×0.8 mm). Higher bandwidth, lower on-resistance (Ron). Better signal integrity for SDIO at higher clock rates. Recommended if timing issues observed on prototype. ~$0.55.
- **TXS02612 (SD-Specific):** QFN-24. SD-optimized with integrated level shifting. Useful if voltage translation is needed between the bridge and storage. ~$0.50.

## 4. USB-C — receptacle + protection (`J1`, `D1`, `R1`, `R2`)
- USB-C receptacle (USB 2.0 wiring) for charging + the wired data path.
- ESD array on D+/D−/VBUS; two 5.1 kΩ CC pulldowns (required for a USB-C sink).
- ~$0.30 + ~$0.22 protection/passives.

## 5. Battery charger — BQ25896 (`U3`)
- **BQ25896** — WQFN-24 (4×4×0.8 mm), I²C-controlled buck charger with
  integrated power-path. System can run from USB power while the battery charges,
  which is critical: you can use the charm while it's plugged in without draining
  the cell.
- Power-path output (`SYS`) feeds the 3.3 V buck; charge current set via I²C
  (or PROG resistor fallback). `STAT` and `INT` pins connect to GPIOs for firmware
  status. Thermistor input (`TS`) for charge-temperature qualification.
- **Pin mapping note:** Pin numbers in the generated netlists are logical
  placeholders. Verify against the BQ25896 datasheet before layout.
- ~$1.50.

### Alternative Chargers
- **MCP73831 (Simple Linear, No Power-Path):** SOT-23-5 (2.9×2.8×1.4 mm). Simplest
  option. Charge current set by PROG resistor only. No I²C. No power-path — system
  runs from battery only, even when plugged in. Acceptable for single-C5 builds
  where simultaneous charge+transfer is rare. ~$0.50.
- **BQ24072 (Power-Path Linear):** VQFN-16 (3.5×3.5×0.9 mm). Power-path without
  the I²C complexity of the BQ25896. Good middle ground if you need to run the
  radio while charging but don't want software-controlled charging. ~$0.90.
- **TP4056 (Ubiquitous Linear):** SOP-8 (~5×6 mm). Cheap and available. Larger
  footprint. No power-path. Consider only for budget/availability builds. ~$0.20.
- **MCP73871 (Tiny Power-Path):** SOT-23-6 / DFN-6 (~2×3 mm). Very small
  power-path option. Consider for Mini variants where space is critical. ~$0.60.
- **MAX14748 (USB Power Manager):** WLP-12 / TQFN-14 (~3×3 mm). USB-specific
  power manager with integrated buck. Compact. ~$1.00.

### 2S (Pro Line) Charger
- **BQ25887:** VQFN-24 (4×4 mm). 2-cell (2S) boost charger with cell balancing.
  Required for the Pro Line (E22-WROOM + 2S LiPo). Not for 1S builds. ~$1.80.

## 6. Fuel gauge — MAX17048 (`U4`)
- I²C model-gauge so the app shows a **real** battery percentage, not a guess.
  `ALRT` → a GPIO for a low-battery interrupt.
- **MAX17048** — TDFN-8 (3×3×0.8 mm). Best accuracy without a current-sense
  resistor. 1S only.
- ~$0.80.

### Alternative Fuel Gauges
- **BQ27421:** QFN (~3.5×3.5 mm). Impedance Track algorithm. Requires external
  sense resistor. Different I²C command set. Suitable for both 1S and 2S
  configurations. ~$0.90.
- **BQ27520:** QFN (~4×4 mm). More features, larger. 2S backup option. ~$1.20.

## 7. Regulator — 3.3 V buck (`U5`)
- Sized for the **Wi-Fi TX peak**, not the average — under-sizing causes mid-transfer brownout resets, the #1 ESP bring-up bug.
- **Tier 1 (C5):**
    - **Single Radio:** ~500 mA bursts. Standard 600 mA+ buck (TPS62740-class, SOT-23-6). ~$0.40.
    - **Dual Radio:** ~1 A peak. Larger buck (TPS62740-class, SOT-23-6). ~$0.55.
- **Tier 2 (E22):**
    - **All E22 Variants:** Dual-core 500 MHz + Wi-Fi 6E draws ~2 A peak. Requires a high-current buck regulator (TPS62840-class, SOT-23-6). ~$1.20.
- **2S (Pro Line):** Step-down from 8.4 V to 3.3 V at ≥2 A. Different inductor/feedback network. TPS62840-class. ~$1.20.
- **Thermal Note:** On E22 and 2S boards, the regulator will dissipate noticeable heat. Ensure copper pour connection.

## 8. Battery — LiPo (`BT1`)
- Final mAh set **after measuring real transfer current** on a prototype.
- **Tier 1 (C5):** 120–200 mAh + PCM. ~$2.00. Fits the "Charm" envelope.
- **Tier 2 (E22):** 300–500 mAh + PCM. ~$3.50. Required to sustain high-throughput sessions. May dictate a slightly larger "Pro" enclosure or denser cell technology.
- **2S (Pro Line):** ~120–150 mAh + PCM (stacked cells). ~6 mm thick. Requires the Pro enclosure (~12–13 mm total). ~$3.50–5.00.

## 9. UI — button + RGB LED (`SW1`, `LED1`, `R6`)
- One tactile button (pair / factory-reset via press patterns) with a 10 kΩ
  pull-up and firmware debounce. One WS2812 addressable RGB for status.
- ~$0.30.

## 10. Passives — SD/I²C pull-ups, decoupling, bulk (`R3–R11`, `C_GRP`)
- SDIO CMD + data pull-ups (10 kΩ), I²C pull-ups (4.7 kΩ), PROG resistor,
  100 nF per IC rail, 10 µF bulk near U1/U5, USB/charger input caps.
- ~$0.80 grouped (expand `C_GRP` to individual C refs during layout).

### BQ25896-Specific Passives
- **I²C Pull-ups:** The BQ25896 shares the I²C bus with U4 (MAX17048). Only one set of 4.7 kΩ pull-ups is needed for the bus.
- **PROG Resistor:** Sets charge current. Size to your cell (e.g., 2 kΩ ≈ 500 mA for a 150 mAh cell).
- **TS Thermistor:** Connect a 10 kΩ NTC thermistor to the `TS` pin for charge-temperature qualification. If omitted, tie `TS` to `VREF` with a resistor divider per the BQ25896 datasheet.

## Shared electronics subtotal
Roughly **$9.5–$10.5** (C5 baseline) to **$14.00+** (E22 baseline) before the storage device, PCB, and enclosure — see each `variants/<name>/BOM.csv` for the exact, authoritative totals. The `singlec5-microsd` baseline lands at **~$13.66 full BOM**, inside the sub-$15 target; E22 variants rise from there (performance costs money — expected for the "Pro" line).

## Not produced here (and why)
- **Gerbers / pick-and-place:** outputs of PCB layout + DFM, not schematic
  capture. Cannot be honestly faked.
- **FCC/CE certification:** required (radios). The pre-certified module reduces
  but does not eliminate it. Budget time + cost before sale.
