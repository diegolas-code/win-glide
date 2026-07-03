# History Log: Window Resizing Smoothness Optimization

*   **Date:** 2026-07-03
*   **Feature:** Keyboard-Driven Window Resizing Smoothness
*   **Branch:** `feat/keyboard-resizing-smoothness`

---

## Technical Decisions & Rationale

### 1. 60Hz Layout Update Throttling
*   **Problem:** Resizing at 120Hz caused massive layout thrashing and paint backlogs in third-party windows. Since foreign window threads run asynchronously and generally cannot paint/layout at 120Hz, bombarding them with 120 resize events per second choked their message queues, resulting in laggy, blocky sizing jumps.
*   **Decision:** Split the resizing cycle:
    *   The **physics simulation** (velocity, friction) still updates at the full **120Hz** loop rate to keep keyboard inputs and velocity curves smooth and Snappy.
    *   The **Win32 layout updates** (`DeferWindowPos`, `UpdateLayeredWindow`, and coordinate updates) are throttled to **60Hz** (triggered every 16ms).
*   **Result:** Gives target windows sufficient time to process paints, keeping the overlay and target window perfectly synced while resolving frame drops.

### 2. Sizing Flag Optimization
*   **SWP_NOCOPYBITS Removal:** Omitted `SWP_NOCOPYBITS` from the `DeferWindowPos` flags during resizing. This allows Windows to copy the valid client area pixels during layout shifts, preventing blank/erased background flashes.
*   **SWP_NOSENDCHANGING Addition:** Added `SWP_NOSENDCHANGING` to prevent sending the blocking `WM_WINDOWPOSCHANGING` message to the foreign window thread, speeding up layouts.

---

## Verification
*   **Unit Tests:** All unit tests compile and pass.
*   **Aesthetics:** Resizing is completely fluid and buttery smooth, without visual detachment of the overlay or blocky stuttering.
