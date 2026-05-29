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

## Important Distinctions

### Why not GPL?
GNU General Public Licenses (GPL) explicitly permit commercial sale. The intent of Tucklet's licensing is to allow the creator to sell the official device while preventing competitors from cloning the hardware or software for profit. Therefore, "No selling" is fundamentally incompatible with the GPL and with the formal definition of "open source."

### "Open Source" vs. "Source-Available"
Because the licenses used here restrict commercial use, Tucklet cannot technically be called "Open Source" (as defined by the Open Source Initiative). It is correctly described as **Source-Available**.

### Patent Notice
Copyright licensing covers the specific implementation (code, design files). It does not grant a patent license on general concepts. However, the project is intended to be a clean-room implementation. See `README.md` for prior art context.

## Enforcement
This license is designed to protect the project's sustainability. If you wish to use Tucklet technology in a commercial product, please contact the project maintainer to discuss commercial licensing terms.
