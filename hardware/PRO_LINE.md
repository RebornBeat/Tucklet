# Pro Line (E22-WROOM + 2S) — Source-available, Non-commercial

**Status:** Design IP documented, source-available, non-commercial. Roadmap for prototyping when E22-WROOM datasheet is released.

**License:** Like the Charm line, the Pro line is fully covered under the **CC BY-NC-SA 4.0** (hardware) and **PolyForm Noncommercial 1.0.0** (software) licenses. You may study, modify, and build for personal use; commercial sale requires a separate agreement.

---

## What the Pro line is

A larger, higher-performance Tucklet variant built around the **ESP32-E22-WROOM** (22×30 mm module with external IPEX antenna) and a **2S LiPo** battery pack. This requires a larger enclosure (~35×28×12–13 mm) and, like the Charm line, is fully covered under the source-available, non-commercial license.

The Pro is **still a charm** — it clips to your phone, hangs on a string, and disappears into your daily carry. It is not a "backpack" or "brick" class device. The thickness is justified by the performance: Wi-Fi 6E throughput and sustained high-current operation that the 1S Charm line cannot deliver.

No Pro variant folders are created at this time. When the E22-WROOM datasheet is released and the Pro line is ready for prototyping, variant folders (e.g., `pro-e22-wroom-microsd/`) will be added with the same `standard/` + `alternatives/` structure, under the same source-available, non-commercial license as the Charm line.

---

## Why E22-WROOM is excluded from Charm but valid for Pro

The E22-WROOM module (22×30 mm) requires an **external IPEX antenna connector** and has a significantly larger footprint than the C5-WROOM or any MINI variant. In the AirTag-class Charm envelope (~35×28×9 mm):

- The module itself nearly spans the full board length
- The IPEX antenna and its cable/ceramic patch add assembly complexity and a reliability risk in a sealed device
- The 2S battery (required to sustain E22 power draw) adds ~3 mm of thickness

These constraints violate the "integrated antenna, sealed, forgettable" Charm identity. However, in a **Pro Charm** envelope (~35×28×12–13 mm):

- The IPEX antenna can be mounted along the long edge or top face with adequate keep-out
- The 2S battery fits with room for the module and supporting components
- The performance justifies the form factor

The Pro remains wearable and phone-mounted — it is a "Pro Charm," not a new product category.

---

## Key differences from the Charm line

| Attribute | **Charm (1S)** | **Pro Charm (2S)** |
|---|---|---|
| **Radio** | ESP32-C5 (WROOM or MINI) | **ESP32-E22-WROOM** (IPEX antenna) |
| **Battery** | 1S LiPo (3.7V, 120–200 mAh) | **2S LiPo** (7.4V, ~120–150 mAh, ~6 mm thick) |
| **Charger** | BQ25896 (1S buck charger) | **BQ25887** (2S boost + cell balancing) |
| **Fuel Gauge** | MAX17048 (ModelGauge, no sense resistor) | **BQ27421** or **BQ27520** (Impedance Track, requires sense resistor) |
| **Buck Regulator** | V_IN up to 4.2V → 3.3V | **V_IN up to 8.4V → 3.3V** (different inductor/feedback) |
| **Enclosure** | 9 mm (microSD) / 8 mm (eMMC) | **12–13 mm** (breaks AirTag class; justified by performance) |
| **Power-Path** | Yes (BQ25896 SYS pin) | Yes (BQ25887 SYS pin) |
| **Peak Current** | Sufficient for single/dual C5 | Sustained high-current for E22 |
| **Antenna** | Integrated PCB (WROOM/MINI) | **External IPEX** + cable/ceramic patch |
| **License** | CC BY-NC-SA 4.0 / PolyForm NC | **CC BY-NC-SA 4.0 / PolyForm NC** (same as Charm) |

---

## Component selection (Pro-specific)

### Charger — BQ25887 (2S Boost + Cell Balancing)

| Attribute | Value |
|---|---|
| **MPN** | BQ25887 |
| **Package** | VQFN-24 (4×4 mm) |
| **Type** | 2-cell (2S) boost charger with cell balancing |
| **I²C** | Yes (shared bus with gauge) |
| **Power-Path** | Yes (SYS output powers system while charging) |
| **Input** | USB-C VBUS (5V) |
| **Output** | 2S LiPo (8.4V max charge) |
| **Balancing** | Integrated cell balancing during charge |
| **Cost** | ~$1.80 |

**Alternative:** No direct pin-compatible alternative for 2S I²C boost charger. If unavailable, a discrete solution (boost converter + separate balancer IC + separate gauge) is possible but significantly more complex.

**Backup (discrete path):**
- **MCP1642B** (boost converter for 2S charging, no I²C)
- **BQ7791502** (standalone cell balancing/protection)
- **BQ27421** (fuel gauge, same as primary)

### Fuel Gauge — BQ27421 or BQ27520 (2S Impedance Track)

| Attribute | BQ27421 | BQ27520 |
|---|---|---|
| **MPN** | BQ27421-G1A | BQ27520 |
| **Package** | QFN (~3.5×3.5 mm) | QFN (~4×4 mm) |
| **Algorithm** | Impedance Track | Impedance Track |
| **Sense Resistor** | Required (external) | Required (external) |
| **Cells** | 1S–2S | 1S–2S |
| **I²C** | Yes | Yes |
| **Cost** | ~$0.90 | ~$1.20 |

**Note:** Unlike the MAX17048 (1S, no sense resistor), both 2S gauges require an external current-sense resistor in the battery path. This is an additional BOM line item and PCB footprint.

### Buck Regulator — 8.4V Input, ≥2A Output

| Attribute | Value |
|---|---|
| **Input** | Up to 8.4V (2S fully charged) |
| **Output** | 3.3V at ≥2A (E22 dual-core + Wi-Fi 6E peak) |
| **Recommended** | TPS62840-class (SOT-23-6 or similar) |
| **Cost** | ~$1.20 |

**Critical:** The buck must handle the full 8.4V input and deliver ≥2A at 3.3V for sustained E22 operation. This is a different operating point than the 1S buck (4.2V → 3.3V at 0.6–1A). The inductor selection and feedback network must be sized accordingly.

### Battery — 2S LiPo (~120–150 mAh, ~6 mm thick)

| Attribute | Value |
|---|---|
| **Configuration** | 2S (2 cells in series) |
| **Nominal Voltage** | 7.4V (8.4V fully charged) |
| **Capacity** | ~120–150 mAh (per cell; stacked) |
| **Thickness** | ~6 mm (dominates enclosure height) |
| **Protection** | Integrated PCM (over-charge, over-discharge, balancing) |
| **Cost** | ~$3.50–5.00 (2S pack with PCM) |

**Note:** Final mAh must be determined after measuring real E22 transfer current on a prototype. The E22's dual-core 500 MHz processor + Wi-Fi 6E radio draws significantly more power than the C5.

---

## Enclosure — Pro Charm Envelope

| Attribute | Value |
|---|---|
| **Outer dimensions** | ~35 × 28 × 12–13 mm |
| **Form factor** | Pro Charm (thicker, still wearable/on-phone) |
| **Lanyard** | Yes (same as Charm) |
| **USB-C** | Yes (same as Charm) |
| **microSD door** | Yes (microSD variant) |
| **Button + LED** | Yes (same as Charm) |
| **Antenna** | IPEX connector + cable to ceramic patch along long edge or top face |

**Mechanical notes:**
- The 2S battery (~6 mm thick) is the primary thickness driver
- The IPEX antenna cable must route from the E22-WROOM module to the antenna position with minimal bending
- Antenna keep-out: 15 mm from metal components on the antenna end of the device
- Thermal management: The E22 generates more heat than the C5. An internal copper pour or foil heat spreader is recommended (see `docs/POWER_THERMAL.md`)
- The parametric `enclosure.scad` will need a `pro` variant parameter to generate this larger envelope

**Material:**
- Prototype: 3D print in PETG (heat tolerance near the charger/radio)
- Production: Injection molding (ABS or PC/ABS)

---

## E22-WROOM Module Details

| Attribute | Value |
|---|---|
| **Module** | ESP32-E22-WROOM (projected) |
| **Dimensions** | 22 × 30 mm (with IPEX connector) |
| **Processor** | Dual-core RISC-V @ 500 MHz |
| **Wi-Fi** | Wi-Fi 6E (2.4, 5, 6 GHz) |
| **BLE** | BLE 5.4 |
| **Antenna** | External IPEX connector (cable to ceramic/PCB patch) |
| **GPIO** | TBD (datasheet pending release) |
| **Status** | **Roadmap.** Module not yet released. Datasheets unavailable. |

**Do not fabricate Pro variants until:**
1. The E22-WROOM hardware is procured
2. Footprints are verified against the official datasheet
3. The GPIO map is reconciled (PIN_MAP.md must be updated)
4. The IPEX antenna placement and routing are validated with a network analyzer

---

## Power Tree (Pro)

```
USB-C VBUS (5V)
    │
    ├─► BQ25887 (2S Boost Charger)
    │       │
    │       ├── BAT (8.4V max) ──► 2S LiPo Pack (BT1)
    │       │                        │
    │       │                        ├── BQ27421/BQ27520 (Fuel Gauge, I²C)
    │       │                        │       └── Sense Resistor
    │       │                        │
    │       │                        └── PCM (Protection + Balancing)
    │       │
    │       └── SYS (Power-Path Output, ~7.4V nominal)
    │               │
    │               └─► TPS62840-class Buck (8.4V → 3.3V, ≥2A)
    │                       │
    │                       └── 3.3V Rail
    │                               │
    │                               ├── ESP32-E22-WROOM (U1)
    │                               ├── GL3224 USB-HS Bridge (U2)
    │                               ├── SD Bus Mux (U6)
    │                               ├── Storage (J2/XU7)
    │                               └── Pull-ups, LED, etc.
    │
    └── GL3224 (USB Data Bridge, owns storage when plugged in)
```

---

## Pro Variant Matrix

When the E22-WROOM is released, the following Pro variants will be created with the same `standard/` + `alternatives/` folder structure as the Charm line:

| # | Radio | Storage | Battery | Wireless | Wired | Envelope | License |
|---|---|---|---|---|---|---|---|
| P1 | Single E22-WROOM | microSD | 2S LiPo | Wi-Fi 6E | USB-HS (GL3224) | 35×28×12–13 mm | CC BY-NC-SA 4.0 / PolyForm NC |
| P2 | Single E22-WROOM | eMMC | 2S LiPo | Wi-Fi 6E | USB-HS (GL3224) | 35×28×12–13 mm | CC BY-NC-SA 4.0 / PolyForm NC |
| P3 | Dual E22-WROOM | microSD | 2S LiPo | Wi-Fi 6E AGG | USB-HS (GL3224) | 35×28×12–13 mm | CC BY-NC-SA 4.0 / PolyForm NC |
| P4 | Dual E22-WROOM | eMMC | 2S LiPo | Wi-Fi 6E AGG | USB-HS (GL3224) | 35×28×12–13 mm | CC BY-NC-SA 4.0 / PolyForm NC |

**No Pro variant folders are created at this time.** This document captures the design IP for when the hardware becomes available.

---

## Standard and Alternative Paths (Pro)

Like the Charm line, each Pro variant will have organized `standard/` and `alternatives/` subfolders:

```
variants/pro-e22-wroom-microsd/
├── SPEC.md
├── PIN_MAP.md
├── block_diagram.svg
├── standard/
│   ├── BOM.csv             (GL3224, BQ25887, BQ27421, TPS62840)
│   ├── SCHEMATIC_PLAN.md
│   └── tucklet-pro-e22-wroom-microsd.net
└── alternatives/
    ├── usb_bridge_gl823/   (USB 2.0 backup)
    ├── usb_bridge_pl2732/  (USB 3.0 backup)
    ├── charger_discrete/   (MCP1642B + BQ7791502 discrete path)
    ├── fuel_gauge_bq27520/ (Alternative gauge)
    └── sd_mux_tmux1574/   (Signal integrity upgrade)
```

---

## Certification Requirements

- **FCC/CE:** Required (radios present). The E22-WROOM is a pre-certified module, which reduces but does not eliminate the burden.
- **Wi-Fi Alliance:** Required if shipping Wi-Fi Aware on iOS.
- **UN38.3:** Required for 2S LiPo battery transport/shipping.
- **Battery safety:** The 2S pack carries higher energy density; transport/shipping regulations are stricter and more expensive to certify for small devices.

---

## Thermal Management

The E22 dual-core 500 MHz processor + Wi-Fi 6E radio generates significantly more heat than the C5 in a sealed enclosure. Mitigations:

1. **Internal copper pour/foil** as a heat spreader during sustained transfer
2. **Firmware throttling:** Monitor internal temperature sensors and reduce throughput if limits are approached
3. **Thermal vias** under the E22 module to inner copper planes
4. **Larger battery** acts as a partial heat sink
5. **Mandatory thermal soak test** on prototype: charging + sustained transfer at high ambient

---

## Licensing Specificity (Pro)

The same copyright principles apply to the Pro line as the Charm line. The specific implementations captured in Pro variant folders are protectable:

1. **2S Power Architecture:** The specific circuit topology using the BQ25887 boost charger + BQ27421 gauge + sense resistor + TPS62840 buck to deliver sustained E22 performance from a 2S pack.
2. **IPEX Antenna Integration in a Charm Envelope:** The specific antenna placement, cable routing, and keep-out strategy required to make external-antenna Wi-Fi 6E work in a 35×28×12 mm wearable.
3. **Pro Enclosure:** The parametric mechanical design that integrates a 2S battery, IPEX antenna, storage, and thermal management into a Pro Charm envelope.

These are specific design expressions, not generic concepts, and are covered under the CC BY-NC-SA 4.0 license.

---

## Bring-up Checklist (Pro)

- [ ] Procure ESP32-E22-WROOM module (when released)
- [ ] Verify footprint against official datasheet
- [ ] Reconcile GPIO map (update PIN_MAP.md)
- [ ] Validate IPEX antenna placement with network analyzer
- [ ] Validate 2S charge/discharge cycle on BQ25887 + BQ27421
- [ ] Measure real E22 TX current; confirm 2S cell sizing
- [ ] Thermal soak test: charging + sustained transfer at high ambient
- [ ] FCC/CE certification (radios present)
- [ ] UN38.3 battery transport certification

---

*"Pro" is a Tucklet product line designation, not a separate company or entity. All Pro design IP is covered under the same source-available, non-commercial license as the Charm line.*
