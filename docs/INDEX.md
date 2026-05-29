# Documentation Index

The map of the whole repository, and the honest status of each piece. Start at
the top.

## Decisions & architecture (the "why")
- **`FINAL_REVIEW.md`** — current source of truth: layered connectivity (BLE +
  Wi-Fi Aware + SoftAP), iOS reality, the invisible/one-time-pairing security
  model. **Supersedes the ADRs where they differ.**
- `VARIANT_MATRIX.md` — every build configuration (radio × storage × transport)
  and how one codebase serves them all.
- `TRANSFER_PERFORMANCE.md` — Layer-1 wireless deep dive, chipset spectrum, real
  throughput, the trickle-sync philosophy, the wired fast path.
- `adr/ADR-001…004` — original decision records (storage, connectivity, iOS,
  power/pairing). **Historical.** Where they say "ESP32-S3", the finalized radio
  is the **ESP32-C5** (see FINAL_REVIEW + hardware/common/COMPONENT_SELECTION.md);
  where they say "press to summon", the finalized model is "press once to pair,
  invisible thereafter" (FINAL_REVIEW §3).

## Product & experience
- `UX_SPEC.md` — the complete UI/UX: plain-language states, per-app
  organization, round-trip restore, the transfer screen with honest ETAs,
  trickle sync, invisible-after-pairing.
- `assets/flow.svg` — one-glance flow diagram (in the top-level README).

## Engineering — protocol & power
- `protocol/PROTOCOL.md` — the wire contract (BLE control + HTTP data), mirrored
  exactly by `firmware/crates/tucklet-proto`.
- `POWER_THERMAL.md` — power budget, battery safety, thermal limits, regulatory
  gates, prototype-only finalization items.

## Business / legal
- `PRIOR_ART_AND_LICENSE.md` — closest existing products and confirmation the
  non-commercial license is valid.
- `../LICENSING.md` — which license covers what; "source-available, not
  open-source" distinction.

## Hardware — build in KiCad from these
- `../hardware/README.md` — overview, the four variants, how to import the
  netlist or build by hand, how to regenerate.
- `../hardware/gen_hardware.py` — the parametric source (regenerates all variants).
- `../hardware/common/COMPONENT_SELECTION.md` — shared parts + rationale (C5 era).
- `../hardware/common/VARIANTS.md` — cross-variant comparison, capacities, dimensions.
- `../hardware/common/block_diagram.svg` — system block diagram.
- `../hardware/variants/<variant>/` — per board: `SPEC.md`, `BOM.csv`,
  `PIN_MAP.md`, `SCHEMATIC_PLAN.md`, `tucklet-<variant>.net` (KiCad netlist),
  `block_diagram.svg`. Variants: `singlec5-microsd`, `singlec5-emmc`,
  `dualc5-microsd`, `dualc5-emmc`.

## Mechanical
- `../mechanical/enclosure.scad` — parametric enclosure; renders watertight STLs
  for both storage envelopes (verified).
- `../mechanical/MECHANICAL.md` — render instructions, envelopes, the one
  board-dependent fit step, materials.

## Code — current state
| Component | Path | Status |
|---|---|---|
| Shared wire types | `../firmware/crates/tucklet-proto` | **Compiles + 5 tests pass** |
| Device logic (estimator, state machine, trickle, allow-list, expiry, transport) | `../firmware/crates/tucklet-core` | **Compiles + 13 tests pass** |
| On-device firmware binary | `../firmware/crates/tucklet-fw` | Next — esp-idf / ESP32-C5; builds on the two crates above |
| iOS app | `../software/ios` | Present (SwiftUI + File Provider); WiFiAware/AccessorySetupKit/Secure-Enclave call-sites flagged `CONFIRM` against the iOS 26 SDK |
| Android app | `../software/android` | Next |
| Desktop app | `../software/desktop` | Next |

## Roadmap order
1. ✅ Reorganize repo; finalize per-variant hardware; verify CAD + code.
2. Finalize remaining docs alignment (ADR supersession banners already noted here).
3. Complete the on-device firmware binary (`tucklet-fw`).
4. Complete the iOS app to full UX-spec functionality.
5. Add Android, then desktop, sharing the same protocol + estimator.
