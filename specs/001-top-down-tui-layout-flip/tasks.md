# Tasks: Top-Down TUI Layout Flip

**Input**: `spec.md`, `research.md`, `data-model.md`, `plan.md`, `quickstart.md`
**Prerequisites**: plan.md (required), spec.md (required)
**Tests**: Update existing test assertions after fixing scroll logic

## Phase 1: Section Order (B1) 🎯 MVP

**Goal**: Reverse section order so newest content appears first in the content array

**Independent Test**: After change, content array dumps show `[Streaming, BatchProgress, Body, Header]` instead of `[Header, Body, BatchProgress, Streaming]`

- [ ] T001 [US1] Reverse section order in `prepare_messages_inner()` at `src/tui/ui_prepare.rs:458-463`

**Checkpoint**: Section order reversed. All absolute offsets recalculated by `from_sections()`.

---

## Phase 2: Scroll Math Fix (B4, B5, B6)

**Goal**: Simplify all scroll functions to use direct `scroll_offset` without inversion math

**Independent Test**: Press Ctrl+J (scroll up) increases `scroll_offset`, Ctrl+K (scroll down) decreases `scroll_offset` back to 0. Auto-follow resumes at 0.

- [ ] T002 [US1] Fix `scroll_up()` in `src/tui/app/navigation.rs:1053-1068` — remove `max - offset` inversion, use direct addition
- [ ] T003 [US1] Fix `scroll_down()` in `src/tui/app/navigation.rs:1086-1100` — use `saturating_sub` instead of addition, resume auto-follow at `scroll_offset == 0`
- [ ] T004 [US1] Fix `pause_chat_auto_scroll()` in `src/tui/app/navigation.rs:1070-1084` — remove inversion math, just set `auto_scroll_paused = true`

**Checkpoint**: Scroll navigation uses direct offset. No `max - offset` anywhere.

---

## Phase 3: Auto-Follow Anchor Fix (B2, B3)

**Goal**: Auto-follow shows newest content at top (scroll = 0) instead of oldest (scroll = max_scroll)

**Independent Test**: Start conversation, new streaming content appears right below the input box at the top of viewport.

- [ ] T005 [P] [US1] Fix auto-follow anchor in `src/tui/ui_viewport.rs:246-250` — change `max_scroll` to `0` when not paused
- [ ] T006 [P] [US1] Fix native scrollbar position in `src/tui/app/handterm_native_scroll.rs:122-126` — change `max_scroll` to `0` when not paused

**Checkpoint**: Auto-follow anchors at newest content.

---

## Phase 4: Test Assertion Fix (B7)

**Goal**: Tests match the corrected scroll semantics

**Independent Test**: `cargo test` passes all scroll tests

- [ ] T007 [US1] Fix `scroll_copy_03.rs` — change `second_offset < first_offset` to `second_offset > first_offset` in `src/tui/app/tests/scroll_copy_03.rs:22-25`
- [ ] T008 Run full test suite: `cargo test -p jcode 2>&1 | tail -50`

---

## Phase 5: Visual Verification

**Goal**: Confirm the fix works in the actual TUI

**Independent Test**: Launch jcode, verify input at top, new content below it, scroll behavior correct

- [ ] T009 Build and launch jcode, verify:
  - [ ] Input box at top of screen
  - [ ] New content appears right below input
  - [ ] Ctrl+J scrolls to older content (offset increases)
  - [ ] Ctrl+K scrolls back to newest (offset reaches 0, auto-follow resumes)
  - [ ] `↑N` indicator shows when scrolled away from newest
