# Variants — cross-comparison, BOM & dimensions

Three hardware axes (Radio × Storage × Form Factor) yield twelve unique board configurations, organized into three product tiers. The per-board `variants/<name>/BOM.csv` and `SPEC.md` are the authoritative source (generated from `gen_hardware.py`); this file is the at-a-glance comparison.

**Note:** The ESP32-E22-WROOM (Standard) variants are excluded to enforce the "Charm" form factor (the WROOM module requires an external antenna which violates the sealed/robust design principle). E22 variants are restricted to the "Mini" form factor (Project/Roadmap).

## The Product Tiers

We organize the 12 configurations into three distinct product lines based on the Chip (C5 vs E22) and Form Factor.

| Tier | Chip | Form Factor | Radio Options | Storage Options | Target User |
|---|---|---|---|---|---|
| **Tier 1: Charm** | ESP32-C5 | Standard (WROOM) | Single / Dual | microSD / eMMC | The baseline: cheapest, universal, swappable. |
| **Tier 2: Nano** | ESP32-C5 | Mini | Single / Dual | microSD / eMMC | Ultra-compact: for smallest footprint. |
| **Tier 3: Stealth Pro** | ESP32-E22 | Mini | Single / Dual | microSD / eMMC | High performance: Wi-Fi 6E in the smallest package. |

Transport (SoftAP / Wi-Fi Aware / wired USB-HS) is a firmware feature flag plus the bridge that *every* board carries — not a board change. See `../../docs/VARIANT_MATRIX.md`.

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
| 1× ESP32-C5 | ~4–9 MB/s (5 GHz Wi-Fi 6) | ~18 × 27.5 mm | yes | **Recommended baseline.** Best balance of power/performance. |
| 2× ESP32-C5 | ~13–16 MB/s (aggregated) | 2 modules + 2 antennas | tight | **Experimental.** Validate RF isolation before production. |
| 1× ESP32-E22 | ~100+ MB/s (Wi-Fi 6E) | Mini Only (~15 × 21 mm) | yes (warm) | **High Performance.** Tri-band (2.4/5/6 GHz). Requires >2A buck & larger battery. **Mini form factor only.** |
| 2× ESP32-E22 | ~200+ MB/s (aggregated) | Mini Only (2 modules) | tight (hot) | **Extreme Performance.** Significant thermal considerations. **Mini form factor only.** |

To go beyond ~9 MB/s on a single chip you must either run two radios (aggregation) or upgrade to the E22. The E22 is restricted to the Mini form factor in this design system to maintain the "Charm" identity (integrated antenna, sealed enclosure).

## Form Factor axis

### Standard (WROOM)
*   **Size:** ~18 × 27.5 mm module footprint (Updated per datasheet).
*   **Pros:** Better antenna efficiency (larger PCB antenna area), easier to hand-solder/route, widely available stock.
*   **Cons:** Larger overall device volume.
*   **Availability:** **C5 Only.** E22-WROOM excluded due to M.2/Antenna constraints.

### Mini (Compact)
*   **Size:** ~15.4 × 21.3 mm module footprint.
*   **Pros:** Significant PCB real-estate savings. Allows for larger battery in same enclosure or smaller device.
*   **Cons:** Slightly reduced antenna range (negligible for on-phone use), tighter routing constraints.
*   **Availability:** **C5 and E22.** The only form factor supporting the high-performance E22 "Stealth Pro" tier.
*   **Note:** Pin-for-pin compatible logic with Standard; only the footprint differs in `gen_hardware.py`.

## Dimensions & fit (confirmed)

The MCU module is the footprint driver; the battery is the thickness driver. The USB-HS bridge is a few mm (negligible).

| Storage | Enclosure envelope | Class |
|---|---|---|
| microSD | ~35 × 28 × 9 mm | AirTag-class |
| eMMC | ~32 × 24 × 8 mm | AirTag-class (smaller — no socket/insertion slot) |

**E22 Thermal Note:** High-speed E22 variants (Mini) generate more heat. The sealed enclosure may require thermal pads or metal injection molding (or a copper pour) to dissipate heat during sustained transfers.

Both satisfy the "charm on a string / forgettable on the back of the phone / doesn't block the charging port" concept. The parametric enclosure in `../../mechanical/enclosure.scad` renders STLs for both storage envelopes and has been verified to produce valid watertight geometry.

## One PCB philosophy

All 12 boards share the same logical architecture: USB-C, charger, gauge, bridge, mux, and UI.
*   **C5 Variants:** Share 600mA-1A buck regulator.
*   **E22 Variants:** Upgrade to 2A+ buck regulator and larger battery.
*   **Storage:** Keep microSD and eMMC footprints pin-compatible (depopulate/swap the storage block) so a single certification + a single end-of-line test jig covers both.
*   **Dual-Radio:** Adds the second module, the AGG crossover link, and larger power components.
