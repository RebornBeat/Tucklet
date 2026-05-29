# Mechanical Design

The enclosure is delivered as a real parametric model — `enclosure.scad` (OpenSCAD) — not a dead mesh. You edit parameters and re-render STLs for either variant. Verified: it renders to valid STL for both variants (base + lid) with OpenSCAD.

## Render
```
openscad -D 'variant="microsd"' -D part=0 -o base_microsd.stl enclosure.scad
openscad -D 'variant="microsd"' -D part=1 -o lid_microsd.stl  enclosure.scad
openscad -D 'variant="emmc"'    -D part=0 -o base_emmc.stl     enclosure.scad
openscad -D 'variant="emmc"'    -D part=1 -o lid_emmc.stl      enclosure.scad
```
(`part=2` previews base+lid together.)

## What it makes
A two-part charm: a hollow base shell with a snap lid, a lanyard loop on the short edge (the "string" attach point), a USB-C opening, a microSD access slot (microSD variant only), and top-face holes for the button plunger and an LED light-pipe.

## Envelopes (from VARIANTS.md, reconfirmed)
- microSD: ~35 x 28 x 9 mm
- eMMC:    ~32 x 24 x 8 mm  (smaller — no socket/insertion slot, thinner storage)
Both sit in the AirTag class (~32 mm round x 8 mm). The ESP32-C5 module is the footprint driver; the battery is the thickness driver.

## Before cutting plastic (the one real fit step)
The connector/feature offsets (`usbc_y`, button/LED positions, microSD slot) are dimensioned to the documented envelope. Set them to your actual KiCad board placement — i.e. the real X/Y of the USB-C receptacle and the button/LED — then re-render. This is the only step that needs your finished board; everything else is parametric and done.

## Material & process
- Prototype: 3D print in PETG (heat tolerance near the charger/radio) or tough PLA for early fit checks.
- Production: injection molding (ABS or PC/ABS). Mold tooling is separate capital (~$3–10k for a small mold), amortized over units — not part of per-unit BOM.
- Optional internal copper pour / foil as a heat spreader during sustained transfer (see POWER_THERMAL.md); validate with a thermal soak on a prototype.

## CAD round-trip
This native parametric source is the source of truth; export STEP/STL for manufacturing. (This follows the deconstruct/reconstruct workflow you established: keep the parametric recipe, not just the baked geometry.)

## Tolerances
- Snap-fit clearance `gap = 0.15 mm` (tune to your printer/process).
- Wall 1.6 mm, floor/ceiling 1.4 mm — adjust for material and drop resistance.
