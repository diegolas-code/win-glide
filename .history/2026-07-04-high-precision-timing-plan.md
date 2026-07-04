# High-Precision Frame Timing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement scheduler resolution tuning and a hybrid sleep/yield frame rate capping system to achieve liquid-smooth 120 FPS resizing and translation.

---

### Task 1: Implement TimerResolutionGuard in main.rs

**Files:**
- Modify: [src/main.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/main.rs)

- [ ] **Step 1: Declare TimerResolutionGuard**
  Add the RAII helper struct and system FFI declarations to `src/main.rs`:
  ```rust
  /// RAII guard to set Windows timer resolution to 1ms on startup and restore it on shutdown.
  struct TimerResolutionGuard;

  impl TimerResolutionGuard {
      fn new() -> Self {
          #[link(name = "winmm")]
          extern "system" {
              fn timeBeginPeriod(uPeriod: u32) -> u32;
          }
          unsafe {
              let _ = timeBeginPeriod(1);
          }
          Self
      }
  }

  impl Drop for TimerResolutionGuard {
      fn drop(&mut self) {
          #[link(name = "winmm")]
          extern "system" {
              fn timeEndPeriod(uPeriod: u32) -> u32;
          }
          unsafe {
              let _ = timeEndPeriod(1);
          }
      }
  }
  ```

- [ ] **Step 2: Instantiate guard in main**
  Create an instance of `TimerResolutionGuard` as the very first line of `main`:
  ```rust
  fn main() -> windows::core::Result<()> {
      let _timer_guard = TimerResolutionGuard::new();
  ```

- [ ] **Step 3: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 4: Commit**
  ```bash
  git add src/main.rs
  git commit -m "feat: implement TimerResolutionGuard to request 1ms scheduler resolution on Windows"
  ```

---

### Task 2: Implement Hybrid Sleep/Yield Frame Rate Capping in app.rs

**Files:**
- Modify: [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs)

- [ ] **Step 1: Modify frame rate capping in App::run**
  Update the capping block at the end of the `run` loop inside [src/app.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs):
  ```rust
              // Cap the frame rate.
              let elapsed = now.elapsed();
              if elapsed < frame_duration {
                  let sleep_dur = frame_duration - elapsed;
                  if sleep_dur.as_millis() > 2 {
                      std::thread::sleep(sleep_dur - std::time::Duration::from_millis(1));
                  }
                  while now.elapsed() < frame_duration {
                      std::thread::yield_now();
                  }
              }
  ```

- [ ] **Step 2: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 3: Commit**
  ```bash
  git add src/app.rs
  git commit -m "perf: implement hybrid sleep/yield frame rate capping loop"
  ```
