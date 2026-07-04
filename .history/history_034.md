# History Log: Keyboard-Driven Window Resizing Design Specification

*   **Date:** 2026-07-02
*   **Feature:** Keyboard-Driven Window Resizing (Design Specification)
*   **Branch:** `feat/keyboard-resizing`

---

## Technical Decisions & Rationale

### Context & Goal
1. **Window Boundary Adjustments:** The user wanted to implement fine-grained keyboard-driven window resizing in `win-glide` during an active glide session.
2. **Control Discovery & Discoverability:** Alt + Arrows (to expand outward) and Shift + Arrows (to shrink inward) were chosen as the fixed, absolute control layout. This avoids key collisions while maintaining symmetry with the existing Arrow-only translation system.

### Rationale & Design Trade-offs
1. **Simplified Sizing Accumulators vs. Double Physics Simulation:**
   * *Decision:* Instead of running a second independent momentum-based physics state, the revised design implements a direct step-based sizing model utilizing high-precision `width_f32` and `height_f32` accumulators.
   * *Why:* Resizing usually requires precise, discrete control rather than momentum-based drift. Step-based scaling remains easy to reason about and code, with the design leaving open the possibility of adding momentum resizing physics in a later phase if needed.
2. **Velocity Reset (Glide-Resize Handoff):**
   * *Decision:* The application resets the translational velocity `self.physics.velocity` to `0.0` immediately when resize modifiers are active.
   * *Why:* This prevents existing glide momentum from causing the window to translate/drift while the user is actively resizing it.
3. **Safety Clamping:**
   * *Decision:* Enforces a hardcoded minimum window size floor of 250x250px, dynamically scaled by the target window's DPI factor, blocks expansions that exceed the nearest monitor's work area, and enforces a 150px off-screen margin limit.
   * *Why:* Prevents divisions by zero, vanishing windows, and UI overlap with OS taskbars.
4. **Rendering & Overlay Resource Protection:**
   * *Decision:* Coordinate layout updates of target window and overlay inside a single Win32 transaction (`BeginDeferWindowPos`). Limit re-rendering (`Overlay::redraw`) to trigger only when the target window's integer size actually changes.
   * *Why:* Recreating GDI DIB sections on every frame of a high-Hz loop (120Hz) creates excessive CPU and GDI resource allocation overhead. Skipping redraws during pure translations keeps resource usage minimal.
5. **Low-Level Hook Passthrough:**
   * *Decision:* The low-level keyboard hook callback `keyboard_proc` is configured to not consume `VK_MENU` or `VK_SHIFT` modifiers, while continuing to intercept arrow keys.
   * *Why:* Bypassing modifier consumption prevents stuck key problems and system-wide hotkey lockups, while querying modifier state via `GetAsyncKeyState` in the main loop keeps logic clean and fast.

## Documentation Updated
*   **[SPEC.md](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/SPEC.md):** Added Section 10 containing specifications for keyboard-driven window resizing.
*   **[TODO.md](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/TODO.md):** Appended Phase 11 detailing the developer implementation roadmap checklist.
*   **[PAUSE.md](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/PAUSE.md):** Updated achievements with the completed design phase and added active next steps.
