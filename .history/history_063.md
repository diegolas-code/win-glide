# History Log: Calibrate Codebase Default Config & Physics

*   **Date:** 2026-07-16
*   **Feature:** Default Configuration Calibration & Test Robustness
*   **Branch:** `master` (via `fix/codebase-defaults`)

---

## Technical Decisions & Rationale

### 1. Sluggish Fallback Configuration Defaults
- **Problem:** When `config.json` did not exist next to the executable (e.g. on new builds or environments), the app fell back to the hardcoded defaults in the code.
- **Consequence:** These defaults were set to old slower values (`2000.0` acceleration, `1500.0` top speed, and `None` for `resize_physics` leading to a default friction of `20.0`). This caused the application to feel sluggish and blocky again when the config file was missing or regenerated.
- **Decision:** Align the hardcoded defaults in the codebase directly with our new calibrated +10% fast/smooth physics parameters.
- **Implementation:**
  - In `src/physics.rs`, update `Default` implementation for `PhysicsConfig` to set `acceleration: 2860.0`, `friction: 11.0`, and `top_speed: 2475.0`.
  - In `src/config.rs`, update `default_resize_speed()` to return `660.0`.
  - In `src/config.rs`, update `Default` implementation for `Config` to set `resize_speed: 660.0` and default `resize_physics` to `Some(PhysicsConfig)` with `acceleration: 3300.0`, `friction: 15.0`, and `top_speed: 1650.0`.

### 2. Flaky Headless Test Failure
- **Problem:** The test `test_get_dpi_for_window` failed when run in a background or headless console worker.
- **Root Cause:** `GetForegroundWindow()` returns `0` (null) when run in a background process, making `GetDpiForWindow` return `0`, which failed the `assert!(dpi > 0)` check.
- **Decision:** Modify the test to robustly handle the null pointer case.
- **Implementation:**
  - In `src/platform.rs`, check if `hwnd.0` is null using `hwnd.0.is_null()`.
  - If null, assert that `dpi` is `0`. Otherwise, assert that `dpi > 0`.

---

## Verification
- **Unit Tests:** All tests compile and pass successfully under background execution.
- **Manual Verification:** Deleting `target/release/config.json` and running `cargo run --release` successfully generates a new `config.json` containing the smooth, fast default values.
