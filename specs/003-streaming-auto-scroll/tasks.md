# Tasks: Streaming Content Auto-Scroll (Push-Down)

**Input**: Design documents from `specs/003-streaming-auto-scroll/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Tests are the PRIMARY implementation artifact — no new production code is needed per the research findings. The push-down behavior is emergent from the existing section-concatenation architecture.

## Format: `[ID] [P] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- All paths below are for the jcode harness at repo root

---

## Phase 1: Foundational (Verify Existing Behavior)

**Purpose**: Confirm the push-down behavior works architecturally, then add verification tests.

**⚠️ No new production code needed** — research confirms the section-concatenation model [Streaming, BatchProgress, Body, Header] naturally implements push-down. Adding lines to the Streaming section shifts all subsequent sections, and `scroll_offset` (absolute from the start) automatically references "further down" content as streaming grows.

- [ ] T001 [P] [US1] Add test `test_streaming_completed_message_persists` — verify committed message stays visible after new streaming enters, in `src/tui/app/tests/scroll_copy_03.rs`
- [ ] T002 [P] [US2] Add test `test_streaming_auto_follow_pegs_to_top` — verify scroll_offset stays 0 and streaming text visible at top during streaming growth, in `src/tui/app/tests/scroll_copy_03.rs`
- [ ] T003 [P] [US1] Add test `test_push_down_prior_content` — verify older completed body content shifts down (different rendered text) when streaming section grows while paused, in `src/tui/app/tests/scroll_copy_03.rs`
- [ ] T004 [US3] Add test `test_scroll_during_streaming_does_not_snap_back` — verify that scrolling away while streaming is in progress pauses auto-follow and viewport stays at user position, in `src/tui/app/tests/scroll_copy_03.rs`
- [ ] T005 [US4] Add test `test_submit_resets_to_top` — verify submitting a new prompt from any scroll position snaps to scroll_offset=0 and new streaming visible at top, in `src/tui/app/tests/scroll_copy_03.rs`
- [ ] T006 [P] Add test `test_up_down_indicator_accuracy_during_streaming` — verify ↑N/↓N indicators are correct when streaming is active and user is paused vs auto-following, in `src/tui/app/tests/scroll_copy_03.rs`

**Checkpoint**: 6 new tests exist and pass. All 11 existing scroll tests still pass.

---

## Phase 2: Cross-Cutting Verification

**Purpose**: Run the full test suite to confirm no regressions from existing top-down layout behavior.

- [ ] T007 Run `cargo test test_scroll -p jcode` — confirm all 17 tests pass (11 existing + 6 new)
- [ ] T008 Run `cargo check` — confirm no compilation warnings introduced

---

## Phase 3: Polish

**Purpose**: Update documentation and update spec status.

- [ ] T009 Update spec.md status to "Implemented" with completion date
- [ ] T010 Commit all changes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — can start immediately. All 6 test tasks are independent.
- **Cross-Cutting (Phase 2)**: Depends on Phase 1 tests being written and passing
- **Polish (Phase 3)**: Depends on Phase 2 verification

### Parallel Opportunities

- T001, T002, T003, T004, T005, T006 — all 6 tests are independent (different file paths, no shared state beyond `scroll_render_test_lock()` which is per-test)

### Parallel Example

```bash
# All 6 tests can be written in parallel (each is a separate function):
- "Add test_streaming_completed_message_persists"     → scroll_copy_03.rs
- "Add test_streaming_auto_follow_pegs_to_top"        → scroll_copy_03.rs
- "Add test_push_down_prior_content"                  → scroll_copy_03.rs
- "Add test_scroll_during_streaming_does_not_snap_back" → scroll_copy_03.rs
- "Add test_submit_resets_to_top"                     → scroll_copy_03.rs
- "Add test_up_down_indicator_accuracy_during_streaming" → scroll_copy_03.rs

# Then run all tests together:
cargo test test_scroll -p jcode
cargo test test_streaming -p jcode
```

---

## Implementation Strategy

### MVP First (Phase 1 only)

1. Write 6 test functions to `scroll_copy_03.rs`
2. Run `cargo test test_scroll -p jcode` — existing 11 tests must pass, 6 new run
3. Run `cargo check` — no new warnings
4. Commit

### Incremental Delivery

1. **Iteration 1**: T001, T002, T003 (US1+US2 — core push-down and auto-follow — P1 stories)
2. **Iteration 2**: T004 (US3 — manual scroll during streaming — P2)
3. **Iteration 3**: T005 (US4 — submit resets — P2)
4. **Iteration 4**: T006 (indicator accuracy — cross-cutting)
5. **Iteration 5**: T007, T008 (verification)
6. **Iteration 6**: T009, T010 (polish)

---

## Notes

- [P] tasks = different files or independent functions with no conflicts
- [Story] label maps task to specific user story for traceability
- Each user story is independently testable per the spec
- No new crates, dependencies, or modules
- All changes in `src/tui/app/tests/scroll_copy_03.rs` (+ downstream verification)
- If any test FAILS, it indicates an actual behavioral bug — NOT a test-in-writing problem
