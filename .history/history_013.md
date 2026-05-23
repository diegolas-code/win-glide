# win-glide Project History: Limited Off-Screen Movement

## Change: Restricted Off-Screen Gliding
**Decision:** Modified window movement to allow partial off-screen positioning while ensuring a minimum level of visibility.
**Reasoning:** 
- The user wanted to move windows off-screen but maintain at least 50 pixels of the window within the visible virtual desktop area.
- This prevents windows from being completely lost while still providing the freedom of the previously implemented "no-border-limit" feature.

## Implementation Details
- Added `Platform::get_virtual_screen_rect()` to retrieve the bounding box of all connected monitors using `GetSystemMetrics`.
- Updated `apply_movement` in `src/app.rs` to clamp the window's destination `RECT`.
- **Constraint:** The window can move at most `width - 50` or `height - 50` beyond the virtual screen edges.
- Cleaned up unused GDI imports in `src/app.rs`.

## Result
- Windows can now be "parked" partially off-screen, with exactly 50px remaining visible as a handle to drag them back.
