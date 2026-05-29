// ============================================================================
// Tucklet enclosure — parametric, both storage variants.
// License: CC BY-NC-SA 4.0 (see /LICENSE-HARDWARE.txt)
//
// Renders a two-part charm enclosure (base + snap lid) dimensioned from the
// envelope in docs/TRANSFER_PERFORMANCE.md / hardware/VARIANTS.md:
//   microSD variant ~ 35 x 28 x 9 mm   |   eMMC variant ~ 32 x 24 x 8 mm
//
// This is dimensioned from the DOCUMENTED envelope. Before cutting plastic,
// set the cutout offsets to your actual KiCad board placement (the connector
// X/Y positions), then re-render. Everything here is a real, editable solid.
//
// Usage (headless):
//   openscad -D 'variant="microsd"' -D part=0 -o base_microsd.stl  enclosure.scad
//   openscad -D 'variant="microsd"' -D part=1 -o lid_microsd.stl   enclosure.scad
//   openscad -D 'variant="emmc"'    -D part=0 -o base_emmc.stl     enclosure.scad
//   openscad -D 'variant="emmc"'    -D part=1 -o lid_emmc.stl      enclosure.scad
// ============================================================================

// ---- parameters ------------------------------------------------------------
variant = "microsd";      // "microsd" | "emmc"
part    = 0;              // 0 = base, 1 = lid, 2 = both (preview)
$fn     = 64;

wall      = 1.6;          // wall thickness
floor_th  = 1.4;          // floor/ceiling thickness
fillet    = 2.5;          // outer corner radius
lid_h     = 3.0;          // lid height (cap)
gap       = 0.15;         // print clearance for the snap fit

// Envelope per variant (outer X, Y, total Z).
outer_x   = (variant == "emmc") ? 32 : 35;
outer_y   = (variant == "emmc") ? 24 : 28;
outer_z   = (variant == "emmc") ? 8  : 9;

base_h    = outer_z - lid_h;

// Connector / feature offsets — SET THESE FROM YOUR BOARD LAYOUT.
usbc_w    = 9.2;          // USB-C receptacle opening width
usbc_h    = 3.6;          // opening height
usbc_y    = outer_y/2;    // centered on the short edge (edit to board)
btn_d     = 3.4;          // button plunger hole dia
led_d     = 2.2;          // LED light-pipe hole dia
sd_slot_w = 13.0;         // microSD door opening width (microsd variant only)
sd_slot_h = 2.0;          // microSD door opening height
lanyard_d = 2.6;          // lanyard hole diameter
lanyard_boss = 5.2;       // lanyard boss outer diameter

// ---- helpers ---------------------------------------------------------------
module rrect(x, y, r, h) {
    linear_extrude(height = h)
        offset(r = r) offset(r = -r)
            square([x, y], center = true);
}

// Hollow shell: outer rrect minus inner cavity.
module shell(h, inner_floor) {
    difference() {
        rrect(outer_x, outer_y, fillet, h);
        translate([0, 0, inner_floor])
            rrect(outer_x - 2*wall, outer_y - 2*wall, max(fillet - wall, 0.6),
                  h - inner_floor + 0.1);
    }
}

// Lanyard loop: a small boss with a through hole on the short edge.
module lanyard_loop() {
    translate([-outer_x/2 + 1, 0, base_h/2]) {
        difference() {
            union() {
                rotate([90,0,0]) cylinder(d = lanyard_boss, h = 3.0, center = true);
                translate([-1.5,0,0]) cube([3, 3.0, lanyard_boss], center = true);
            }
            rotate([90,0,0]) cylinder(d = lanyard_d, h = 6, center = true);
        }
    }
}

// ---- base ------------------------------------------------------------------
module base() {
    difference() {
        union() {
            shell(base_h, floor_th);
            lanyard_loop();
        }
        // USB-C opening on +X short edge
        translate([outer_x/2 - wall - 0.1, usbc_y - outer_y/2, floor_th + 0.6])
            cube([wall + 0.6, usbc_w, usbc_h], center = false);
        // re-center the USB cut properly (front edge, centered Y)
        translate([outer_x/2, 0, floor_th + usbc_h/2 + 0.6])
            cube([wall*3, usbc_w, usbc_h], center = true);
        // microSD access slot (microsd variant) on -X short edge
        if (variant == "microsd") {
            translate([-outer_x/2, 0, floor_th + sd_slot_h/2 + 0.8])
                cube([wall*3, sd_slot_w, sd_slot_h], center = true);
        }
    }
}

// ---- lid -------------------------------------------------------------------
module lid() {
    difference() {
        union() {
            // outer cap
            rrect(outer_x, outer_y, fillet, lid_h);
            // inner lip that fits inside the base walls (snap)
            translate([0,0,-1.4])
                rrect(outer_x - 2*wall - 2*gap, outer_y - 2*wall - 2*gap,
                      max(fillet - wall, 0.6), 1.4);
        }
        // hollow the cap
        translate([0,0,floor_th])
            rrect(outer_x - 2*wall, outer_y - 2*wall, max(fillet-wall,0.6), lid_h);
        // button hole + LED light-pipe hole on the top face
        translate([outer_x/2 - 7, outer_y/2 - 6, -0.1])
            cylinder(d = btn_d, h = lid_h + 0.4);
        translate([outer_x/2 - 7, -outer_y/2 + 6, -0.1])
            cylinder(d = led_d, h = lid_h + 0.4);
    }
}

// ---- render ----------------------------------------------------------------
if (part == 0) base();
else if (part == 1) translate([0,0,base_h + 4]) lid();
else { base(); translate([0,0,base_h + 6]) lid(); }
