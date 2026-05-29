# Tucklet

**A pocket-sized companion drive that clips to your phone like a charm and holds your memories — no data cable, no babysitting, no clutter.**

Tucklet is a small, battery-powered, wireless storage device. After a one-time pairing, your phone recognizes it automatically — no button press in daily use — and your photos and videos move on and off it through a clean app that never makes you think about "caches" or "sync states." It is designed to disappear into your daily carry — small enough to hang off your phone on a string or sit on the back — and to be usable by someone who has never thought about storage in their life.

![Tucklet flow](docs/assets/flow.svg)


> **Status:** Foundation / pre-hardware. This repository contains the architecture, protocol, hardware design plan, firmware scaffold, and documentation. It is **not** a manufacture-ready board yet — see [Honesty & Scope](#honesty--scope).

---

## Why Tucklet exists

Phone storage fills up. The "solutions" on the market are either old clunky WiFi drives or glorified pendrives that hog your charging port and need a cable for everything. Cloud storage hides what is actually stored where, and quietly bills you forever.

Tucklet's bet: people want **local storage they control**, with a UX as calm as the cloud and none of the mystery. Show the state plainly. Let the user decide what lives where. Make the hardware vanish.

## What it is (and isn't)

| | |
|---|---|
| **Storage** | A user-swappable **microSD card** behind a managed controller. Pick your own capacity; upgrade anytime. |
| **Control link** | **Bluetooth LE** — discovery, the button-press authorization handshake, battery %, wake/sleep. Low power, always-cheap. |
| **Data link** | **WiFi (SoftAP)** — the device hosts a tiny private network *only while transferring*. This is the only way to move real photo/video volumes wirelessly; Bluetooth is physically too slow. |
| **Charging / fallback data** | **USB-C** — charge the battery, and a wired high-speed fallback when you want it. |
| **Security** | The device is invisible and silent until you **press the button**. A press opens a short authorization window; your phone must be approved before any connection. Nothing is discoverable in the background. |
| **It is NOT** | A Bluetooth-only drive (impossible — see [ADR-002](docs/adr/ADR-002-connectivity.md)), a cloud service, or a device with a custom flash chip (see [ADR-001](docs/adr/ADR-001-storage-media.md)). |

## The experience we are building

- **Plain-language state, never jargon.** Files are shown as **On phone**, **On Tucklet**, or **Temporary** — never "cached." If you pull a file onto your phone temporarily, *you* set how long "temporary" lasts.
- **Round-trip metadata.** When Tucklet takes a file off your phone, it remembers where it came from (album, app) so it can put it back exactly there. No orphaned files.
- **Per-app organization, with a friendly view.** Organize by app (Camera, Screenshots, WhatsApp, …) rather than raw folders. Power users can still drill into folders; the default view never forces it.
- **Battery & status you can trust.** The app shows real battery %, free space, and transfer progress — no guessing.

## Repository layout

```
tucklet/
├── README.md                  # You are here
├── LICENSING.md               # Which license covers what, and why
├── LICENSE-HARDWARE.txt        # Hardware design files
├── LICENSE-SOFTWARE.txt        # Firmware + app source
├── docs/
│   ├── adr/                   # Architecture Decision Records (the "why")
│   └── protocol/PROTOCOL.md   # The contract between firmware and apps
├── hardware/                  # Component selection, pin map, BOM (KiCad work lives here)
├── firmware/                  # Rust firmware for the ESP32-S3 (scaffold)
├── software/                  # Companion apps (Android-first, then iOS, then desktop)
└── mechanical/                # Enclosure design + the CAD round-trip workflow
```

## Start here

1. Read the four Architecture Decision Records in [`docs/adr/`](docs/adr/) — they explain the load-bearing choices (storage media, connectivity, iOS, power & pairing).
2. Read [`docs/protocol/PROTOCOL.md`](docs/protocol/PROTOCOL.md) — the single contract that keeps firmware and apps in sync.
3. Read [`hardware/COMPONENT_SELECTION.md`](hardware/COMPONENT_SELECTION.md) and [`hardware/PIN_MAP.md`](hardware/PIN_MAP.md) — these are what you build the KiCad schematic from.

## Honesty & scope

This repo is a real, buildable **foundation**, not a finished product. Specifically:

- The hardware section gives you a **verified component plan, pin map, and block diagram** to build the KiCad schematic from — it does **not** contain gerbers or pick-and-place files, because those require iterative layout, RF/antenna tuning, and a design-for-manufacture review on real silicon. Faking them would waste your money.
- The firmware is a **structured Rust scaffold** with the real architecture in place, not flashed-and-tested production binaries.
- Regulatory certification (FCC/CE) is required before sale because Tucklet contains radios. That is a real cost and timeline, budgeted in the docs.

What *is* production-grade here: the architecture, the protocol contract, the decision records, the licensing, and the build sequence. That is the part that is expensive to get wrong and cheap to get right early.

## License

Dual: hardware and software are licensed separately. See [LICENSING.md](LICENSING.md). Short version: **source-available, non-commercial** — you may study, build, and modify Tucklet, but not sell it. (Note: this is *not* an OSI/GNU "open source" license; that distinction is explained in LICENSING.md.)

---

*Tucklet is an independent hardware project. "Tucklet" is a working product name — verify trademark availability in your launch markets before commercial use.*
