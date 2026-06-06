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
- `../hardware/README.md` — overview, the variants, how to import the
  netlist or build by hand, how to regenerate.
- `../hardware/gen_hardware.py` — the parametric source (regenerates all variants).
- `../hardware/common/COMPONENT_SELECTION.md` — shared parts + rationale (C5 era + E22 roadmap).
- `../hardware/common/VARIANTS.md` — cross-variant comparison, capacities, dimensions.
- `../hardware/common/block_diagram.svg` — system block diagram.
- `../hardware/variants/<variant>/` — per board: `SPEC.md`, `BOM.csv`,
  `PIN_MAP.md`, `SCHEMATIC_PLAN.md`, `tucklet-<variant>.net` (KiCad netlist),
  `block_diagram.svg`.

  **Variants (Generated):**
  *Production (C5):*
  - `singlec5-wroom-microsd`
  - `singlec5-wroom-emmc`
  - `singlec5-mini-microsd`
  - `singlec5-mini-emmc`
  - `dualc5-wroom-microsd`
  - `dualc5-wroom-emmc`
  - `dualc5-mini-microsd`
  - `dualc5-mini-emmc`

  *Roadmap (E22-MINI Projected):*
  - `singlee22-mini-microsd`
  - `singlee22-mini-emmc`
  - `duale22-mini-microsd`
  - `duale22-mini-emmc`

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
| On-device firmware binary | `../firmware/crates/tucklet-fw` | **Present** — esp-idf/ESP32-C5; pure logic tested (ui 3, httpd 3), esp glue flagged CONFIRM |
| iOS app | `../software/ios` | **Present**, full UX-spec (PhotoKit, AccessorySetupKit pairing, trickle, restore, undo); pure logic verified; SDK call-sites flagged `CONFIRM` |
| Android app | `../software/android` | **Present**, full UX-spec (MediaStore, CompanionDeviceManager, programmatic Wi-Fi, WorkManager trickle); estimator+protocol compiled & tested with kotlinc; SDK call-sites flagged `CONFIRM` |
| Desktop app | `../software/desktop` | **Present**: wired = mounts as a USB drive (no app); wireless companion = Rust core + CLI **compiled & tested** (21 tests, incl. a live socket smoke test and an end-to-end enroll/handshake/data integration test against a mock device built from the shared crates) reusing the shared crates, plus a Tauri+React GUI shell flagged `CONFIRM` |

## Roadmap order
1. ✅ Reorganize repo; finalize per-variant hardware; verify CAD + code.
2. Finalize remaining docs alignment (ADR supersession banners already noted here).
3. ✅ On-device firmware binary (`tucklet-fw`) written; esp glue pending hardware bring-up.
4. ✅ iOS app to full UX-spec functionality.
5. ✅ Android app to full UX-spec functionality.
6. ✅ Desktop: wired mounts as a USB drive (no app); wireless companion (Rust core + CLI, compiled & tested) + Tauri/React GUI shell, reusing the shared protocol + estimator.

All three client platforms (iOS, Android, desktop) and the firmware now share one wire protocol and one transfer estimator.
7. ✅ Crypto seam closed: a shared, host-tested `tucklet-crypto` crate (Ed25519, RFC 8032) does the device verify + desktop signing; iOS uses CryptoKit, Android uses BouncyCastle. Verified cross-stack: BouncyCastle and ed25519-dalek produce byte-identical signatures for the same key+nonce, so a phone-signed challenge verifies on the device. Remaining: confirm at-rest key storage (Keychain / EncryptedSharedPreferences / OS keychain) on real devices.
