# win-glide Project History: v0.1.1 Troubleshooting

## Issue: Window Hang & Movement Failure
**Symptoms:**
- Application hotkey triggers correctly.
- Overlay border appears around the window.
- **Problem:** The window (or overlay) turns black and shows an hourglass cursor (Not Responding).
- **Problem:** Arrow keys do not move the window.

## Root Cause Investigation
1.  **Hanging Window:** The `Overlay` window was created on the main thread, but the main `App::run` loop was just a `loop` with `std::thread::sleep`. It lacked a Win32 message pump (`GetMessage`/`PeekMessage`). Win32 requires the owning thread to process messages for its windows to remain responsive.
2.  **Movement Failure:** Position accumulation was being done using `i32` and `RECT`. With a ~120Hz loop and `0.008s` delta time, frame-by-frame movement (`velocity * dt`) was often less than 1.0, which truncated to `0` when cast to `i32`. This prevented any movement from accumulating.

## Implemented Fixes
- **Message Pump:** Added `pump_messages` using `PeekMessageW` to the main `App::run` loop. This keeps the overlay responsive and fixes the "black window" issue.
- **High-Precision Position:** Refactored `App` to track window position using `f32` (`pos_x`, `pos_y`). Movement is accumulated in `f32` and only cast to `i32` when calling `SetWindowPos`.
- **Timing Fix:** Passed the actual loop `dt` to the physics engine instead of using a hardcoded placeholder.
- **Diagnostics:** Added `println!` logging to the `App` loop to trace hotkey activation and key presses.

## Troubleshooting Advice
- Always ensure the thread that creates a Win32 window pumps its message queue.
- Use floating point types for accumulating movement in high-frequency physics loops to avoid integer truncation errors.
