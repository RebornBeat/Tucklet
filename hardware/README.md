# Tucklet Hardware

This directory holds the complete electrical design for every Tucklet board
variant. It is organized so that "what is what" is obvious:

```
hardware/
├── README.md                 # you are here
├── gen_hardware.py           # the parametric SOURCE — regenerates every variant
├── PRO_LINE.md               # Pro line (E22-WROOM + 2S) — source-available, non-commercial
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
| **Pro High-Perf** | ESP32-E22 | WROOM | Roadmap | Larger: Wi-Fi 6E speed, 2S battery, pending module release |

Each `variants/<name>/` folder contains, all generated and mutually consistent:

- `SPEC.md` — what the board is, performance, dimensions, bring-up checklist.
- `BOM.csv` — full bill of materials (electronics + PCB + enclosure) with totals.
- `PIN_MAP.md` — signal → MCU pin reconciliation (the one datasheet step).
- `SCHEMATIC_PLAN.md` — net-by-net connection plan.
- `tucklet-<name>.net` — **KiCad-importable netlist** (Pcbnew → File → Import Netlist).
- `block_diagram.svg` — that variant's block diagram.

## Variant Folder Organization (Standard vs. Alternatives)

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
The `ESP32-E22-WROOM` (22×30 mm module with external IPEX antenna) is explicitly excluded from the Charm variants due to size and antenna constraints. However, this module is the basis for the **Pro Line**, a larger, higher-performance variant with a 2S LiPo battery pack.

The Pro design IP is documented in [`hardware/PRO_LINE.md`](hardware/PRO_LINE.md). Like the Charm line, the Pro line is fully covered under the **CC BY-NC-SA 4.0** (hardware) and **PolyForm Noncommercial 1.0.0** (software) licenses. You may study, modify, and build for personal use; commercial sale requires a separate agreement.

**No Pro variant folders are created at this time.** When the E22-WROOM datasheet is released and the Pro line is ready for prototyping, variant folders (e.g., `pro-e22-wroom-microsd/`) will be added with the same structure, under the same source-available, non-commercial license as the Charm line.

## How to use these (your KiCad workflow)

You have two equally valid paths, matching the deconstruct/reconstruct approach
you use for CAD:

**A. Reconstruct from the netlist (fastest).** In KiCad's Pcbnew, *File → Import
Netlist*, pick `tucklet-<variant>.net` from the `standard/` or appropriate `alternatives/` subfolder. KiCad creates the components and a
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
