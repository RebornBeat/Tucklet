# Tucklet Hardware

This directory holds the complete electrical design for every Tucklet board
variant. It is organized so that "what is what" is obvious:

```
hardware/
├── README.md                 # you are here
├── gen_hardware.py           # the parametric SOURCE — regenerates every variant
├── common/                   # shared, variant-independent design
│   ├── COMPONENT_SELECTION.md   # the shared parts + why (ESP32-C5 / E22 era)
│   ├── VARIANTS.md              # cross-variant comparison + BOM/dimension overview
│   ├── block_diagram.svg        # system block diagram (all variants)
│   └── kicad/                   # (place your KiCad project + symbol/footprint libs here)
└── variants/                 # one folder per PHYSICAL board variant
    ├── singlec5-wroom-microsd/
    ├── singlec5-wroom-emmc/
    ├── dualc5-wroom-microsd/
    ├── dualc5-wroom-emmc/
    ├── singlec5-mini-microsd/
    ├── singlec5-mini-emmc/
    ├── dualc5-mini-microsd/
    ├── dualc5-mini-emmc/
    ├── singlee22-mini-microsd/
    ├── singlee22-mini-emmc/
    ├── duale22-mini-microsd/
    └── duale22-mini-emmc/
```

## The Variants (Charm Strategy)

Tucklet variants are defined by three axes: **Radio**, **Storage**, and **Form Factor**. The generator enforces a "Charm-Only" strategy, excluding oversized modules to ensure every board fits the intended product identity.

### Axis 1: Radio
- **Single C5:** Baseline Wi-Fi 6 (2.4/5 GHz).
- **Dual C5:** Link-aggregated (experimental).
- **Single/Dual E22:** Wi-Fi 6E High-Performance (Roadmap).

### Axis 2: Storage
- **microSD:** Swappable, user-supplied.
- **eMMC:** Sealed, fixed capacity.

### Axis 3: Form Factor
- **WROOM (Standard):** 18.0 x 27.5 mm module.
- **MINI (Compact):** 15.4 x 21.3 mm module (enables smaller enclosures).
- **Constraint:** E22 variants are restricted to **MINI** form factor (or compatible future modules) to maintain the "Charm" envelope; oversized M.2 variants are excluded.

## The Product Lines

| Line | Radio | Form Factor | Variants | Position |
|---|---|---|---|---|
| **Charm Standard** | ESP32-C5 | WROOM | 4 | Baseline: Lowest cost, easy assembly |
| **Charm Compact** | ESP32-C5 | MINI | 4 | Nano: Smallest footprint, tighter layout |
| **Charm High-Perf** | ESP32-E22 | MINI | 4 | Roadmap: Wi-Fi 6E speed, pending module release |

Each `variants/<name>/` folder contains, all generated and mutually consistent:

- `SPEC.md` — what the board is, performance, dimensions, bring-up checklist.
- `BOM.csv` — full bill of materials (electronics + PCB + enclosure) with totals.
- `PIN_MAP.md` — signal → MCU pin reconciliation (the one datasheet step).
- `SCHEMATIC_PLAN.md` — net-by-net connection plan.
- `tucklet-<name>.net` — **KiCad-importable netlist** (Pcbnew → File → Import Netlist).
- `block_diagram.svg` — that variant's block diagram.

## How to use these (your KiCad workflow)

You have two equally valid paths, matching the deconstruct/reconstruct approach
you use for CAD:

**A. Reconstruct from the netlist (fastest).** In KiCad's Pcbnew, *File → Import
Netlist*, pick `tucklet-<variant>.net`. KiCad creates the components and a
ratsnest. The netlist is a *logical* netlist: the MCU module's pins are
referenced by **signal name** (e.g. `SD_CLK`), so when you place the symbol
you name its pins to match — `PIN_MAP.md` is the reconciliation table. Fixed
ICs (charger, gauge, USB-C, CC resistors) already use real pin numbers.

**B. Build the schematic by hand (your usual flow).** Place the symbols listed
in `BOM.csv`, then wire the nets exactly as `SCHEMATIC_PLAN.md` lists them.
Use `PIN_MAP.md` to assign the GPIOs. Export your own netlist when done and
iterate from there.

## Regenerating everything (the recipe)

`gen_hardware.py` is the single source of truth. Edit it and re-run to
regenerate all variants consistently — never hand-edit the generated files,
or they drift apart.

```
python3 gen_hardware.py          # regenerate variants/ + validate
python3 gen_hardware.py --check  # generate to ./_check and validate only
```

The built-in validator checks every netlist has balanced parens, every net has
≥2 nodes, power/USB/SD/storage nets exist, and the dual-radio aggregation
crossover is correct.

## Honesty notes (so the files mean what they say)

- **No gerbers / pick-and-place here.** Those come *after* schematic capture +
  PCB layout + RF/antenna tuning + a design-for-manufacture review on real
  silicon. Generating them blind would waste a board run. The netlist + BOM +
  pin map are exactly what you turn into a layout.
- **Exact Naming:** The generator uses official module names (`ESP32-C5-WROOM-1`,
  `ESP32-C5-MINI-1`, `ESP32-E22-MINI-1`). GPIO maps must be read from the specific
  datasheet for that exact model (the MINI pinout differs from the WROOM).
- **E22 Roadmap Status:** The `ESP32-E22-MINI-1` is a projected component. While
  the netlist and architecture are valid, the module is not yet released. Do not
  fabricate E22 variants until the hardware is procured and footprints are verified.
- **eMMC costs are small-volume estimates.** Verify on LCSC/Mouser/Arrow;
  industrial/pSLC grades cost more.
- **Certification is real money + weeks.** FCC/CE always (radios present);
  Wi-Fi Alliance only if you ship the Wi-Fi Aware transport on iOS. Build the
  SoftAP prototype first and prove demand before spending on Aware certification.

See `common/COMPONENT_SELECTION.md` for the shared parts rationale and
`common/VARIANTS.md` for the cross-variant comparison and dimensions.
