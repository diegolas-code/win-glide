# High-Performance Resize Cache Optimization Spec

## Overview
This document describes the design for optimizing overlay rendering during continuous resizing. Currently, continuous resizing changes the target window size on every frame, resulting in GDI Device Context and DIB section re-allocations on every frame (cache misses). This design introduces over-allocated buffer growth to achieve 100% cache hit rates during active resizing.

## Architecture
1. **Capacity-Based Buffer Caching:**
   Modify the GDI cache validation logic in `prepare_surface`. Instead of verifying exact dimensions (`cache_w == width && cache_h == height`), verify capacity (`cache_w >= width && cache_h >= height`).
   
2. **Buffer Over-Allocation:**
   On a cache miss (when the target window grows larger than the cached buffer), allocate a DIB section that is `width + 256` by `height + 256` pixels. This ensures subsequent growth of up to 256px along either axis is absorbed without re-allocating GDI handles.

3. **Stride-Safe Rendering:**
   Instantiate the `tiny-skia` `PixmapMut` using the over-allocated dimensions (`cache_w` and `cache_h`) to maintain byte alignment and pitch constraints. Restrict paths, chevrons, and help text layouts to the actual window bounds (`width` and `height`).

4. **Sub-rect Layered Window Commits:**
   Call `UpdateLayeredWindow` using the actual window bounds (`width` and `height`) as the destination size, copying only the active top-left portion from the larger cached DC.
