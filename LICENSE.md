# Licensing

This document explains the licensing structure for the Tucklet project. The project uses a dual-licensing model to balance open collaboration with commercial protection.

## Overview

Tucklet is **source-available**, not "open source" in the OSI definition sense. You are free to study, modify, and build the project for personal use, but commercial use is restricted.

### 1. Firmware and Software
**Directory:** `firmware/`, `software/`
**License:** [`LICENSE-SOFTWARE.txt`](LICENSE-SOFTWARE.txt) (**PolyForm Noncommercial 1.0.0**)

This license grants you the right to use, modify, and distribute the code, but **not for commercial purposes**.
*   **Permitted:** Personal projects, research, education, non-profit use, contributing improvements back to the project.
*   **Prohibited:** Selling the software, devices running the software, or using the code in a commercial product without a separate commercial license agreement.

### 2. Hardware and Mechanical Design
**Directory:** `hardware/`, `mechanical/`
**License:** [`LICENSE-HARDWARE.txt`](LICENSE-HARDWARE.txt) (**CC BY-NC-SA 4.0**)

This license grants you the right to study, modify, and redistribute the hardware design files (schematics, PCB layouts, 3D models) under the following conditions:
*   **Attribution:** You must give appropriate credit to the Tucklet project.
*   **Non-Commercial:** You may not use the material for commercial purposes.
*   **ShareAlike:** If you remix, transform, or build upon the material, you must distribute your contributions under the same license.

This license covers **both the Charm and Pro product lines**. The Pro line (ESP32-E22-WROOM + 2S LiPo) is still a "Pro Charm" — it clips to your phone, hangs on a string, and disappears into your daily carry. It is not a "backpack" or "brick" class device. The thickness is justified by the performance, not a change in product category. Therefore, the Pro line falls under the same source-available, non-commercial license terms as the Charm line.

## Important Distinctions

### Why not GPL?
GNU General Public Licenses (GPL) explicitly permit commercial sale. The intent of Tucklet's licensing is to allow the creator to sell the official device while preventing competitors from cloning the hardware or software for profit. Therefore, "No selling" is fundamentally incompatible with the GPL and with the formal definition of "open source."

### "Open Source" vs. "Source-Available"
Because the licenses used here restrict commercial use, Tucklet cannot technically be called "Open Source" (as defined by the Open Source Initiative). It is correctly described as **Source-Available**.

### Patent Notice
Copyright licensing covers the specific implementation (code, design files). It does not grant a patent license on general concepts. However, the project is intended to be a clean-room implementation. See `README.md` for prior art context.

## Hardware scope & novelty

The Hardware License (CC BY-NC-SA 4.0) applies to the specific implementation files
(schematics, PCB layouts, netlists, and mechanical models) contained in this
repository for **both the Charm and Pro product lines**. Specifically, the license covers the
following novel design elements:

1. **SDIO Bus Multiplexing Architecture:** The specific circuit topology where the
   SDIO storage bus is arbitrated between a Wi-Fi radio (ESP32-C5/E22) and a
   dedicated USB-HS Bridge (U2) via a hardware mux (U6), enabling seamless
   switching between wireless and high-speed wired modes without user intervention.
2. **Dual-Radio SPI Link Aggregation Topology:** The unique placement, antenna
   isolation strategy, and inter-processor SPI communication (AGG link) design
   required to fit two ESP32 radios in a "Charm" form factor for aggregated
   throughput.
3. **Integrated Charm Enclosure:** The parametric mechanical design that integrates
   a battery, antenna, storage, and thermal management into an AirTag-class envelope.

## Enforcement
This license is designed to protect the project's sustainability. If you wish to use Tucklet technology in a commercial product, please contact the project maintainer to discuss commercial licensing terms.
