# win-glide Project History: Removing Monitor Edge Limits

## Change: Disabled Monitor Boundary Clamping
**Decision:** Removed the `clamp_to_work_area` logic to allow windows to move freely across the virtual desktop.
**Reasoning:** 
- The user requested that monitor edges should not limit window movement.
- Removing this restriction allows for more fluid movement across multi-monitor setups and even allows windows to be moved partially or fully off-screen if the user desires.

## Implementation Details
- Removed the `clamp_to_work_area` method from `src/app.rs`.
- Updated `apply_movement` in `src/app.rs` to stop calling the clamping logic.
- Ensured that window positions are still correctly updated and synced with the overlay.
