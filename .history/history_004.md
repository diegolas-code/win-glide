# win-glide Project History: Overlay Visual Refinement

## Change: Solid Tint Overlay
**Decision:** Replaced the 3px border with a full-window semi-transparent tint.
**Reasoning:** 
- Provides a more modern and integrated "active" session indicator.
- Simplifies the visual feedback by highlighting the entire window rather than just its edges.
- Maintains high performance by using a direct `pixmap.fill()` operation in `tiny_skia`.

**Implementation Details:**
- **Color:** Win-glide blue (`0, 120, 215`) at approximately 20% opacity (`alpha: 50`).
- **Cleanup:** Removed unused `Paint`, `PathBuilder`, and `Stroke` imports from `src/ui.rs`.
- **Interactivity:** Ensured the overlay remains `WS_EX_TRANSPARENT`, allowing all user interaction to pass through to the underlying window.

## Testing Note
- **Hotkey Registration:** Encountered `0x80070581` (Already registered) during unit tests. This is a known environmental issue when the main application or another test session is already running and holding the hotkey. It does not indicate a regression in the UI logic.
