# History Log: Overlay Topmost Z-Order Synchronization

*   **Date:** 2026-06-22
*   **Feature:** Synchronize Overlay Z-Order with Target Window's Topmost Status
*   **Branch:** `feat/overlay-topmost`

---

## Technical Decisions & Root Cause Fix

### Root Cause
When the target window is pinned on top (by having the `WS_EX_TOPMOST` style applied by another window-pinning utility), the win-glide blue overlay window was being displayed *behind* the target window. 
This happened because the overlay window was created as a standard popup window without the `WS_EX_TOPMOST` style. In Windows, topmost windows reside in a Z-order band above all standard windows. Even though the overlay parent-owner relationship was set via `GWLP_HWNDPARENT`, a non-topmost window owned by a topmost window does not automatically draw on top of it properly without its own Z-order status being explicitly synchronized.

### Resolution
We resolved the Z-order layout issue by dynamically synchronizing the overlay window's topmost status with the target window's topmost status during session activation:
1.  **Extended Style Querying (`src/ui.rs`):** In `Overlay::set_owner`, we query the target window's extended style flags using `GetWindowLongW(owner, GWL_EXSTYLE)`.
2.  **Topmost State Extraction:** We check if the owner has the `WS_EX_TOPMOST` style: `(ex_style & WS_EX_TOPMOST.0) != 0`.
3.  **Z-Order Sync:** Depending on this check, we call `SetWindowPos` on the overlay specifying `HWND_TOPMOST` if the target is topmost, or `HWND_NOTOPMOST` if it is a standard window, with the flags `SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE`. 
    *   This forces Windows to place the overlay window in the correct Z-order band (topmost vs. standard). Since the overlay is owned by the target window, it is guaranteed to render directly on top of the target window in both bands.
    *   It avoids visual bugs where the overlay of a standard window incorrectly draws over unrelated topmost windows.

## Verification

*   Added a unit test in `src/ui.rs` (`ui::tests::test_overlay_topmost_sync`) verifying:
    *   Overlay is not topmost when the owner window is normal.
    *   Overlay successfully promotes to topmost when the owner is updated to topmost.
    *   Overlay demotes back to normal when the owner topmost status is removed.
*   Verified that all 13 tests compile and pass perfectly with `cargo test --bin win-glide`.
