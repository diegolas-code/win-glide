# win-glide Project History: Refined Visibility and Timeout

## Change: Increased Off-Screen Visibility Margin
**Decision:** Increased the minimum visible portion of the window from 50px to **150px**.
**Reasoning:** 
- 50px felt too small for easily identifying or retrieving windows moved partially off-screen.
- 150px provides a much more substantial "handle" while still allowing for significant off-screen positioning.

## Change: Increased Idle Timeout
**Decision:** Increased the session idle timeout from 3 seconds to **5 seconds**.
**Reasoning:** 
- 3 seconds was occasionally too short, causing unintended session deactivations during brief pauses in input.
- 5 seconds provides a better buffer for the user to think or reposition without losing the glide session.

## Implementation Details
- Updated `check_exit_conditions` in `src/app.rs` to use `Duration::from_secs(5)`.
- Updated `apply_movement` in `src/app.rs` to use `min_visible = 150`.
