# win-glide Project History: Overlay Top Extension

## Change: Top-Side Overlay Extension
**Decision:** Extended the overlay by 10 pixels above the target window's top border.
**Reasoning:** 
- Creates a "header" effect that makes it clearer which window is active, even if the window title bar is visually busy.
- Enhances the sense of the window being "captured" or "managed" by win-glide.

**Implementation Details:**
- Added `TOP_EXTENSION: i32 = 10` constant in `src/ui.rs`.
- Updated `redraw` to increase pixmap height and shift `pt_dst.y` upwards by `TOP_EXTENSION`.
- Updated `update_position` to include the offset and height increase during window movement.
