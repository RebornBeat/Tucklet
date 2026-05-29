# Hardware Variants — microSD vs eMMC, BOM & Dimensions

Two product variants share one PCB design philosophy and differ only in the storage block. Everything else (ESP32-S3 module, USB-C charge, charger, fuel gauge, regulator, LiPo, button, LED, passives) is identical.

## Shared electronics (both variants)

| Block | Part | ~Cost |
|---|---|---|
| SoC/radio | ESP32-S3-WROOM-1-N8R2 | 2.75 |
| USB-C + ESD + CC resistors | receptacle + protection | 0.50 |
| Charger | MCP73831 | 0.50 |
| Fuel gauge | MAX17048 | 0.80 |
| Regulator | 3.3V buck | 0.40 |
| Battery | LiPo ~150 mAh w/PCM | 2.00 |
| Button + RGB LED | tactile + WS2812 | 0.30 |
| Passives | decoupling/pullups/misc | 0.80 |
| **Shared subtotal** | | **~$8.05** |

Plus PCB (~$1.50 volume) and enclosure (~$1.50–4.00 depending on 3D-print vs molded).

---

## Variant A — microSD (swappable, low entry price)

Add a push-push microSD socket (~$0.40). Customer chooses/swaps the card; you don't pay for the flash.

| | |
|---|---|
| Storage block | microSD socket +$0.40 |
| **Electronics BOM** | **~$8.45** |
| **Full BOM (incl. PCB + 3D-print shell)** | **~$11.50–14.00** (card extra or bundled) |
| Pros | Cheapest to make, customer-upgradable, dead card = $5 swap not dead device |
| Cons | Socket + insertion clearance makes the enclosure slightly larger |

---

## Variant B — eMMC (sealed, slim, premium)

Replace the socket with a soldered eMMC (153-ball TFBGA, **11.5 × 13.0 × 1.2 mm**, JEDEC eMMC 5.1). Smaller and cleaner than a socket; capacity fixed at manufacture.

eMMC cost rises with capacity (flash costs money — this is why only the small capacities stay near a $15 BOM). Prices below are **commercial-grade small-volume estimates — VERIFY on LCSC/Mouser/Arrow; industrial/pSLC grades cost significantly more.**

| Capacity | eMMC est. | Electronics BOM (8.05 + eMMC) | Full BOM (+PCB +shell ~$3) |
|---|---|---|---|
| 8 GB | ~$2.50 | ~$10.55 | ~$13.5 |
| 16 GB | ~$3.50 | ~$11.55 | ~$14.5 |
| 32 GB | ~$5.00 | ~$13.05 | ~$16.0 |
| 64 GB | ~$8.00 | ~$16.05 | ~$19.0 |
| 128 GB | ~$13.00 | ~$21.05 | ~$24.0 |
| 256 GB | ~$22.00 | ~$30.05 | ~$33.0 |

**Reading the table:** only 8–16 GB eMMC stays inside the sub-$15 BOM target. That's the honest cost of integrated flash. This is exactly why offering BOTH variants is smart: microSD keeps the entry price low and pushes storage cost onto the customer's card; eMMC is the sealed/slim premium SKU where the price reflects the built-in capacity.

Pros: smaller, thinner, "invisible," nothing to lose or insert.
Cons: capacity fixed forever; higher BOM at higher capacities; requires BGA assembly (a fab/CM cost, not a hand-solder job).

---

## Dimensions & fit — confirmed for both variants

The largest single component sets the floor, and it is **not** the storage:

| Component | Footprint (approx) |
|---|---|
| ESP32-S3-WROOM-1 module | 18 × 25.5 × 3.1 mm  ← size driver |
| microSD socket (Variant A) | 15 × 14 × 1.4 mm + insertion clearance |
| eMMC (Variant B) | 11.5 × 13 × 1.2 mm, no clearance needed |
| USB-C receptacle | ~9 × 7 × 3.2 mm |
| LiPo ~150 mAh | varies (e.g. ~30 × 20 × 5 mm) ← thickness driver |
| Charger / gauge / buck / passives | a few mm each |

### Enclosure envelopes (target)
- **Variant A (microSD):** ~35 × 28 × 8–10 mm
- **Variant B (eMMC):** ~32 × 24 × 7–8 mm (smaller — no socket, no insertion slot, thinner storage)

Both sit inside an **AirTag-class envelope** (AirTag is ~32 mm round × 8 mm). That satisfies the "charm on a string / forgettable on the back of the phone / doesn't block the charging port" concept. The eMMC variant is the one to market as the slim, sealed, most-invisible option.

### Notes that affect fit
- Battery thickness is the main variable; finalize after measuring real transfer-current draw and choosing the cell.
- The microSD slot should be **internal behind a small door** so the device still looks sealed day-to-day (per ADR-001).
- BGA eMMC needs a CM with BGA reflow + (ideally) X-ray inspection; budget that into assembly, not BOM.
- Both variants reuse the same module, USB-C, PMIC, and UI — so the PCB is one design with a depopulate/swap on the storage footprint, or two near-identical board revs. Keeping them pin-compatible saves a second certification effort.

---

## SoC / Layer-1 radio options (added after the chipset review)

The storage variant (microSD vs eMMC) is independent of the radio choice. Both storage variants work with any radio below. See `docs/TRANSFER_PERFORMANCE.md` for the full analysis.

| Radio option | Real wireless throughput | Module size | ~Cost | Fits charm envelope? |
|---|---|---|---|---|
| ESP32-S3 (original plan) | ~2–5 MB/s (2.4GHz) | 18 × 25.5 × 3.1 mm | ~$2.75 | Yes |
| **ESP32-C5 (recommended)** | ~4–9 MB/s (adds 5GHz WiFi6) | ~comparable to S3 | ~$2–3 | Yes |
| Dual ESP32-C5 (experimental) | ~13–16 MB/s | 2 modules + 2 antennas | ~$5–6 | Tight; validate antenna isolation |
| Linux router SoC (FileHub-class) | ~6–50 MB/s | SoC + DDR + flash | ~$10–30 | No — exceeds charm size |

**Default: ESP32-C5.** Swaps in for the ESP32-S3 in COMPONENT_SELECTION.md at similar cost/size, adds the 5GHz band, keeps BLE.

## Layer-2 fast wired path (added)

Optional but it fits, so included: a USB 2.0 High-Speed card-reader/bridge IC (~$1–2, ~5mm QFN) gives ~20–40 MB/s wired by owning the storage directly when plugged in. Adds a storage mux/arbitration (wireless mode = SoC owns card; wired mode = bridge owns card). Negligible footprint; does not change the enclosure envelope.

Updated electronics subtotal with C5 + USB-HS bridge: shared ~$8.05 (S3) becomes roughly:
- swap S3->C5: about the same (~$8.05)
- add USB-HS bridge IC + mux: +~$1.50–2.50
- New shared subtotal: **~$9.5–10.5**, plus storage block (microSD socket $0.40, or eMMC per capacity), plus PCB + enclosure.

Still lands the microSD variant around **$13–16 BOM** and keeps the eMMC variant's per-capacity math from the tables above (shifted up ~$1.5–2.5 for the bridge). The sub-$15 BOM target holds for the microSD variant and the low-capacity eMMC variants; higher eMMC capacities exceed it as before (flash cost).

## Dimensions — final reconfirmation
- ESP32-C5 module footprint is comparable to the ESP32-S3 (the size driver is unchanged).
- USB-HS bridge IC + mux are a few mm — negligible.
- Both storage variants + C5 + bridge fit the AirTag-class envelope (~32–35 mm; eMMC variant ~32 × 24 mm, microSD variant ~35 × 28 mm).
- Battery remains the thickness driver. Finalize cell after measuring real transfer-current on a C5 prototype (5GHz TX draw differs from the S3 estimate).
