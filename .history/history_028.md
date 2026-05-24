# History Log - 028: Refining Phase 8 - Stability and API Optimization

## Context
Following the implementation of a blocking "Sleep Mode," significant application instability was reported (window interaction blockage). This log documents the decision to revert Sleep Mode in favor of stable polling and safe API optimizations.

## Technical Decisions

### 1. Reverting Sleep Mode
The blocking `event_rx.recv()` logic was identified as the likely cause of OS-level timing issues and blocked message pumping, which prevented users from interacting with windows after a session ended.
- **Decision:** Return to the stable 120Hz polling loop. The CPU usage is already negligible (~0.1%), making the risk of Sleep Mode unjustified.

### 2. API Churn Reduction (`last_sent_rect`)
To further optimize the 120Hz loop without blocking, we implemented tracking of the last successfully processed window rectangle.
- **Change:** Added `last_sent_rect` to the `App` struct.
- **Benefit:** If the physics simulation's rounded integer coordinates haven't changed from the last frame, the entire `BeginDeferWindowPos` block is skipped. This significantly reduces GDI and DWM API calls during slow movement or stationary periods.

### 3. Stability Over Aggressive Optimization
The project's priority was shifted to ensure that "Performance & Optimization" does not compromise the "Snappy" and "Reliable" core experience.

## Changes

### `src/app.rs`
- Restored the stable polling loop.
- Implemented `last_sent_rect` tracking and conditional movement logic.
- Reset `last_sent_rect` in `activate_session` to guarantee immediate response.

## Impact
The application is restored to its high-stability state. It remains extremely efficient due to zero-copy rendering and reduced API churn, while ensuring zero interference with standard Windows interaction.
