# Mechanical Design

The enclosure is delivered as a real parametric model — `enclosure.scad` (OpenSCAD) — not a dead mesh. You edit parameters and re-render STLs for any variant. Verified: it renders to valid STL for all three storage types and form factors (base + lid) with OpenSCAD.

## Render
```bash
# Standard WROOM Variants
openscad -D 'variant="microsd"' -D part=0 -o base_microsd.stl enclosure.scad
openscad -D 'variant="microsd"' -D part=1 -o lid_microsd.stl  enclosure.scad
openscad -D 'variant="emmc"'    -D part=0 -o base_emmc.stl     enclosure.scad
openscad -D 'variant="emmc"'    -D part=1 -o lid_emmc.stl      enclosure.scad

# Pro Variant (E22-WROOM + 2S LiPo)
openscad -D 'variant="pro"'     -D part=0 -o base_pro.stl      enclosure.scad
openscad -D 'variant="pro"'     -D part=1 -o lid_pro.stl       enclosure.scad
```
(`part=2` previews base+lid together.)

*Note: Mini variants (ESP32-C5-MINI-1 / ESP32-E22-MINI-1) fit comfortably within the eMMC envelope parameters. The parametric script can be adjusted to reduce length by ~6mm for dedicated Mini designs, but the standard "eMMC" envelope provides a unified shell for both WROOM-eMMC and MINI-eMMC builds.*

## What it makes
A two-part charm: a hollow base shell with a snap lid, a lanyard loop on the short edge (the "string" attach point), a USB-C opening, a microSD access slot (microSD variant only), and top-face holes for the button plunger and an LED light-pipe.

The design philosophy is **"Charm-First"**: All dimensions are constrained to the AirTag class for the Charm line, and to the Pro Charm class for the Pro line. The Pro variant (~35 x 28 x 12-13 mm) exceeds the AirTag thickness but remains a "Pro Charm" — it clips to your phone, hangs on a string, and disappears into your daily carry. It does not support oversized modules (like the ESP32-E22-M2) to ensure the product remains invisible and wearable.

## Envelopes (from VARIANTS.md, reconfirmed)
Dimensions are driven by two factors: **Storage Type** (width) and **Module Length** (height/length).

- **microSD Envelope:** ~35 x 28 x 9 mm
  *   *Drivers:* Width defined by microSD socket; Length defined by **ESP32-C5-WROOM-1** (27.5mm) + USB-C connector + clearance.
- **eMMC Envelope:** ~32 x 24 x 8 mm
  *   *Drivers:* Width reduced (no socket); Length defined by **ESP32-C5-WROOM-1**.
  *   *Mini Advantage:* **ESP32-C5-MINI-1** (21.3mm) builds allow for smaller custom enclosures (approx ~28mm length) or increased battery capacity within this same shell.
- **Pro Envelope:** ~35 x 28 x 12-13 mm
  *   *Drivers:* Width and length defined by **ESP32-E22-WROOM** module (22 x 30 mm) + IPEX antenna clearance; Thickness defined by **2S LiPo** battery (~6mm) + components.
  *   *Pro Charm Identity:* This is still a "Charm" — it clips to your phone, hangs on a string, and disappears into your daily carry. The thickness is justified by the performance (Wi-Fi 6E throughput and sustained high-current operation) that the 1S Charm line cannot deliver. The Pro line is fully covered under the **same source-available, non-commercial license** (CC BY-NC-SA 4.0 / PolyForm Noncommercial 1.0.0) as the Charm line.

Both the Charm and Pro envelopes sit in the charm class. The thickness is primarily driven by the battery (1S or 2S) and USB-C receptacle height.

## Before cutting plastic (the one real fit step)
The connector/feature offsets (`usbc_y`, button/LED positions, microSD slot, IPEX slot) are dimensioned to the documented envelope. Set them to your actual KiCad board placement — i.e. the real X/Y of the USB-C receptacle, the button/LED, and (for Pro variants) the IPEX antenna cable exit — then re-render.

**Critical Check:**
- **WROOM Variants:** Ensure clearance for the full 27.5mm module length.
- **MINI Variants:** The smaller module (15.4mm width, 21.3mm length) provides extra internal volume. Verify antenna clearance (keep metal objects away from the PCB antenna area on the end of the module).
- **Pro Variants:** Ensure clearance for the full 30mm E22-WROOM module length along the X-axis. The IPEX antenna cable must route from the module to the antenna position with minimal bending. Antenna keep-out: 15 mm from metal components on the antenna end of the device.

This is the only step that needs your finished board; everything else is parametric and done.

## Material & process
- **Prototype:** 3D print in PETG (heat tolerance near the charger/radio) or tough PLA for early fit checks.
- **Production:** Injection molding (ABS or PC/ABS). Mold tooling is separate capital (~$3–10k for a small mold), amortized over units — not part of per-unit BOM.
- **Thermal Management:** Optional internal copper pour / foil as a heat spreader during sustained transfer (see `docs/POWER_THERMAL.md`); validate with a thermal soak on a prototype. Note that sealed eMMC variants and Pro variants may trap more heat than vented microSD variants. The Pro enclosure includes heat vent slots for thermal relief during sustained transfer.

## CAD round-trip
This native parametric source is the source of truth; export STEP/STL for manufacturing. (This follows the deconstruct/reconstruct workflow you established: keep the parametric recipe, not just the baked geometry.)

## Tolerances
- Snap-fit clearance `gap = 0.15 mm` (tune to your printer/process).
- Wall 1.6 mm, floor/ceiling 1.4 mm — adjust for material and drop resistance.
- Lanyard boss: Designed for 2.6mm cord/loop; verify pull-strength if using weaker materials like standard PLA.

## Pro Charm Identity & Licensing

The Pro variant (~35 x 28 x 12-13 mm) exceeds the AirTag-class thickness but remains a **"Pro Charm"** — it clips to your phone, hangs on a string, and disappears into your daily carry. It is not a "backpack" or "brick" class device. The thickness is justified by the performance: Wi-Fi 6E throughput and sustained high-current operation that the 1S Charm line cannot deliver.

The Pro line is fully covered under the **same source-available, non-commercial license** as the Charm line:
- **Hardware:** CC BY-NC-SA 4.0 (see `/LICENSE-HARDWARE.txt`)
- **Software:** PolyForm Noncommercial 1.0.0 (see `/LICENSE-SOFTWARE.txt`)

You may study, modify, and build Pro variants for personal use; commercial sale requires a separate agreement. The Pro design IP is documented in [`hardware/PRO_LINE.md`](hardware/PRO_LINE.md).
