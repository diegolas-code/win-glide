# History Log - 023: Visual Tweaks - Header Reduction & Rounded Corners

## Context
The user requested two specific refinements to the overlay: reducing the vertical space above the window and rounding the corners of the tinted overlay.

## Technical Decisions

### 1. Header Reduction
The `OVERLAY_TOP_EXTENSION` was reduced from 10px to 7px. This constant affects both the rendering size of the overlay and its vertical offset from the target window.

### 2. Rounded Corners implementation
Instead of using `pixmap.fill()`, which fills the entire rectangular buffer, we switched to path-based rendering.
- **Path Construction:** Used `tiny_skia::PathBuilder` to construct a rounded rectangle using `line_to` and `quad_to` for the corners.
- **Corner Radius:** Set to a fixed 8.0f32, which provides a modern Windows 11-style look.
- **Anti-aliasing:** Enabled `Paint::anti_alias` to ensure smooth edges for the rounded corners.

## Changes

### `src/ui.rs`
- Reduced `OVERLAY_TOP_EXTENSION` to 7.
- Updated imports to include `FillRule`, `Paint`, `PathBuilder`, `Rect`, and `Transform`.
- Refactored `Overlay::redraw` to use `fill_path` with a rounded rect path instead of `pixmap.fill()`.

## Impact
- **Visuals:** The overlay now has a more tailored fit and a modern, polished appearance with rounded corners.
- **Performance:** Path-based rendering with anti-aliasing is slightly more intensive than a simple fill, but for a single full-screen tinted rectangle, the impact is negligible and well within the performance budget for a 120Hz loop.
