# win-glide Project History: Movement Asymmetry Fix

## Issue: Asymmetric Movement Speed
**Symptoms:**
- The window moved significantly faster to the left and up than to the right and down.

**Root Cause:**
- In `apply_movement`, the calculation `new_rect.left = self.pos_x as i32` used floor truncation.
- When moving right (positive `x`), `self.pos_x` might be `100.9`, which truncates to `100`. The comparison `new_rect.left != self.window_rect.left` would then be false (since it was already `100`), preventing `SetWindowPos` from being called.
- When moving left (negative `x`), `self.pos_x` might be `99.1`, which truncates to `99`. The comparison would be true (since it was `100`), so the movement was applied immediately.
- This effectively "ate" sub-pixel movements in the positive direction but applied them in the negative direction.

**Fix:**
- Switched to `self.pos_x.round() as i32` for more balanced integer conversion.
- Removed the conditional `if new_rect.left != self.window_rect.left` check. If velocity is high enough to pass the threshold (`> 0.1`), we now always call `SetWindowPos` to ensure the OS and our internal state are perfectly synced.
- Ensured `pos_x` and `pos_y` are updated to match the final clamped/rounded integer positions to prevent drift.

## Result
- Movement speed is now perfectly symmetric in all cardinal and diagonal directions.
