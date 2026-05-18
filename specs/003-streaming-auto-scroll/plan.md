# Implementation Plan: Streaming Content Auto-Scroll (Push-Down)

**Branch**: `003-streaming-auto-scroll` | **Date**: 2026-05-18 | **Spec**: specs/003-streaming-auto-scroll/spec.md
**Input**: Feature specification from `specs/003-streaming-auto-scroll/spec.md`

## Summary

When a streaming response finishes, the completed text must stay visible below the input box. When a new streaming response enters, it must push the completed text downward — mimicking a natural chat feed. Auto-scroll pegs to the top during streaming, pauses on manual scroll, and resets on submit.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)  
**Primary Dependencies**: ratatui 0.29+, cargo (project is jcode harness)  
**Storage**: N/A (in-memory UI state only)  
**Testing**: `cargo test` (built-in unit tests via `#[test]` in TUI source files; scroll tests in `src/tui/app/tests/scroll_copy_03.rs`)  
**Target Platform**: Terminal (cross-platform: Linux, macOS, Windows)  
**Project Type**: TUI desktop app (single Rust binary)  
**Performance Goals**: <16ms per render frame (<60fps); streaming append should not cause visible jank  
**Constraints**: `scroll_offset` runs from 0 (newest) to max (oldest); sections order is [Streaming, BatchProgress, Body, Header]; `auto_scroll_paused=true` means user has manually scrolled  
**Scale/Scope**: Single conversation session; messages from 1 to 10k+ lines  

### Architecture Context

The rendering pipeline flows: `App state → prepare_messages_inner() → PreparedChatFrame` (4 sections concatenated) → `draw_messages()` (viewport slice + indicators + prompt preview). The `Streaming` section uses `app.streaming_text()` and is only non-empty when `is_processing() && !streaming_text.is_empty()`. When `commit_pending_streaming_assistant_message()` runs, it moves `streaming_text` into `display_messages` (Body section). The `scroll_offset` is absolute line offset from the start of the combined sections.

### Key APIs

| API | File | Role |
|-----|------|------|
| `App::follow_chat_top()` | navigation.rs | Sets scroll_offset=0, auto_scroll_paused=false |
| `App::scroll_up(amount)` | navigation.rs | Increases offset (older content) |
| `App::scroll_down(amount)` | navigation.rs | Decreases offset (newer content), resumes at 0 |
| `App::submit_input()` | input.rs | Calls follow_chat_top() then commit_pending+send |
| `commit_pending_streaming_assistant_message()` | input.rs | Moves streaming_text → display_messages |
| `prepare_messages_inner()` | ui_prepare.rs | Builds [Streaming, BatchProgress, Body, Header] |
| `draw_messages()` | ui_viewport.rs | Slices from scroll_offset for viewport + indicators |

### Potential Problem Areas

1. **Section transition flicker**: When streaming completes and moves from Streaming→Body, the prepared lines may differ slightly because streaming uses `prepare_streaming_cached` (raw wrapping) while committed messages use `prepare_body` (full markdown rendering). Line count could differ by 1-2 lines → content shift.

2. **Auto-scroll resume timing**: `follow_chat_top()` runs in `submit_input` BEFORE `commit_pending_streaming_assistant_message()`. If the next streaming response starts before the next render, the user sees a flash of "committed content at top" before "streaming text at top".

3. **Streaming growth while paused**: When auto_scroll_paused=true and streaming grows, the content above the viewport increases but scroll_offset is unchanged. The user sees the same lines they were looking at (now shifted down by the streaming line count). The ↑ indicator already reflects the new offset. Need to verify no jitter.

### Open Questions (resolved in Phase 0 research)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Constitution alignment**: This feature directly serves jcode's core purpose — "blazing-fast, TUI-first" and "dramatically faster, more memory-efficient" by ensuring the streaming scroll behavior feels polished and natural. No constitutional conflict.

**Gates**:
- ✅ GATE-001: Feature serves end-user visible UI polish — within constitution scope
- ✅ GATE-002: No new external dependencies or crate additions — purely modifies existing TUI state machine
- ✅ GATE-003: No performance regression risk — scroll math is O(1), rendering is already incremental
- ✅ GATE-004: Fix is contained to existing files — no new modules needed

**Status**: All gates pass. Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/003-streaming-auto-scroll/
├── plan.md              # This file
├── research.md          # Phase 0 — resolved unknowns
├── data-model.md        # Phase 1 — entity definitions
├── quickstart.md        # Phase 1 — implementation guide
├── checklists/
│   └── requirements.md  # Spec quality checklist (done)
└── tasks.md             # Phase 2 — created by /speckit.tasks
```

### Source Code (repository root)

The feature touches existing TUI Rust source files only — no new files:

```text
src/tui/
├── app/
│   ├── input.rs            # submit_input, commit_pending_streaming, clear_streaming_render_state
│   ├── navigation.rs       # scroll_up, scroll_down, follow_chat_top, pause_chat_auto_scroll
│   └── tests/
│       └── scroll_copy_03.rs  # Scroll unit tests
├── ui_prepare.rs           # prepare_messages_inner, prepare_streaming_cached
└── ui_viewport.rs          # draw_messages, scroll indicators, prompt preview
```

**Structure Decision**: No structural changes. All modifications in existing files.

## Complexity Tracking

> No constitution violations requiring justification.
