# History Log: Overlay Help Legends

*   **Date:** 2026-07-04
*   **Feature:** Overlay Help Legends
*   **Branch:** `feat/overlay-help-legends`

---

## Technical Decisions & Rationale

### 1. Single Source of Truth Opacity
*   **Problem:** Ensure that both the chevron indicators and the help text legends share the exact same opacity, rather than using separate magic numbers.
*   **Decision:** Define `pub const INDICATOR_OPACITY: u8 = 204;` in `src/ui.rs`. This value (80% opacity) is used directly in `tiny-skia` color configurations for chevrons and in the alpha recovery post-processing loop for the text.

### 2. GDI Text Drawing & Safe Bounds
*   GDI text rasterization using `CreateFontW` and `DrawTextW` is utilized directly on the DIB section's Device Context (`mem_dc`) to bypass `tiny-skia`'s lack of text support.
*   We define a safe text rect that is indented from the window edges:
    ```rust
    let target_left = (margin + arrow_size + 10.0) as i32;
    let target_right = (width as f32 - margin - arrow_size - 10.0) as i32;
    ```
    This prevents the text from overlapping the border or the chevrons on all sides.
*   We use a two-pass centering layout algorithm:
    1.  Call `DrawTextW` with `DT_CALCRECT` to compute the required height of the word-wrapped text.
    2.  Calculate the vertical offset to center the text block, then call `DrawTextW` to draw it.

### 3. Grayscale Smoothing & Alpha Reconstruction
*   **Problem:** Windows subpixel ClearType anti-aliasing can cause color fringing (red/blue borders) when rendered onto transparent backgrounds, resulting in a pixelated look. Additionally, GDI text drawing writes `0` to the alpha channel of 32-bit DIB sections.
*   **Decision:** Use `ANTIALIASED_QUALITY` (value `4` for grayscale anti-aliasing) in `CreateFontW`. This creates smooth grayscale outlines rather than subpixel color patterns.
*   **Alpha Recovery:** We run a post-processing loop over the raw DIB pixel bytes to restore the correct alpha channel based on the blue channel (`b`) intensity. Since the background (both transparent black and orange) has a blue value of `0`, the blue channel is a direct measure of the white text's intensity:
    ```rust
    for offset in (0..slice.len()).step_by(4) {
        let b = slice[offset];
        let a = &mut slice[offset + 3];
        if b > 0 {
            let intensity = b as f32 / 255.0;
            let bg_alpha = *a;
            *a = (bg_alpha as f32 + (INDICATOR_OPACITY as f32 - bg_alpha as f32) * intensity) as u8;
        }
    }
    ```
    This leaves GDI's blended RGB channels untouched while setting the alpha channel dynamically.

### 4. Dynamic Movement Text State & Brackets
*   **Default Idle State (No key pressed):**
    ```
    Press [Arrow keys] to move the window around.
    Press [Shift] and [Arrow keys] to resize the window up.
    Press [Alt] and [Arrow keys] to resize the window down.
    ```
*   **Active Move State (Arrow key pressed without modifier):**
    ```
    Press the [Arrow keys] in the direction you want to move the window to
    ```
    *   **Arrow Detection Fix:** Because the low-level keyboard hook blocks arrow keys from reaching the OS, `GetAsyncKeyState` queries return `false`. We check against the application's internal `self.pressed_keys` set instead to reliably detect active movement.
*   **Active Expand/Shrink States:** Bracketed names like `[Arrow keys]`, `[Shift]`, and `[Alt]` are styled with regular font weight (`400` / `FW_NORMAL`).

---

## Verification
*   **Unit Tests:** Expanded `test_overlay_arrow_rendering` in `src/ui.rs` to cover all three help text states (No modifiers, Shift active, Alt active). All tests pass successfully.
