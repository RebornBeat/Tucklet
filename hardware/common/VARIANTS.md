# Variants — cross-comparison, BOM & dimensions

Two hardware axes (radio × storage) give four physical boards. The per-board
`variants/<name>/BOM.csv` and `SPEC.md` are the authoritative source (generated
from `gen_hardware.py`); this file is the at-a-glance comparison.

## The four boards

| Variant | Radio | Storage | Wireless | Full BOM | For |
|---|---|---|---|---|---|
| `singlec5-microsd` | 1× C5 | microSD | ~9 MB/s | ~$13.66 | entry / swappable |
| `singlec5-emmc` | 1× C5 | eMMC | ~9 MB/s | ~$13.5–$33 (by GB) | slim / sealed |
| `dualc5-microsd` | 2× C5 | microSD | ~13–16 MB/s | ~$16+ | speed / swappable |
| `dualc5-emmc` | 2× C5 | eMMC | ~13–16 MB/s | ~$19+ (by GB) | top of line |

Transport (SoftAP / Wi-Fi Aware / wired USB-HS) is a firmware feature flag plus
the bridge that *every* board carries — not a board change. See
`../../docs/VARIANT_MATRIX.md`.

## Storage axis

### microSD (swappable)
Push-push socket, SDIO 4-bit, +~$0.40. Customer supplies/upgrades the card; you
don't pay for flash. Dead card = $5 swap, not a dead device. Slightly larger
enclosure (socket + insertion clearance), placed internally behind a small door
so the unit still looks sealed.

### eMMC (sealed) — per-capacity cost (small-volume estimate; VERIFY)
153-ball TFBGA eMMC 5.1, 11.5 × 13.0 × 1.2 mm. Capacity fixed at manufacture.

| Capacity | eMMC est. | Full BOM (single-C5) |
|---|---|---|
| 8 GB | ~$2.50 | ~$13.5 |
| 16 GB | ~$3.50 | ~$14.5 |
| 32 GB | ~$5.00 | ~$16.0 |
| 64 GB | ~$8.00 | ~$19.0 |
| 128 GB | ~$13.00 | ~$24.0 |
| 256 GB | ~$22.00 | ~$33.0 |

Only 8–16 GB eMMC stays near the sub-$15 target — the honest cost of integrated
flash, and exactly why offering microSD *and* eMMC is smart. Requires BGA
assembly (CM with reflow + ideally X-ray).

## Radio axis

| Radio | Real wireless | Module size | Fits charm? | Notes |
|---|---|---|---|---|
| 1× ESP32-C5 | ~4–9 MB/s (5 GHz Wi-Fi 6) | ~18 × 25.5 mm | yes | recommended baseline |
| 2× ESP32-C5 | ~13–16 MB/s (aggregated) | 2 modules + 2 antennas | tight | **experimental** — validate two-antenna RF isolation on a prototype before finalizing the enclosure |

To go beyond ~9 MB/s on a single chip you must add a second radio or jump to a
Linux router SoC (which is a deck-of-cards "FileHub", not a charm). The C5 is the
balance point; the form factor itself maximizes its throughput (centimeter-range
link = top modulation). Full analysis in `../../docs/TRANSFER_PERFORMANCE.md`.

## Dimensions & fit (confirmed)

The ESP32-C5 module is the footprint driver; the battery is the thickness
driver. The USB-HS bridge is a few mm (negligible).

| Variant storage | Enclosure envelope | Class |
|---|---|---|
| microSD | ~35 × 28 × 9 mm | AirTag-class |
| eMMC | ~32 × 24 × 8 mm | AirTag-class (smaller — no socket/insertion slot) |

Both satisfy the "charm on a string / forgettable on the back of the phone /
doesn't block the charging port" concept. The parametric enclosure in
`../../mechanical/enclosure.scad` renders STLs for both storage envelopes and has
been verified to produce valid watertight geometry.

## One PCB philosophy

All four boards share the same module, USB-C, charger, gauge, buck, bridge, mux,
and UI. Keep the microSD and eMMC footprints pin-compatible (depopulate/swap the
storage block) so a single certification + a single end-of-line test jig covers
both. The dual-radio boards add the second module, the AGG crossover link, and a
larger buck/cell.
