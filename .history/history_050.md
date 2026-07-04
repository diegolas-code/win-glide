# History Log: Customizable Resizing Physics in config.json

*   **Date:** 2026-07-04
*   **Feature:** Customizable Resizing Physics
*   **Branch:** `feat/tuned-resize-physics`

---

## Technical Decisions & Rationale

### 1. Optional nested `resize_physics` in `config.json`
*   **Problem:** To allow you to tune the resizing momentum to your exact preference (finding the right balance between snappiness and slippery glide), we need a way to customize individual parameters (acceleration, friction, thrust_friction, and top speed) without recompiling.
*   **Decision:** Expose a nested `resize_physics: Option<PhysicsConfig>` block in `config.json` and `src/config.rs`:
    *   **Fallback Logic:** If `resize_physics` is not specified, it automatically falls back to defaults scaled proportionally to `resize_speed` (`acceleration = resize_speed * 5.0`, `friction = 15.0`, `top_speed = resize_speed * 2.5`).
    *   **Explicit Override:** If specified, it overrides defaults completely.
*   **Default settings exposed in config.json:**
    ```json
      "resize_physics": {
        "acceleration": 3000.0,
        "friction": 15.0,
        "thrust_friction": 0.5,
        "top_speed": 1500.0
      }
    ```

### 2. Constructor Refactoring
*   **Decision:** Change `App::new` to accept `&Config` directly instead of listing individual physics/speed fields. This keeps constructor calls clean and scales with future configuration changes.

---

## Verification
*   **Unit Tests:** Added `test_config_deserialization_with_resize_physics` in `src/config.rs` to verify that custom properties are read correctly and the struct deserializes with success. All 20 tests pass.
