# win-glide Project History: Graceful Shutdown & Ctrl+C Handling

## Change: Graceful Shutdown Mechanism
**Decision:** Implemented a console control handler to catch `Ctrl+C` and signal a clean exit.
**Reasoning:** 
- Terminating with `Ctrl+C` without a handler often results in abrupt termination and potential non-zero exit codes.
- A graceful shutdown allows the `App` to clean up resources (like the overlay) and exit the main loop naturally.

## Implementation Details
- Added `Win32_System_Console` feature to `Cargo.toml`.
- Added `InputEvent::Shutdown` to signal the application to close.
- Implemented `register_shutdown_handler` in `src/input.rs` using `SetConsoleCtrlHandler`.
- Updated `App::run` in `src/app.rs` to use a `while self.running` loop.
- Updated `main.rs` to initialize the shutdown handler and provide user feedback.
