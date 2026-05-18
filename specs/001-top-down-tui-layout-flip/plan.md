# Implementation Plan: Top-Down TUI Layout Flip

**Branch**: `001-top-down-tui-layout-flip` | **Date**: 2026-05-18 | **Spec**: `specs/001-top-down-tui-layout-flip/spec.md`
**Input**: `spec.md`, `research.md`, `data-model.md`

## Summary

Fix the inverted TUI layout so content flows **top-to-bottom**: input at top, newest response right below it, older content pushed downward. The implementation has bugs in section ordering, scroll math, and auto-follow anchors. Fix requires changes to 5 files.

## Technical Context

**Language/Version**: Rust (2024 edition)  
**Primary Dependencies**: ratatui 0.29+  
**Testing**: `cargo test` (TUI integration tests)  
**Target Platform**: Terminal (macOS, Linux)  
**Project Type**: TUI CLI application  
**Performance Goals**: Frame rendering < 16ms  
**Constraints**: Must not break copy selection, badge rendering, or prompt navigation  

## Project Structure

### Documentation (this feature)

```
specs/001-top-down-tui-layout-flip/
├── plan.md              # This file
├── research.md          # Phase 0 — code analysis & bug catalog
├── data-model.md        # Phase 1 — design & architecture
├── quickstart.md        # Phase 1 — verification steps
├── contracts/
└── tasks.md
```

### Source Code

```
jcode-src/src/tui/
├── ui_prepare.rs              # Section order reversal
├── app/
│   └── navigation.rs          # Scroll math simplification
├── ui_viewport.rs             # Auto-follow anchor fix
├── handterm_native_scroll.rs  # Native scrollbar position fix
└── app/tests/
    └── scroll_copy_03.rs      # Test assertion flip
```

## Files to Change

| File | Change Type | Lines Affected | Risk |
|------|-----------|---------------|------|
| `ui_prepare.rs` (L458-463) | Section order reversal | 4 lines | Medium — affects all absolute offsets |
| `navigation.rs` (L1053-1122) | Remove inversion math | ~20 lines | High — affects all scroll semantics |
| `ui_viewport.rs` (L246-250) | Auto-follow anchor | 3 lines | Medium — changes viewport behavior |
| `handterm_native_scroll.rs` (L122-126) | Native scrollbar anchor | 3 lines | Low — cosmetic |
| `scroll_copy_03.rs` | Test assertion direction | ~5 lines | Low — tests must match new behavior |

## Complexity Tracking

No violations. This is a surgical bug fix to existing code — no new abstractions or dependencies.
