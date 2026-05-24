# Current Status - win-glide

## Recent Achievements
- **Phase 7: Visual Polish & UX (Completed)**
    - Finalized visual refinements: reduced overlay "header" extension to 7px and implemented 8px rounded corners.
- **Phase 8: Performance & Optimization (In Progress)**
    - **API Churn Reduction:** Implemented `last_sent_rect` tracking. The 120Hz movement loop now skips all Win32 API calls (`BeginDeferWindowPos`, etc.) if the window's integer position hasn't changed.
    - **Zero-Copy Rendering:** Refactored overlay rendering to draw directly into Win32 GDI memory, eliminating full-frame memory copies.
    - **Instant Activation:** Optimized the hotkey trigger path with immediate message pumping.
    - **Elevated Window Safety:** Proactively detects and skips high-integrity windows (like Task Manager) if the app is not elevated.

## Immediate Next Steps
- **Phase 8 Completion:**
    - Re-evaluate and implement a stable version of "Sleep Mode" for the physics loop (or decide if polling at 120Hz with 0.1% CPU is already optimized enough).
    - Profile memory and GDI handle lifecycle.

## Technical Notes
- Stability is prioritizing over aggressive idle power savings.
- `DeferWindowPos` synchronization is maintained as the core movement mechanism.
