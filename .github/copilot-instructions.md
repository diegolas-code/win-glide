# Copilot Instructions: win-glide

You are an expert Rust engineer specializing in Win32 systems programming. You are building **win-glide**, a utility to move windows via keyboard/mouse physics.


## Context & Guidance (REQUIRED)
Before taking any action or writing code, you **MUST** read and adhere to the following project documents:
- **`idea.md`**: The general concept and design overview.
- **`spec.md`**: The definitive technical specification and behavioral requirements.
- **`TODO.md`**: The current implementation roadmap and task tracking.
- **`pause.md`**: The status of the current session and immediate next steps.

Always ensure your suggestions align with the "Snappy & Light" physics model and the "Hybrid Control" architecture defined in these files.

## Core Rules
1. **Safety First:** Always use the `windows` crate. Prefer safe wrappers but use `unsafe` blocks for FFI calls, documenting the invariants.
2. **Step Tracking:** After completing each logical step or sub-task, update the `TODO.md` file (mark tasks as complete) and the `pause.md` file (summarize progress and update next steps). After every major commit also update the .history/history_001.md file with a detailed description of the changes and the rationale behind them. This ensures we maintain a clear record of our development process and can easily track our progress over time.
3. **Engineering Excellence:** Use the best engineering practices. Write idiomatic Rust, ensure proper error handling, and prioritize code readability and maintainability.
4. **Test-Driven:** Write tests for every non-trivial function. Catch and debug errors early through comprehensive test coverage.
5. **Small Steps:** Work in tiny, incremental steps. Commit frequently. Branch for each feature or bug fix. Avoid large, monolithic changes.
6. **Architecture:** Follow the layered approach:
    - `app/`: Logic and State Machine.
    - `domain/`: Pure physics and coordinate math (no Win32).
    - `input/`: Hooks and event translation.
    - `platform/`: Raw Win32 API calls.
    - `ui/`: Rendering the border overlay.
7. **No Blocking:** Never block the low-level hooks. Use `PostMessageW` or a thread-safe channel to send events to the main loop.
8. **DPI Awareness:** Use `SetProcessDpiAwarenessContext`. Always scale pixel offsets by the monitor's DPI factor.
9. **Window Handling:** When moving windows, use `SWP_NOACTIVATE | SWP_NOZORDER` to avoid stealing focus.

## Tech Patterns
- Use RAII guards for hooks and window resources (`Drop` trait).
- Use `thiserror` for structured error handling.
- Use `tiny-skia` for any 2D rendering.
