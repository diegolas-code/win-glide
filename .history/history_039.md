# History Log: Coordinated Resize Synchronization

*   **Date:** 2026-07-03
*   **Feature:** Keyboard-Driven Window Resizing (Atomic Synchronization)
*   **Branch:** `feat/keyboard-resizing-sync`

---

## Technical Decisions & Rationale

### 1. Coordinated Window Layouts via DeferWindowPos
*   **Problem:** Even with discrete step-based resizing, updating the target window's geometry (`SetWindowPos`) and redrawing/updating the overlay (`UpdateLayeredWindow`) in separate sequential API calls caused them to compile into separate DWM compositing cycles. This resulted in a brief one-frame visual lag, making the overlay look like it was "dragging" behind the target window during active resizing.
*   **Decision:** Grouped the geometry updates of both the target window and the overlay window inside a single `BeginDeferWindowPos(2)` transaction during the keydown event handler:
    *   This forces the Desktop Window Manager (DWM) to update the boundaries of both windows atomically in the exact same refresh cycle.
    *   Directly after the deferred layout transaction commits, `Overlay::redraw` is called to refresh the overlay's translucent pixel buffer.
*   **Result:** Visual dragging is entirely resolved. The overlay borders remain locked to the target window borders throughout the resizing action.

---

## Verification
*   **Unit Tests:** All unit tests pass cleanly.
*   **Aesthetics:** Resizing is completely synchronized and locked together, with zero visual desync, dragging, or rubber-banding.
