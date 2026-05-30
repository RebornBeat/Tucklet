> **SUPERSESSION NOTICE (read first).** This ADR is kept for history.
> The finalized design has moved on in the ways noted here; where this
> document and `../FINAL_REVIEW.md` differ, **FINAL_REVIEW wins**.
>
> microSD remains the entry variant, but eMMC is now a shipping sealed variant too (see hardware/common/VARIANTS.md), not just a 'revisit later' option. Both are built.

# ADR-001: Storage media — microSD over custom flash or eMMC

**Status:** Accepted · **Date:** 2026-05 · **Decision owner:** Christian

## Context

Tucklet needs onboard storage. Three options were considered: custom NAND flash integration, a soldered eMMC chip, or a microSD socket.

## Decision

**Use a user-swappable microSD socket** for the first product. Offer a sealed eMMC variant later if demand justifies it.

## Why

- **Custom NAND is not viable at this scale.** Raw NAND requires a flash controller doing wear leveling, error-correction coding (ECC), and bad-block management. This is the core IP of companies like SanDisk and Samsung and represents enormous R&D. It is incompatible with a sub-$15 BOM and a small-seller timeline. Rejected.
- **eMMC is viable but premature.** eMMC packages include the controller (managed NAND), so the hard flash-management problem disappears — but you commit to a capacity at manufacture, the BOM rises ($4–12 depending on size), and the user can never upgrade or replace it. Good for a future "sealed premium" SKU, not the first one.
- **microSD wins for v1.** The card carries its own controller, so Tucklet's firmware never touches flash management. It is the cheapest option (~$0.40 socket), it makes capacity the *customer's* choice, and a dead card is a $5 swap instead of a dead device.

## Consequences

- Firmware talks to the card over **SDIO** (4-bit, fast) on the ESP32-S3, with SPI as a fallback bring-up mode.
- The enclosure must expose or conceal a microSD slot. Decision: **internal slot** behind a small door, so the device still looks seamless and "sealed" day-to-day. (See `mechanical/`.)
- Firmware must handle **card absent / corrupt / wrong filesystem** gracefully and surface that state to the app in plain language ("No card inserted").
- Filesystem: **exFAT** for cross-platform compatibility and large-file support. Bundle an exFAT implementation with a license compatible with the firmware license (verify before shipping).

## Revisit if

Volume reaches the point where a sealed, thinner, more "invisible" device clearly outsells the swappable one — then introduce the eMMC SKU alongside, don't replace.
