# History Log: Resolve config.json Path relative to Executable

*   **Date:** 2026-07-16
*   **Feature:** Executable-Relative Configuration Path
*   **Branch:** `dev` (via `fix/config-path`)

---

## Technical Decisions & Rationale

### 1. Relative Working Directory Configuration Jitter
- **Problem:** Configuration files were loaded/saved using the relative string `"config.json"`. If the utility was executed from a startup folder, via task scheduler, or from another working directory, `config.json` was created in the current working directory instead of next to the running executable binary.
- **Decision:** Resolve the `config.json` path dynamically relative to the running binary.
- **Implementation:**
  - Create a private helper function `get_config_path()` in `src/config.rs` that reads the running executable path using `std::env::current_exe()` and targets `config.json` in its parent directory.
  - Update `Config::load()` and `Config::save()` to use this absolute path instead of a relative path string.

---

## Verification
- **Unit Tests:** Verified that deserialization and hotkey display tests continue to pass.
- **Manual Verification:** Checked that running `win-glide` from a different directory correctly loads the `config.json` located next to its executable.
