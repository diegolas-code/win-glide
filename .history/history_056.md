# History Log: Glide Speed Calibration and Legibility Background

*   **Date:** 2026-07-04
*   **Feature:** Usability & Physics Fine-tunes (Phase 10c)
*   **Branch:** `feat/text-background` (merged to `dev`)

---

## Technical Decisions & Rationale

### 1. Glide Speed Calibration
- **Problem:** At `acceleration: 4000.0` and `top_speed: 4000.0`, window movement was too sensitive and fast to control accurately, leading to overshoot.
- **Decision:** Fine-tune the physics parameters in `config.json` to 0.3 of the delta between the new slow defaults (`2000` accel, `1500` top speed, `12` friction) and the original values (`4000` accel, `4000` top speed, `10` friction):
  - **Acceleration**: `2600.0`
  - **Top Speed**: `2250.0`
  - **Friction**: `11.0`
- **Result:** Balanced, snappy movement that remains fast but is highly controllable without drifting or overshooting.

### 2. Help Text Legibility Background
- **Problem:** Drawing white help text instructions directly on the overlay window makes it difficult to read on bright window contents (e.g. text editors, web browsers with light backgrounds).
- **Decision:** Draw a translucent black background rectangle behind the help text block.
- **Implementation:**
  - Before rendering the text, calculate the exact text layout bounding box using `DrawTextW` with `DT_CALCRECT` to obtain the actual wrapped text dimensions (`rect.right - rect.left`).
  - Center `draw_rect` horizontally within the target area based on this calculated text width, ensuring the background only spans the actual width of the text block rather than the full window width.
  - Re-open `PixmapMut` on the back buffer slice, pad the calculated bounding box (12px horizontal, 8px vertical), and draw a rounded rectangle filled with a 30% black color block (`0, 0, 0` at `76` alpha) using `tiny-skia`.
  - Draw the white GDI text on top of the newly filled background.
- **Result:** The white text pops clearly on any type of window content without losing the sleek transparency of the overlay border, and the background remains tightly fitted to the text size.

---

## Verification
- **Unit Tests:** All 20 tests pass.
- **Clippy:** Completely clean checks.
