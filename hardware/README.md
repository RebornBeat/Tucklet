# Tucklet Hardware

This directory holds the complete electrical design for every Tucklet board
variant. It is organized so that "what is what" is obvious:

```
hardware/
├── README.md                 # you are here
├── gen_hardware.py           # the parametric SOURCE — regenerates every variant
├── common/                   # shared, variant-independent design
│   ├── COMPONENT_SELECTION.md   # the shared parts + why (ESP32-C5 era)
│   ├── VARIANTS.md              # cross-variant comparison + BOM/dimension overview
│   ├── block_diagram.svg        # system block diagram (all variants)
│   └── kicad/                   # (place your KiCad project + symbol/footprint libs here)
└── variants/                 # one folder per PHYSICAL board variant
    ├── singlec5-microsd/
    ├── singlec5-emmc/
    ├── dualc5-microsd/
    └── dualc5-emmc/
```

## The four physical board variants

Two hardware axes multiply into four boards. (The third axis — transport:
SoftAP / Wi-Fi Aware / wired — is a **firmware feature flag plus the USB-HS
bridge that every board carries**, not a board change. See
`../docs/VARIANT_MATRIX.md`.)

| Variant | Radio | Storage | Full BOM | Position |
|---|---|---|---|---|
| `singlec5-microsd` | 1× ESP32-C5 | microSD | ~$13.66 | baseline: cheapest, swappable |
| `singlec5-emmc`    | 1× ESP32-C5 | eMMC    | ~$13.5–$33 by capacity | sealed/slim premium |
| `dualc5-microsd`   | 2× ESP32-C5 | microSD | ~$16+ | speed-focused, swappable |
| `dualc5-emmc`      | 2× ESP32-C5 | eMMC    | ~$19+ by capacity | top of line |

Each `variants/<name>/` folder contains, all generated and mutually consistent:

- `SPEC.md` — what the board is, performance, dimensions, bring-up checklist.
- `BOM.csv` — full bill of materials (electronics + PCB + enclosure) with totals.
- `PIN_MAP.md` — signal → ESP32-C5 pin reconciliation (the one datasheet step).
- `SCHEMATIC_PLAN.md` — net-by-net connection plan.
- `tucklet-<name>.net` — **KiCad-importable netlist** (Pcbnew → File → Import Netlist).
- `block_diagram.svg` — that variant's block diagram.

## How to use these (your KiCad workflow)

You have two equally valid paths, matching the deconstruct/reconstruct approach
you use for CAD:

**A. Reconstruct from the netlist (fastest).** In KiCad's Pcbnew, *File → Import
Netlist*, pick `tucklet-<variant>.net`. KiCad creates the components and a
ratsnest. The netlist is a *logical* netlist: the ESP32-C5 module's pins are
referenced by **signal name** (e.g. `SD_CLK`), so when you place the C5 symbol
you name its pins to match — `PIN_MAP.md` is the reconciliation table. Fixed
ICs (charger, gauge, USB-C, CC resistors) already use real pin numbers.

**B. Build the schematic by hand (your usual flow).** Place the symbols listed
in `BOM.csv`, then wire the nets exactly as `SCHEMATIC_PLAN.md` lists them.
Use `PIN_MAP.md` to assign the C5 GPIOs. Export your own netlist when done and
iterate from there.

## Regenerating everything (the recipe)

`gen_hardware.py` is the single source of truth. Edit it and re-run to
regenerate all four variants consistently — never hand-edit the generated files,
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
- **ESP32-C5 GPIO numbers are not invented.** They must be read from the current
  ESP32-C5-WROOM-1 datasheet (the C5 map differs from the S3, and the strapping
  pins must stay clean). `PIN_MAP.md` flags this as the one reconciliation step.
- **eMMC costs are small-volume estimates.** Verify on LCSC/Mouser/Arrow;
  industrial/pSLC grades cost more.
- **Certification is real money + weeks.** FCC/CE always (radios present);
  Wi-Fi Alliance only if you ship the Wi-Fi Aware transport on iOS. Build the
  SoftAP prototype first and prove demand before spending on Aware certification.

See `common/COMPONENT_SELECTION.md` for the shared parts rationale and
`common/VARIANTS.md` for the cross-variant comparison and dimensions.
