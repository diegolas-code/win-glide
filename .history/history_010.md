# win-glide Project History: Dual-Friction Physics Model

## Issue: Perceived Lack of Acceleration & Low Speed
**Symptoms:**
- The user reported not seeing any acceleration while holding keys.
- The final movement speed felt slow even with a high `top_speed` setting.

**Root Cause:**
- The physics model used a single high friction value (10.0) applied every frame.
- This created a low "terminal velocity" (`acceleration / friction`). With 6,000 acceleration and 10 friction, the window capped at 600 pixels/s, far below the intended 4,000.
- Because it hit this low terminal velocity almost instantly, no acceleration was perceived.

**Fix: Dual-Friction Model**
- Introduced `thrust_friction` in `PhysicsConfig`.
- **Thrusting:** While keys are held, a very low friction (0.5) is applied. This allows the terminal velocity to reach 8,000 pixels/s, meaning the 4,000 `top_speed` limit is reachable.
- **Coasting:** When keys are released, the original high friction (10.0) is applied. This ensures the window still stops quickly (within ~100ms), maintaining the "snappy" feel.
- **Tuned Constants:** Set acceleration to 4,000 pixels/s² and top speed to 4,000 pixels/s. This results in a clear 1-second build-up to maximum velocity.

## Implementation Details
- Updated `src/physics.rs` to support conditional friction in `update`.
- Updated `src/app.rs` to track thrusting state and pass it to the physics engine.
- Verified with unit tests.
