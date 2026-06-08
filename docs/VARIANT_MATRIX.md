# Variant Matrix — every build configuration, and how one codebase serves all

Tucklet is not one device; it's a small product line from a single shared codebase. Four independent axes multiply into the full matrix. Firmware and apps adapt at runtime via the `DeviceCapabilities` descriptor (see `tucklet-proto`), so there is **one** firmware source and **one** app per platform — variants are configuration + capability negotiation, not forks.

## The four axes

| Axis | Options | Chosen by |
|---|---|---|
| **Chip** | `C5` (Production) / `E22` (Roadmap) | hardware build (compile-time feature) |
| **Form Factor** | `WROOM` (Standard) / `MINI` (Compact) | hardware build |
| **Radio Count** | `Single` / `Dual` | hardware build |
| **Storage** | `MicroSd` / `Emmc{capacity_gib}` | hardware build |
| **Transport(s)** | `SoftAp` (always) + optionally `WifiAware` + optionally `WiredUsbHs` | firmware feature flags + runtime |

Theoretical maximum: 2 Chips x 2 Forms x 2 Counts x 2 Storage = 16 Variants.
**Actual Generated Variants:** 12 (See "Charm Strategy Exclusion" below).

## Charm Strategy Exclusion
To maintain the "Charm" product identity (invisible, robust, AirTag-class), we enforce a strict size constraint.
*   **Excluded:** `E22-WROOM` variants. The WROOM/M.2 module is oversized (22x30mm) and requires an external antenna (IPEX), which violates the sealed/integrated design philosophy.
*   **Included:** `E22-MINI` variants. We anticipate a future Mini module that fits the Charm form factor. These are marked as **Roadmap** items.

## The 12 generated configurations

### Tier 1: The "Charm" (Standard C5)
The production-ready baseline using the ESP32-C5-WROOM-1 module.

| # | Radio | Storage | Wireless | Wired | Position |
|---|---|---|---|---|---|
| 1 | SingleC5 | microSD | SoftAp | USB-HS | **Baseline** — cheapest, swappable |
| 2 | SingleC5 | eMMC | SoftAp | USB-HS | Slim, sealed, fixed capacity |
| 3 | DualC5 | microSD | SoftAp | USB-HS | Speed-focused, swappable |
| 4 | DualC5 | eMMC | SoftAp | USB-HS | Speed + sealed |

### Tier 2: The "Nano" (Mini C5)
The ultra-compact production line using the ESP32-C5-MINI-1 module.

| # | Radio | Storage | Wireless | Wired | Position |
|---|---|---|---|---|---|
| 5 | SingleC5 | microSD | SoftAp | USB-HS | Ultra-compact, swappable |
| 6 | SingleC5 | eMMC | SoftAp | USB-HS | Smallest sealed charm |
| 7 | DualC5 | microSD | SoftAp | USB-HS | Experimental speed in Nano form |
| 8 | DualC5 | eMMC | SoftAp | USB-HS | Top-tier Nano |

### Tier 3: The "Super-Charm" (Roadmap E22)
High-performance variants dependent on the future release of an ESP32-E22-MINI-1 module.

| # | Radio | Storage | Wireless | Wired | Position |
|---|---|---|---|---|---|
| 9 | SingleE22 | microSD | SoftAp | USB-HS | High-speed roadmap |
| 10 | SingleE22 | eMMC | SoftAp | USB-HS | High-speed sealed roadmap |
| 11 | DualE22 | microSD | SoftAp | USB-HS | Extreme performance roadmap |
| 12 | DualE22 | eMMC | SoftAp | USB-HS | Ultimate speed roadmap |

*(Note: microSD and eMMC each span the capacity sub-axis; eMMC capacities and BOM are in `hardware/common/VARIANTS.md`.)*

## The Product Lines

Tucklet is organized as two distinct product lines, both under the same source-available, non-commercial licensing:

| Line | Radio | Form Factor | Variants | Position |
|---|---|---|---|---|
| **Charm Standard** | ESP32-C5 | WROOM | 4 | Baseline: Lowest cost, easy assembly |
| **Charm Compact** | ESP32-C5 | MINI | 4 | Nano: Smallest footprint, tighter layout |
| **Charm High-Perf** | ESP32-E22 | MINI | 4 | Roadmap: Wi-Fi 6E speed, pending module release |
| **Pro High-Perf** | ESP32-E22 | WROOM | Roadmap | Larger: Wi-Fi 6E speed, 2S battery, pending module release |

All lines are licensed under **CC BY-NC-SA 4.0** (hardware) and **PolyForm Noncommercial 1.0.0** (software). The Pro line is a **Charm** — it fits a "charm" envelope (wearable, on-phone), even though it is thicker to accommodate the 2S battery. It is not a "backpack" or "brick" class device. You may study, modify, and build for personal use; commercial sale requires a separate agreement.

**Note:** The ESP32-E22 variants (both MINI and WROOM) are **Roadmap** items. The `ESP32-E22-MINI-1` module is projected but not yet released. The `ESP32-E22-WROOM` module (22×30 mm, external IPEX antenna) is the basis for the Pro line. **No E22 variant folders are created at this time** (see "Variant Folder Organization" below).

## How firmware handles the matrix

Compile-time Cargo features select the hardware reality; everything else is runtime:

```
[features]
# Chip selection
chip_c5 = []
chip_e22 = []

# Form Factor
form_wroom = []
form_mini  = []

# Radio Count (derived or explicit)
radio_single = []
radio_dual   = []

# Storage
storage_microsd = []
storage_emmc    = []

# Transport
transport_softap = []        # always on
transport_wifi_aware = []    # optional
transport_wired_usbhs = []   # optional (the bridge IC is populated)

# AGG Link
agg_spi = []                 # dual variants use SPI; single doesn't need it
```

At boot the firmware assembles a `DeviceCapabilities` from its enabled features and advertises it. `tucklet-core::variant::usable_transports()` (unit-tested) then narrows that to what the connected client can use.

## How apps handle the matrix

The app never hard-codes a variant. On connect it reads `DeviceCapabilities` over BLE, then:

1. Calls the equivalent of `usable_transports(caps, platform)` to pick a transport (Wi-Fi Aware first if both sides have it, else SoftAP; wired when plugged in).
2. Loads the matching `LinkProfile` (`tucklet-core::link::profile_for`) so the transfer-time estimate reflects *this* unit's real speed.
3. Renders capacity, storage type, and battery from the live `StatusReport`.

So a user with a microSD SingleC5 Mini and a user with a DualC5 eMMC Standard run the identical app; it simply reports different speeds and capacities. The same is true across iOS, Android, and desktop because all three consume the same `tucklet-proto` types.

## Capability negotiation sequence

1. BLE connect + cryptographic auth (allow-list).
2. App reads `DeviceCapabilities` + `StatusReport`.
3. App resolves transport + link profile for the current platform and physical state (plugged in or not).
4. App shows accurate ETAs and storage states before any bytes move.

## Storage axis

### microSD (swappable)
Push-push socket, SDIO 4-bit, +~$0.40. Customer supplies/upgrades the card; you don't pay for flash. Dead card = $5 swap, not a dead device. Slightly larger enclosure (socket + insertion clearance), placed internally behind a small door so the unit still looks sealed.

### eMMC (sealed) — per-capacity cost (small-volume estimate; VERIFY)
153-ball TFBGA eMMC 5.1, 11.5 × 13.0 × 1.2 mm. Capacity is fixed at manufacture.

| Capacity | eMMC est. | Full BOM (single-C5) |
|---|---|---|
| 8 GB | ~$2.50 | ~$13.5 |
| 16 GB | ~$3.50 | ~$14.5 |
| 32 GB | ~$5.00 | ~$16.0 |
| 64 GB | ~$8.00 | ~$19.0 |
| 128 GB | ~$13.00 | ~$24.0 |
| 256 GB | ~$22.00 | ~$33.0 |

Only 8–16 GB eMMC stays near the sub-$15 target — the honest cost of integrated flash, and exactly why offering microSD *and* eMMC is smart. Requires BGA assembly (CM with reflow + ideally X-ray).

## Radio axis

Performance and Power vary significantly between the C5 and E22 generations.

| Radio | Real wireless | Module size | Fits charm? | Notes |
|---|---|---|---|---|
| 1× ESP32-C5 | ~6–9 MB/s (5 GHz Wi-Fi 6) | WROOM: ~18 × 27.5 mm / MINI: ~15.4 × 21.3 mm | yes | **Recommended baseline.** Best balance of power/performance. |
| 2× ESP32-C5 | ~12–15 MB/s (SPI AGG) | 2 modules + 2 antennas | tight | **Experimental.** Validate RF isolation before production. SPI link prevents inter-chip bottleneck. |
| 1× ESP32-E22 | ~20–40+ MB/s (Wi-Fi 6E) | MINI: ~15 × 21 mm (projected) | yes (warm) | **High Performance.** Tri-band (2.4/5/6 GHz). Requires >2A buck & larger battery. **MINI form factor only. Roadmap.** |
| 2× ESP32-E22 | ~40–70+ MB/s (SPI AGG) | MINI: 2 modules | tight (hot) | **Extreme Performance.** Significant thermal considerations. **MINI form factor only. Roadmap.** |
| 1× ESP32-E22 (Pro) | ~20–40+ MB/s (Wi-Fi 6E) | WROOM: 22 × 30 mm | yes (Pro envelope) | **Pro Charm.** External IPEX antenna. 2S battery. ~35
