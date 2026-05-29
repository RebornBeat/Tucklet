# ADR-003: iOS platform constraints — Android first

**Status:** Accepted · **Date:** 2026-05

## Context

The product targets a broad, non-technical audience on both Android and iOS. iOS imposes hard limits on third-party wireless storage accessories that shape the roadmap.

## The constraints

1. **No arbitrary background radio.** iOS apps cannot freely scan/connect to WiFi peripherals in the background. BLE background modes exist but are restricted.
2. **Joining the device's WiFi requires NEHotspotConfiguration.** The app can prompt the user to join Tucklet's SoftAP, but it is more visible/clunky than Android's programmatic join, and the OS may show its own prompts.
3. **Filesystem access is sandboxed.** To make Tucklet's files appear in the native Files app, you implement a **File Provider extension**. There is no general "see all my photos' raw folders" access; you work through PhotoKit (`PHPhotoLibrary`) for the camera roll.
4. **MFi only matters for Lightning/wired-accessory protocols.** For a WiFi/BLE device you generally do not need MFi, but verify against current Apple rules before shipping, especially if you ever add a wired iOS data mode.

## Decision

- **Launch Android first.** It supports programmatic SoftAP join, broader filesystem access, and faster iteration. This is where the product feels most magical with least friction.
- **iOS second**, built around: BLE for control, `NEHotspotConfiguration` to join the SoftAP, local HTTP for transfer, PhotoKit for the camera roll, and a File Provider extension so Tucklet appears in Files.
- **Set expectations honestly in product copy:** the iOS experience is excellent but cannot be as invisible as Apple's own AirDrop, because AirDrop is closed to third parties.

## Consequences

- The PROTOCOL is designed to be transport-agnostic above the socket so the same firmware serves both platforms unchanged.
- iOS-specific friction (the join-WiFi prompt) is handled with a guided one-time onboarding flow, not hidden — honesty over false seamlessness.
