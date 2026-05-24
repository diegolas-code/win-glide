# History Log - 024: Zero-Copy Rendering & Activation Optimization

## Context
While the visual refinements (rounded corners) were successful, they initially introduced a slight perceived latency during overlay activation. The goal was to eliminate this "instant" delay and achieve true zero-latency appearance.

## Technical Decisions

### 1. Zero-Copy Rendering Pipeline
The previous implementation rendered to an intermediate `tiny_skia::Pixmap` and then copied the bytes to a GDI DIB section. 
- **Optimization:** We now allocate the GDI DIB section first using `CreateDIBSection`. We then wrap the raw pointer to the bitmap bits in a `tiny_skia::PixmapMut` using `PixmapMut::from_bytes`.
- **Benefit:** `tiny-skia` renders directly into the memory that Windows uses for the layered window. This eliminates a full-frame copy (`std::ptr::copy_nonoverlapping`), saving millions of memory operations per frame.

### 2. Immediate Message Pumping
The hotkey is handled on a separate thread and communicated via a channel to the main loop.
- **Optimization:** After calling `activate_session`, the app now immediately calls `pump_messages()`.
- **Benefit:** This forces the Win32 subsystem to process the `UpdateLayeredWindow` and visibility changes instantly, rather than waiting for the next loop iteration or the 120Hz timer's sleep interval.

### 3. Source Color Optimization
The Red and Blue channels are swapped in the source `Paint` color.
- **Benefit:** This aligns `tiny-skia`'s output directly with Win32's `BGRA` memory layout, avoiding an in-place byte swap loop.

## Changes

### `src/ui.rs`
- Refactored `Overlay::redraw` to use `PixmapMut::from_bytes`.
- Eliminated `std::ptr::copy_nonoverlapping`.
- Ensured `pixmap.fill(Color::TRANSPARENT)` is used to clear the potentially uninitialized GDI memory.

### `src/app.rs`
- Added `self.pump_messages()` immediately after `self.activate_session()` in the hotkey handler.
- Verified that `self.overlay.show(true)` is necessary and maintained its use alongside the optimized `redraw`.

## Impact
The overlay now appears with zero perceived latency, even on high-resolution displays, while significantly reducing CPU and memory bus pressure during activation.
