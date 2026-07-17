# History Log: Pass Window DPI to Overlay

*   **Date:** 2026-07-16
*   **Feature:** Overlay DPI Query Optimization
*   **Branch:** `dev` (via `perf/overlay-dpi`)

---

## Technical Decisions & Rationale

### 1. GDI DPI Query Bottleneck in 120Hz Loop
- **Problem:** During active glide sessions, the overlay was redrawn at 120Hz. On every frame, `Overlay::prepare_surface` queried the target window's monitor DPI by calling `GetDC(None)`, `GetDeviceCaps()`, and `ReleaseDC()`.
- **Consequence:** These Win32 GDI calls require user-to-kernel mode context switching. Calling them at 120Hz created a performance bottleneck and caused frame pacing spikes.
- **Decision:** Eliminate GDI queries inside `prepare_surface` by passing the already cached target window's DPI directly from the `App` struct state.
- **Implementation:**
  - Update `Overlay::prepare_surface` and `Overlay::redraw` in `src/ui.rs` to accept `dpi: u32` parameter. Replace internal calls to `GetDC` / `GetDeviceCaps` with scale factors calculated from the passed DPI.
  - Update all overlay rendering call sites in `src/app.rs` to supply the cached `self.dpi` value.
  - Update all overlay unit tests in `src/ui.rs` to supply a mock `96` DPI value.

---

## Verification
- **Unit Tests:** Updated unit tests in `src/ui.rs` pass successfully.
- **Performance:** Dynamic GDI context switching calls eliminated in the main loop, reducing render-pacing spikes.
