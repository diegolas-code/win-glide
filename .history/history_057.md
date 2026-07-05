# History Log: Text Anti-Aliasing Pre-multiplied Alpha Correction

*   **Date:** 2026-07-04
*   **Feature:** Usability & Physics Fine-tunes (Phase 10c - Part 2)
*   **Branch:** `fix/text-pixelation` (merged to `dev`)

---

## Technical Decisions & Rationale

### 1. Pre-multiplied Alpha Blending Correction
- **Problem:** When drawing white text over the black background, the text edges looked pixelated, jagged, and harsh.
- **Root Cause:** In the alpha post-processing loop, we were updating the alpha channel `*a = new_alpha`, but leaving the RGB channels set to `255` (fully saturated white). Windows' `UpdateLayeredWindow` expects pixels in **pre-multiplied alpha format** (where $RGB \le A$). Having $RGB > A$ is physically invalid. When a black background is active underneath, DWM failed to blend these invalid pixels properly, turning them into harsh, solid-white blocks.
- **Decision:** Implement proper pre-multiplied alpha blending for white text on a black background in the GDI post-processing loop:
  - Let $A_T$ be the straight text alpha (`intensity * INDICATOR_OPACITY`).
  - Let $A_B$ be the background alpha (`bg_alpha`).
  - The blended pre-multiplied color is $C'_{out} = C'_{T} + (1 - A_T / 255) \times C'_{B}$.
  - Since the background is black ($C'_{B} = [0,0,0]$), the pre-multiplied RGB values are simply equal to $A_T$.
  - The blended output alpha is $A_{out} = A_T + (1 - A_T / 255) \times A_B$.
  - Therefore, we set:
    - `slice[offset..offset+3] = text_alpha` (RGB)
    - `slice[offset+3] = new_alpha` (Alpha)
- **Result:** Beautifully smooth, sub-pixel anti-aliased text that blends perfectly over the black background with no pixelation or jagged edges.

### 2. Scan Area Padding
- **Problem:** Shrunk bounds caused anti-aliasing edges of the font to cut off at the margins.
- **Decision:** Pad the scan bounding box by `8` pixels on all sides (`scan_padding = 8`) to ensure all surrounding GDI anti-aliased sub-pixels are captured by the loop.

---

## Verification
- **Unit Tests:** All 20 tests pass.
- **Clippy:** Clean checks.
