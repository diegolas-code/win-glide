# Pure White Text Alpha-Only Blending Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate the dimly tinted/gray background around the text by forcing all text pixels to pure white (`RGB = 255`) and using the alpha channel for anti-aliasing.

**Architecture:** GDI's font smoothing blends the white text with the destination buffer's black/orange channels, creating dark/gray pixels. By overriding the RGB bytes of all pixels where `b > 0` to `255`, we ensure the text is rendered in pure white, letting the updated alpha channel handle the smooth blending.

**Tech Stack:** Rust (2024), GDI post-processing.

---

### Task 1: Force Pure White RGB in Alpha Recovery Loop

**Files:**
- Modify: [src/ui.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/ui.rs)

- [ ] **Step 1: Update the alpha recovery loop inside prepare_surface**
  For each pixel where `b > 0`, override the `r`, `g`, and `b` channels to `255`:
  ```rust
                      let stride = width as usize * 4;
                      for y in scan_top..scan_bottom {
                          let row_offset = y * stride;
                          for x in scan_left..scan_right {
                              let offset = row_offset + x * 4;
                              let b = slice[offset];
                              let a = &mut slice[offset + 3];
                              if b > 0 {
                                  let intensity = b as f32 / 255.0;
                                  let bg_alpha = *a;
                                  *a = (bg_alpha as f32
                                      + (INDICATOR_OPACITY as f32 - bg_alpha as f32) * intensity)
                                      as u8;
                                  slice[offset] = 255;     // Blue
                                  slice[offset + 1] = 255; // Green
                                  slice[offset + 2] = 255; // Red
                              }
                          }
                      }
  ```

- [ ] **Step 2: Run tests and verify**
  Run: `cargo test`
  Expected: PASS

- [ ] **Step 3: Commit changes**
  Stage `src/ui.rs` and commit.
