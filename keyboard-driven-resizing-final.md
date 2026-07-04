# Specification & Implementation Design: Keyboard-Driven Window Resizing

This document defines the final design, architecture, and step-by-step guidelines for implementing fine-grained, keyboard-driven window resizing in [win-glide](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide). 

---

## 1. Feature Overview & Control Scheme

During an active [win-glide](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide) session (activated via `Ctrl + Alt + F10`), arrow key actions are modified using absolute and fixed keyboard modifiers:

| Key Combination | Action | Description |
|---|---|---|
| **Arrow Key** | Glide (Move) | Window moves dynamically using standard physics. |
| **`Alt` + Arrow Key** | Expand (Grow) | Window grows outward from the edge matching the arrow direction. |
| **`Shift` + Arrow Key** | Shrink (Pull In) | Window shrinks inward from the edge matching the arrow direction. |

---

## 2. Configuration (`config.json`)

To allow users to calibrate the resizing feel, a new configuration parameter is introduced:

### A. Schema Updates
Add `resize_speed` at the root level of [config.json](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/config.json):
```json
{
  "physics": {
    "acceleration": 4000.0,
    "friction": 10.0,
    "thrust_friction": 0.5,
    "top_speed": 4000.0
  },
  "resize_speed": 600.0,
  "hotkey": {
    "modifiers": 3,
    "vk": 121
  },
  "center_hotkey": {
    "modifiers": 9,
    "vk": 67
  }
}
```

### B. Code changes in [config.rs](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/config.rs)
Update the [Config](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/config.rs#L13) struct:
```rust
pub struct Config {
    pub physics: PhysicsConfig,
    pub resize_speed: f32, // New field
    pub hotkey: HotkeyConfig,
    pub center_hotkey: HotkeyConfig,
}
```
*   **Default Value:** `600.0` pixels per second.

---

## 3. Architecture & State Management

The resizing system uses high-precision size accumulators to avoid truncation errors during low-velocity updates.

### A. State Additions in [App](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/app.rs#L23) (`src/app.rs`)
Add high-precision sizing fields to track fractional dimension changes:
*   `width_f32: f32` – Fractional window width accumulator.
*   `height_f32: f32` – Fractional window height accumulator.

*These fields must be initialized in `App::activate_session` using the window's starting bounding rectangle.*

### B. Control Flow & Handoff Integration
1.  **Real-Time Modifier Checking**: Query key states using `GetAsyncKeyState` for `VK_MENU` (Alt) and `VK_SHIFT` (Shift) directly in the main loop thread to capture instant hardware states.
2.  **Velocity Reset**: If a resize key combination (`Alt` or `Shift` + Arrow) is actively processed in a frame, immediately reset `self.physics.velocity` to `0.0` (both `x` and `y`). This prevents translational momentum from drifting the window during resizing.

---

## 4. Coordinate Calculations & Math

Resizing step sizes are frame-rate independent:
$$\text{step} = \text{config.resize\_speed} \cdot dt$$

### A. Expansion (`Alt` held, no `Shift`)
*   **Right Arrow:** Grows right edge.
    $$\Delta \text{width} = \text{step}$$
*   **Left Arrow:** Grows left edge.
    $$\Delta \text{pos\_x} = -\text{step}, \quad \Delta \text{width} = \text{step}$$
*   **Down Arrow:** Grows bottom edge.
    $$\Delta \text{height} = \text{step}$$
*   **Up Arrow:** Grows top edge.
    $$\Delta \text{pos\_y} = -\text{step}, \quad \Delta \text{height} = \text{step}$$

### B. Shrinkage (`Shift` held, no `Alt`)
*   **Right Arrow:** Pulls right edge left.
    $$\Delta \text{width} = -\text{step}$$
*   **Left Arrow:** Pulls left edge right.
    $$\Delta \text{pos\_x} = \text{step}, \quad \Delta \text{width} = -\text{step}$$
*   **Down Arrow:** Pulls bottom edge up.
    $$\Delta \text{height} = -\text{step}$$
*   **Up Arrow:** Pulls top edge down.
    $$\Delta \text{pos\_y} = \text{step}, \quad \Delta \text{height} = -\text{step}$$

---

## 5. Safety Bounds & Clamping Rules

Before applying the calculated position or size to the window, the boundary rectangle must satisfy:

1.  **Minimum Size Floor (Hardcoded and Scaled):**
    The window dimensions must never collapse below:
    $$\text{min\_w} = 250 \cdot \text{scale\_factor}, \quad \text{min\_h} = 250 \cdot \text{scale\_factor}$$
    *Where $\text{scale\_factor} = \text{dpi} / 96.0$.*
2.  **Monitor Work Area Boundary:**
    Expansion must not grow the window outside the current monitor's `WorkArea` (respecting taskbars). If an edge hits the boundary, expansion in that direction is blocked.
3.  **Off-Screen Parking Constraints:**
    Shrinking or shifting must not push the window off-screen beyond the virtual screen margins. At least `150px` of the window must remain visible at all times.

---

## 6. UI & Overlay Sync Optimization

1.  **Coordinated Layout Updates**: Synchronously update both the target window and the overlay inside a single Win32 transaction:
    ```rust
    if let Ok(hdwp) = BeginDeferWindowPos(2) {
        let mut hdwp = hdwp;
        if let Ok(h) = DeferWindowPos(hdwp, target_hwnd, HWND::default(), new_x, new_y, new_w, new_h, SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOCOPYBITS) {
            hdwp = h;
        }
        if let Ok(h) = self.overlay.defer_update_position(hdwp, new_rect) {
            hdwp = h;
        }
        let _ = EndDeferWindowPos(hdwp);
    }
    ```
2.  **Memory & Allocation Guard**: Creating GDI DIB sections in [Overlay::redraw](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/ui.rs#L120) has rendering overhead. To avoid thrashing allocations, only invoke the redraw pipeline when the target window's integer width or height changes. If the window only translates (moves) without resizing, skip redrawing the bitmap.

---

## 7. Step-by-Step Developer Checklist

- [ ] **Step 1: Input Hook Passthrough (`src/input.rs`)**
  Update [keyboard_proc](file:///C:/Users/Diegolas/Code/rust/KEYBOARDPAL/win-glide/src/input.rs#L127) to allow `VK_MENU` (Alt) and `VK_SHIFT` (Shift) to propagate normally, while still capturing arrow keys during active sessions.
- [ ] **Step 2: Configuration Deserialization (`src/config.rs`)**
  Add `resize_speed: f32` to the `Config` struct and define the default value as `600.0`.
- [ ] **Step 3: App State Setup (`src/app.rs`)**
  Add `width_f32` and `height_f32` to `App`. Initialize these values on session activation.
- [ ] **Step 4: Resize Detection & Glide Handoff (`src/app.rs`)**
  Use `GetAsyncKeyState` to query the modifier status. If resizing is active, set `self.physics.velocity = Vector2D::default();` to zero translation momentum.
- [ ] **Step 5: Boundary Clamping & Safety (`src/app.rs` or `src/window.rs`)**
  Implement the delta coordinates. Apply the $250\text{px}$ minimum size floor (multiplied by the DPI factor), virtual desktop off-screen limitations ($150\text{px}$ visible area), and active monitor work area bounds.
- [ ] **Step 6: Coordinated UI Refresh (`src/app.rs`)**
  Use `BeginDeferWindowPos` to reposition and resize the window and overlay together. Only invoke `Overlay::redraw` if the dimensions changed.
- [ ] **Step 7: Testing**
  Write unit tests to verify resizing calculations and clamping logic under mock configurations.
