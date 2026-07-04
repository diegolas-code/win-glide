# History Log: Resizing Physics Parameters Tuning

*   **Date:** 2026-07-04
*   **Feature:** Snappy Resizing Physics Parameters Tuning
*   **Branch:** `feat/tuned-resize-physics`

---

## Technical Decisions & Rationale

### 1. Scaling Resizing Physics to Match Translation
*   **Problem:** Resizing was configured with a very low acceleration (`resize_speed * 1.5` = `900.0`) and low top speed (`resize_speed` = `600.0`) because physically repainting target windows at higher speeds previously lagged target applications. However, with Ghost Resizing active, only the lightweight overlay is moved in real-time, meaning we can increase velocity parameters with zero performance overhead.
*   **Decision:** Tune `resize_physics_config` to match the snappiness of the translation physics:
    *   **Acceleration:** Increase from `resize_speed * 1.5` to `resize_speed * 5.0` (`3000.0` pixels/s² at default `600.0` speed setting). This results in near-instant response when keys are pressed.
    *   **Top Speed:** Increase from `resize_speed` to `resize_speed * 2.5` (`1500.0` pixels/s at default `600.0` speed setting).
    *   **Friction:** Set to `12.0` (up from `10.0`) to provide sharp, high-precision stops when resizing keys are released.

---

## Verification
*   **Unit Tests:** Verified that all 19 tests compile and pass successfully with zero warnings.
