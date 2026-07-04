# High-Precision Frame Timing Spec

## Overview
This document describes the design for achieving microsecond-accurate loop intervals on Windows to eliminate frame-time jitter during active window rendering (such as overlay resizing and movement animation).

## Architecture
1. **OS Scheduler Resolution Tuning:**
   By default, Windows scheduler timer resolution is set to 15.6ms. This causes calls to `std::thread::sleep` to overshoot by up to 15ms, introducing major stuttering at high frame rates (120Hz = 8.33ms frames). We introduce a `TimerResolutionGuard` RAII structure on startup that invokes `timeBeginPeriod(1)` to request a 1ms scheduler resolution, and automatically calls `timeEndPeriod(1)` on application exit.

2. **Hybrid Sleep/Yield Limiter:**
   To cap the loop at exactly 120Hz (8.33ms) without scheduler jitter, we implement a hybrid frame rate capping model in `App::run`:
   * If the remaining time in the frame is greater than 2ms, call `std::thread::sleep` for `remaining_time - 1ms`.
   * For the remaining portion (< 1ms), busy-loop using `std::thread::yield_now()` until the exact frame time is reached.
   This guarantees sub-millisecond precision, maintaining perfectly steady 8.33ms tick rates with near-zero CPU overhead.
