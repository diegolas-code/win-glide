# Specification: Discrete Resizing Steps and Overlay Size Synchronization

This specification defines the behavior, architecture, and alignment logic for discrete keyboard-driven window resizing in `win-glide`.

## 1. Discrete Steps Control Loop

Instead of continuous physics-based updates in the main 120Hz thread loop, resizing will be triggered on discrete keypress events:
*   Resizing actions are processed inside the `process_events` event handler when receiving `InputEvent::KeyDown(vk)`.
*   If `Shift` (Expand) or `Alt` (Shrink) is held down during an arrow key press, translation momentum is zeroed out (`self.physics.velocity = Vector2D::default()`), and a discrete resize step is performed.
*   By executing on `InputEvent::KeyDown`, the resizing naturally utilizes the OS-native keyboard repeat rate and delay when arrow keys are held down, offering a snappy and responsive experience.

### Step Size Calculation
The resizing step size in pixels is dynamically derived from the user's `resize_speed` configuration:
$$\text{step\_size} = \max\left(10.0, \text{round}\left(\frac{\text{resize\_speed}}{12.0}\right)\right)$$
For the default `resize_speed` of `600.0`, the step size is exactly `50` pixels.

---

## 2. Overlay & Target Window Size Synchronization

To resolve the issue where the overlay continues to shrink even when the target window has hit its application-internal minimum sizing limits:
1.  **Set Window Size:** Perform the `SetWindowPos` call on the target window using the calculated layout rectangle.
2.  **Query Actual Size:** Immediately query the target window's resulting bounds using `GetWindowRect(hwnd)`.
3.  **Update State:** Update the internal state (`pos_x`, `pos_y`, `width_f32`, `height_f32`, and `window_rect`) to match the **actual** bounds returned by the OS.
4.  **Repaint Overlay:** Redraw the overlay using the synced actual bounds.

This guarantees the overlay never detaches or mismatch-sizes when the target window hits its sizing limits.
