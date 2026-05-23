# win-glide Project History: Movement Fluidity Improvements

## Change: Continuous Thrust Application
**Decision:** Shifted from event-based thrust to state-based thrust.
**Reasoning:** 
- The OS keyboard repeat rate (~30Hz) is too slow for a 120Hz physics loop, leading to "steppy" movement.
- By tracking key states (`KeyDown`/`KeyUp`) in a `HashSet`, we can apply thrust every frame, resulting in perfectly smooth acceleration.

## Change: Diagonal Normalization
**Decision:** Implemented vector normalization for multi-key input.
**Reasoning:** 
- Prevents diagonal movement from being faster than cardinal movement (maintains consistent top speed in all directions).

## Change: Physics Tuning (Snappy & Light)
**Decision:** Significantly increased acceleration and top speed.
**Reasoning:** 
- Aligned the implementation with the project specification's "Snappy & Light" model.
- **Acceleration:** 10,000 pixels/s² (was 1,000).
- **Top Speed:** 2,500 pixels/s (was 1,200).
- **Result:** Spin-up time to top speed is now ~250ms, making the window feel highly responsive.

## Implementation Details
- Refactored `src/app.rs` to manage `pressed_keys` state.
- Updated `src/physics.rs` default constants.
- Cleaned up redundant imports and unused logic in `src/app.rs`.
