# Variants — cross-comparison, BOM & dimensions

Three hardware axes (Radio × Storage × Form Factor) yield twelve unique board configurations for the **Charm** product line. The per-board `variants/<name>/BOM.csv` and `SPEC.md` are the authoritative source (generated from `gen_hardware.py`); this file is the at-a-glance comparison.

## The Product Lines

Tucklet is organized as two distinct product lines, both under the same source-available, non-commercial licensing:

| Line | Radio | Form Factor | Battery | Variants | Position |
|---|---|---|---|---|---|
| **Charm Standard** | ESP32-C5 | WROOM | 1S LiPo | 4 | Baseline: Lowest cost, easy assembly |
| **Charm Compact** | ESP32-C5 | MINI | 1S LiPo | 4 | Nano: Smallest footprint, tighter layout |
| **Charm High-Perf** | ESP32-E22 | MINI | 1S LiPo (Roadmap) | 4 | Wi-Fi 6E speed, pending module release |
| **Charm Pro** | ESP32-E22 | WROOM | 2S LiPo (Roadmap) | Planned | High-Performance, larger envelope |

All lines are licensed under **CC BY-NC-SA 4.0** (hardware) and **PolyForm Noncommercial 1.0.0** (software). The Pro line is a **Charm** — it fits a "charm" envelope (wearable, on-phone), even though it is thicker to accommodate the 2S battery. It is not a "backpack" or "brick" class device. You may study, modify, and build for personal use; commercial sale requires a separate agreement.

**Note:** The ESP32-E22 variants (both MINI and WROOM) are **Roadmap** items. The `ESP32-E22-MINI-1` module is projected but not yet released. The `ESP32-E22-WROOM` module (22×30 mm, external IPEX antenna) is the basis for the Pro line. **No E22 variant folders are created at this time** (see "Variant Folder Organization" below).

## Storage axis

### microSD (swappable)
Push-push socket, SDIO 4-bit, +~$0.40. Customer supplies/upgrades the card; you don't pay for flash. Dead card = $5 swap, not a dead device. Slightly larger enclosure (socket + insertion clearance), placed internally behind a small door so the unit still looks sealed.

### eMMC (sealed) — per-capacity cost (small-volume estimate; VERIFY)
153-ball TFBGA eMMC 5.1, 11.5 × 13.0 × 1.2 mm. Capacity is fixed at manufacture.

| Capacity | eMMC est. | Full BOM (single-C5) |
|---|---|---|
| 8 GB | ~$2.50 | ~$13.5 |
| 16 GB | ~$3.50 | ~$14.5 |
| 32 GB | ~$5.00 | ~$16.0 |
| 64 GB | ~$8.00 | ~$19.0 |
| 128 GB | ~$13.00 | ~$24.0 |
| 256 GB | ~$22.00 | ~$33.0 |

Only 8–16 GB eMMC stays near the sub-$15 target — the honest cost of integrated flash, and exactly why offering microSD *and* eMMC is smart. Requires BGA assembly (CM with reflow + ideally X-ray).

## Radio axis

Performance and Power vary significantly between the C5 and E22 generations.

| Radio | Real wireless | Module size | Fits charm? | Notes |
|---|---|---|---|---|
| 1× ESP32-C5 | ~6–9 MB/s (5 GHz Wi-Fi 6) | WROOM: ~18 × 27.5 mm / MINI: ~15.4 × 21.3 mm | yes | **Recommended baseline.** Best balance of power/performance. |
| 2× ESP32-C5 | ~12–15 MB/s (SPI AGG) | 2 modules + 2 antennas | tight | **Experimental.** Validate RF isolation before production. SPI link prevents inter-chip bottleneck. |
| 1× ESP32-E22 | ~20–40+ MB/s (Wi-Fi 6E) | MINI: ~15 × 21 mm (projected) | yes (warm) | **High Performance.** Tri-band (2.4/5/6 GHz). Requires >2A buck & larger battery. **MINI form factor only. Roadmap.** |
| 2× ESP32-E22 | ~40–70+ MB/s (SPI AGG) | MINI: 2 modules | tight (hot) | **Extreme Performance.** Significant thermal considerations. **MINI form factor only. Roadmap.** |
| 1× ESP32-E22 (Pro) | ~20–40+ MB/s (Wi-Fi 6E) | WROOM: 22 × 30 mm | yes (Pro envelope) | **Pro Charm.** External IPEX antenna. 2S battery. ~35×28×12–13 mm envelope. **Roadmap.** |

To go beyond ~9 MB/s on a single chip you must either run two radios (aggregation) or upgrade to the E22. The E22 is restricted to MINI in the standard Charm line to maintain the integrated-antenna sealed identity. The E22-WROOM is reserved for the Pro line where the external antenna and 2S battery justify a thicker envelope that remains charm-class (wearable, on-phone).

## Form Factor axis

### Standard (WROOM)
*   **Size:** ~18 × 27.5 mm module footprint.
*   **Pros:** Better antenna efficiency (larger PCB antenna area), easier to hand-solder/route, widely available stock.
*   **Cons:** Larger overall device volume.
*   **Availability:** **C5 Only.** E22-WROOM reserved for Pro line.

### Mini (Compact)
*   **Size:** ~15.4 × 21.3 mm module footprint.
*   **Pros:** Significant PCB real-estate savings. Allows for larger battery in same enclosure or smaller device.
*   **Cons:** Slightly reduced antenna range (negligible for on-phone use), tighter routing constraints.
*   **Availability:** **C5 and E22.** The only form factor supporting the high-performance E22 in the standard Charm line.

## Battery axis (1S vs 2S)

The choice between 1S (3.7V) and 2S (7.4V) LiPo impacts the Charger, Gauge, Buck Regulator, Battery, and Enclosure dimensions.

| Attribute | **1S LiPo (Charm Standard)** | **2S LiPo (Charm Pro)** |
|---|---|---|
| **Nominal Voltage** | 3.7V (4.2V max) | 7.4V (8.4V max) |
| **Charger (U3)** | **BQ25896** (Buck, I2C, Power-Path) | **BQ25887** (Boost + Cell Balancing) |
| **Fuel Gauge (U4)** | **MAX17048** (ModelGauge, no sense resistor) | **BQ27421** or **BQ27520** (Impedance Track, requires sense resistor) |
| **Buck Regulator (U5)** | Step-down from 4.2V to 3.3V | Step-down from 8.4V to 3.3V (sized for ≥2A) |
| **Battery** | 120–200 mAh, ~3mm thick | ~120–150 mAh, ~6mm thick (stacked cells) |
| **Enclosure** | 9mm (microSD) / 8mm (eMMC) | **12–13mm** (breaks AirTag class; justified by performance) |
| **Power-Path** | Yes (BQ25896 SYS pin) | Yes (BQ25887 SYS pin) |
| **Peak Current** | Sufficient for single/dual C5 | Sustained high-current for E22/dual |

**The 1S path is the default for all C5 and E22-MINI Charm variants.** The 2S path is exclusive to the **Pro** line (E22-WROOM) and requires the larger Pro envelope.

## Dimensions & fit (confirmed)

The MCU module is the footprint driver; the battery is the thickness driver. The USB-HS bridge is a few mm (negligible).

| Storage + Battery | Enclosure envelope | Class |
|---|---|---|
| microSD + 1S | ~35 × 28 × 9 mm | AirTag-class |
| eMMC + 1S | ~32 × 24 × 8 mm | AirTag-class (smaller — no socket/insertion slot) |
| Any + 2S (Pro) | ~35 × 28 × 12–13 mm | Charm-class (thicker Pro; still wearable) |

**E22 Thermal Note:** High-speed E22 variants (MINI and Pro) generate more heat. The sealed enclosure may require thermal pads or metal injection molding (or a copper pour) to dissipate heat during sustained transfers.

All 1S envelopes satisfy the "charm on a string / forgettable on the back of the phone / doesn't block the charging port" concept. The Pro envelope remains a charm (wearable, on-phone), just thicker. The parametric enclosure in `../../mechanical/enclosure.scad` renders STLs for both storage envelopes and has been verified to produce valid watertight geometry.

## One PCB philosophy

All boards share the same logical architecture: USB-C, charger, gauge, bridge, mux, and UI.
*   **C5 Variants:** Share 600mA-1A buck regulator.
*   **E22 Variants:** Upgrade to 2A+ buck regulator and larger battery.
*   **Pro Variants:** Further upgrade to 2S charger/gauge and Pro envelope.
*   **Storage:** Keep microSD and eMMC footprints pin-compatible (depopulate/swap the storage block) so a single certification + a single end-of-line test jig covers both.
*   **Dual-Radio:** Adds the second module, the SPI AGG crossover link, and larger power components.

## Component Alternatives Summary

Every variant has a `standard/` and `alternatives/` subfolder structure capturing the exact MPNs, pin mappings, and netlists for each component choice. See `hardware/README.md` for full details.

| Component | Primary (Standard) | Backup (Alternatives) |
|---|---|---|
| **USB Bridge (U2)** | GL3224-OEM (QFN-32, USB 3.0, ~70–100+ MB/s) | GL823 (QFN-24, USB 2.0, ~25–35 MB/s), PL2732 (QFN-32, USB 3.0), AU6601 (QFN-48, USB 3.0) |
| **Charger (U3)** | BQ25896 (WQFN-24, power-path buck) | MCP73831 (SOT-23-5, simple linear), BQ24072 (VQFN-16, power-path linear) |
| **Fuel Gauge (U4)** | MAX17048 (TDFN-8, 1S) | BQ27421 (QFN, 1S/2S, requires sense resistor) |
| **SD Mux (U6)** | TS3A-class (QFN-20) | TMUX1574 (QFN-24, signal integrity), TXS02612 (QFN-24, SD-specific) |
| **Buck Regulator (U5)** | TPS62740-class (1S, per variant) | TPS62840-class (E22/2S, ≥2A) |
| **Radio (U1/U1B)** | ESP32-C5-WROOM-1 / MINI-1 | ESP32-E22-MINI-1 (Roadmap), ESP32-E22-WROOM (Pro Roadmap) |

## Variant Folder Organization

Within each `variants/<name>/` folder, the component alternatives and design paths are organized into subfolders. This ensures that every physical board variant has a clear, traceable path for both the **Primary (Standard)** build and any **Backup (Alternative)** component choices. This is critical for BOM flexibility and supply-chain resilience, and it ensures the specific implementations are captured for licensing provenance.

```
variants/<name>/
├── SPEC.md
├── PIN_MAP.md
├── block_diagram.svg
├── standard/               # PRIMARY build path (verified components)
│   ├── BOM.csv             # BOM with primary MPNs (GL3224, BQ25896, etc.)
│   ├── SCHEMATIC_PLAN.md   # Net list using primary pinouts
│   └── tucklet-<name>.net  # Netlist for primary components
└── alternatives/           # BACKUP build paths (supply chain alternatives)
    ├── usb_bridge_gl823/   # Alternative: GL823 (USB 2.0, QFN-24)
    │   ├── BOM.csv         # BOM delta or full BOM with GL823
    │   ├── SCHEMATIC_PLAN.md  # Pin/net changes for GL823
    │   └── tucklet-<name>-gl823.net
    ├── usb_bridge_pl2732/  # Alternative: PL2732 (USB 3.0, QFN-32)
    │   ├── BOM.csv
    │   ├── SCHEMATIC_PLAN.md
    │   └── tucklet-<name>-pl2732.net
    ├── charger_mcp73831/   # Alternative: MCP73831 (Linear, no power-path)
    │   ├── BOM.csv
    │   ├── SCHEMATIC_PLAN.md
    │   └── tucklet-<name>-mcp73831.net
    ├── charger_bq24072/    # Alternative: BQ24072 (Power-path linear)
    │   ├── BOM.csv
    │   ├── SCHEMATIC_PLAN.md
    │   └── tucklet-<name>-bq24072.net
    └── sd_mux_tmux1574/   # Alternative: TMUX1574 (Signal integrity upgrade)
        ├── BOM.csv
        ├── SCHEMATIC_PLAN.md
        └── tucklet-<name>-tmux1574.net
```

**Why this structure?**
- **Licensing Specificity:** Copyright protects *specific implementations*. The `standard/` and `alternatives/` folders capture the exact MPNs, pin mappings, and netlists for each component choice. A generic "USB Bridge" is unprotectable; `GL3224-OEM` is.
- **Supply Chain Resilience:** If the GL3224 is out of stock, the GL823 or PL2732 path is already documented and netlisted.
- **Design Clarity:** No ambiguity about which BOM goes with which schematic.

**Note:** Not all alternatives apply to all variants. The generator creates only the relevant subfolders based on the variant's configuration (e.g., a single-C5 variant may not need the TMUX1574 upgrade unless signal integrity issues are observed).

## E22 and Pro Line Status

### E22-MINI Variants (Roadmap)
The `ESP32-E22-MINI-1` is a projected component. While the architecture and netlists for the 4 E22-MINI Charm variants are generated and valid, the module is **not yet released** and datasheets are unavailable. Do not fabricate E22-MINI variants until:
1. The hardware is procured.
2. Footprints are verified against the official datasheet.
3. The GPIO map is reconciled (PIN_MAP.md must be updated).

### E22-WROOM / Pro Line (Source-available, Non-commercial)
The `ESP32-E22-WROOM` (22×30 mm module with external IPEX antenna) is explicitly excluded from the standard Charm variants due to size and antenna constraints. However, this module is the basis for the **Pro Line**, a thicker but still charm-class wearable with a 2S LiPo battery pack.

The Pro design IP is documented in [`hardware/PRO_LINE.md`](hardware/PRO_LINE.md). Like the Charm line, the Pro line is fully covered under the **CC BY-NC-SA 4.0** (hardware) and **PolyForm Noncommercial 1.0.0** (software) licenses. You may study, modify, and build for personal use; commercial sale requires a separate agreement.

**No Pro variant folders are created at this time.** When the E22-WROOM datasheet is released and the Pro line is ready for prototyping, variant folders (e.g., `pro-e22-wroom-microsd/`) will be added with the same `standard/` + `alternatives/` structure, under the same source-available, non-commercial license as the Charm line.
