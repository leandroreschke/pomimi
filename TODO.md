# TODO (Performance, Memory Footprint, Bugs)

This file is a structured backlog derived from an audit of the current codebase.

Conventions:
- Each TODO includes **Goal**, **Where**, and **Acceptance**.
- Keep changes pragmatic; avoid adding new third-party crates unless strictly necessary.

## P0 (High ROI / correctness)

- [x] **Fix SQL error handling in today focus time aggregation**
  - **Goal**: Do not silently swallow DB errors.
  - **Where**: `src/model.rs` (`Database::get_today_focus_time`)
  - **Notes**:
    - Current code uses `.unwrap_or(None)` on the `Result`, turning any error into `None`.
    - Prefer `await?` and map `NULL SUM` to `0`.
  - **Acceptance**:
    - Any SQL error is propagated to the caller.
    - Empty result still returns `0`.

- [x] **Replace GUI tick-decrement timer with monotonic-time based timer (fix drift)**
  - **Goal**: Timer must not drift when backgrounded or when ticks are delayed.
  - **Where**: `src/gui.rs` (`TimerState`, `Message::Tick`, `PomimiApp::subscription`)
  - **Notes**:
    - Current design decrements `remaining_secs -= 1` per tick.
    - Use `std::time::Instant` to derive remaining time from a start/deadline.
    - Update `session_focus_seconds` based on actual elapsed delta, not `+= 1`.
  - **Acceptance**:
    - If the app is stalled for N seconds, remaining time decreases by ~N seconds on next tick.
    - Pause/resume preserves correct remaining time.
    - No underflow/negative time; use saturating math.

- [x] **Log actual completed focus duration to `sessions` (not nominal duration)**
  - **Goal**: Persist real focus seconds for analytics/stats.
  - **Where**: `src/gui.rs` (phase completion logic in `Message::Tick`), `src/model.rs` (`Database::add_session` call sites)
  - **Notes**:
    - Current code logs `state.timer.total_secs` which is configuration, not actual elapsed.
  - **Acceptance**:
    - Stored session duration matches actual time spent focusing for that phase.

## P1 (Footprint + performance)

- [ ] **Reduce binary size: gate `iced` backend features by OS**
  - **Goal**: Avoid compiling unused windowing backends (esp. `x11`/`wayland` on macOS).
  - **Where**: `Cargo.toml`
  - **Notes**:
    - Current `iced` features include `x11` and `wayland` unconditionally.
    - Use target-specific dependency sections.
  - **Acceptance**:
    - macOS builds do not include `x11`/`wayland`.
    - Linux builds still work with relevant backends.

- [ ] **Sound playback: avoid repeated thread/process spawning storms**
  - **Goal**: Prevent unbounded concurrent sound plays and reduce overhead.
  - **Where**: `src/gui.rs` (`play_sound()`)
  - **Options**:
    - Single-flight/throttle: ignore play requests while a sound is already playing.
    - Single worker thread with a channel that lives for app lifetime.
  - **Acceptance**:
    - Rapid phase completion toggles cannot spawn unlimited threads/processes.

- [ ] **Reduce small UI allocations by caching formatted strings**
  - **Goal**: Avoid `format!()` allocations on every view/title render.
  - **Where**:
    - `src/gui.rs` (`title()`, `view_footer`, mini mode task rendering)
    - `src/components/timer.rs` (`format!("{:02}", ...)`)
  - **Acceptance**:
    - Formatted strings update only when underlying numeric values change.

## P2 (UX correctness / state machine edges)

- [ ] **Active task fallback when completing the active task**
  - **Goal**: Keep `active_task_id` meaningful (pick next task) without waiting on reload.
  - **Where**: `src/gui.rs` (`Message::ConfirmCompleteTask`, `Message::TasksLoaded`)
  - **Acceptance**:
    - Completing the active task selects another task if any exist.

- [ ] **Clarify require-confirmation semantics at phase boundary**
  - **Goal**: Confirm the UX is intended: you advance to next phase immediately but wait for user to start.
  - **Where**: `src/gui.rs` (`Message::Tick`)
  - **Acceptance**:
    - Behavior is explicit and tested (manual QA at least).

## P3 (Optional deeper cuts)

- [ ] **Evaluate persistence approach for footprint (sqlx is heavy)**
  - **Goal**: Reduce runtime memory and binary size.
  - **Where**: `Cargo.toml`, `src/model.rs`
  - **Notes**:
    - With current constraints (no new crates), options are limited.
    - If constraints are relaxed, consider a lighter DB layer.
  - **Acceptance**:
    - Documented tradeoffs and decision (keep vs replace).

## Execution plan suggestion

- [ ] **Milestone A (quick wins)**: SQL error propagation + iced feature gating + sound throttling
- [ ] **Milestone B (core correctness)**: monotonic timer + accurate session duration logging
- [ ] **Milestone C (polish)**: cache formatted strings + active task fallback + confirmation UX check
