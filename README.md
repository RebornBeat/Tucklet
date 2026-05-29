# Tucklet

**A pocket-sized companion drive that clips to your phone like a charm and holds your memories — no data cable, no babysitting, no clutter.**

Tucklet is a small, battery-powered, wireless storage device. After a one-time
pairing, your phone recognizes it automatically — no button press in daily use —
and your photos and videos move on and off it through a clean app that never
makes you think about "caches" or "sync states." It is designed to disappear
into your daily carry, and to be usable by someone who has never thought about
storage in their life.

![Tucklet flow](docs/assets/flow.svg)

> **Status:** Engineering foundation, organized for build. The architecture,
> protocol, per-variant hardware design, mechanical model, and the shared
> firmware/app core are complete and (where they can be) compiled and tested.
> The on-device firmware binary and the native apps build on this foundation —
> see [Build status](#build-status).

---

## What it is

| | |
|---|---|
| **Storage** | **microSD** (swappable) or **eMMC** (sealed) — your choice of variant. |
| **Radio** | **ESP32-C5** (dual-band Wi-Fi 6 + BLE), single or dual. |
| **Control link** | **BLE** — discovery, auth, battery %, wake. Low power, always cheap. |
| **Wireless data** | **Wi-Fi** (SoftAP universally; **Wi-Fi Aware** seamlessly where certified). On only while transferring. |
| **Wired data** | **USB-C → USB-HS bridge** — ~20–40 MB/s for bulk; also charges. |
| **Security** | Invisible after a one-time pairing: silent low-duty BLE, cryptographic challenge-response, physical-press to enroll a *new* phone. |

## The experience

- **Plain-language state, never jargon:** **On phone / On Tucklet / Temporary** — the word "cache" never appears.
- **You set how long "Temporary" lasts** (1 hour / 1 day / 1 week / keep).
- **Round-trip metadata:** files remember their origin album/app so "put it back" restores them exactly.
- **Per-app organization** by default; a friendly folder view for power users, never forced.
- **Honest transfer time:** every transfer shows an estimate before it starts and a live ETA while it runs.
- **Trickle backup:** new photos back up automatically when the charm is near and idle, so the big transfer never has to happen.

Full detail in [`docs/UX_SPEC.md`](docs/UX_SPEC.md).

## Repository layout

```
tucklet/
├── README.md                  # you are here
├── LICENSING.md               # which license covers what (source-available, non-commercial)
├── LICENSE-HARDWARE.txt        # CC BY-NC-SA 4.0 (hardware/mechanical)
├── LICENSE-SOFTWARE.txt        # PolyForm Noncommercial 1.0.0 (firmware/apps)
├── docs/
│   ├── INDEX.md               # documentation map — start here to navigate
│   ├── FINAL_REVIEW.md        # current source of truth (supersedes ADRs)
│   ├── VARIANT_MATRIX.md      # every build config, one codebase
│   ├── TRANSFER_PERFORMANCE.md, UX_SPEC.md, POWER_THERMAL.md, PRIOR_ART_AND_LICENSE.md
│   ├── adr/                   # original decision records (history)
│   ├── protocol/PROTOCOL.md   # the wire contract (mirrored by tucklet-proto)
│   └── assets/flow.svg
├── hardware/
│   ├── README.md              # hardware overview + how the variants relate
│   ├── gen_hardware.py        # parametric SOURCE — regenerates every variant
│   ├── common/                # shared parts rationale, cross-variant comparison, block diagram
│   └── variants/              # one folder per board: SPEC, BOM, PIN_MAP, SCHEMATIC_PLAN, .net, diagram
│       ├── singlec5-microsd/  ├── singlec5-emmc/  ├── dualc5-microsd/  └── dualc5-emmc/
├── firmware/                  # Rust workspace (tucklet-proto + tucklet-core compile + tested)
├── software/ios/              # iOS companion app (SwiftUI + File Provider)
└── mechanical/                # parametric enclosure (renders STLs for both envelopes)
```

## Start here

1. [`docs/INDEX.md`](docs/INDEX.md) — the full documentation map.
2. [`docs/FINAL_REVIEW.md`](docs/FINAL_REVIEW.md) — the load-bearing decisions, current.
3. [`hardware/README.md`](hardware/README.md) — the four board variants and how to build them in KiCad.
4. [`docs/protocol/PROTOCOL.md`](docs/protocol/PROTOCOL.md) — the contract that keeps firmware and apps in sync.

## Build status

What is **compiled and tested** here (run `cargo test` in `firmware/`):

- `firmware/crates/tucklet-proto` — every wire type (variant matrix, states, origin metadata, transfers). *5 tests._
- `firmware/crates/tucklet-core` — device brain: transfer-time estimator, link profiles, state machine, trickle scheduler, Temporary expiry, allow-list, transport resolution. *13 tests._

What is **complete and verified to render**: the parametric hardware (4 variants, all netlists validated), the enclosure (renders watertight STLs for both envelopes), and all design docs.

What **builds on this foundation and needs on-hardware/SDK bring-up**: the
on-device ESP32-C5 firmware binary and the native apps (iOS present; Android +
desktop next). Those can't be compiled in CI here against the C5 toolchain and
the iOS 26 SDK, so they are written real and complete with the exact
SDK-confirmation points flagged rather than guessed. See `docs/INDEX.md` for the
current state of each.

## License

Dual, **source-available, non-commercial** (study/modify/build, don't sell):
hardware under CC BY-NC-SA 4.0, software under PolyForm Noncommercial 1.0.0. This
is *not* an OSI/GNU "open source" license — see [`LICENSING.md`](LICENSING.md).

---

*"Tucklet" is a working product name (verified clear vs. the taken "Locket"/"Tuckit"). Trademark-check class 9 before commercial use.*
