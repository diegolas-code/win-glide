# win-glide Project History: Physics Refinement (Acceleration)

## Change: Slower Acceleration
**Decision:** Reduced acceleration from 4,000 to **3,000** pixels/s².
**Reasoning:** 
- The user liked the previous dual-friction change but wanted the build-up to top speed to be even more gradual.
- **Result:** Spin-up time to top speed is now approximately **1.33 seconds** (was 1s). This provides a more deliberate and heavy feel as the window gains momentum.

## Implementation Details
- Updated `src/physics.rs` default constants.
- No changes to logic were required.
