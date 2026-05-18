# TUI Scroll Logic Gauntlet — Tasks

## Task 1: Fix scroll_to_prev_prompt direction

**File**: `src/tui/app/state_ui_runtime.rs`
**Priority**: P0 (user-facing bug)
**Root cause**: `positions.last()` should be `positions.first()` for top-down convention (positions[0]=newest). Search loop iterates backward seeking lower indices when it should iterate forward seeking higher indices.

**Action**: Restore to committed version from 275ea8ac:
- Unpaused: `positions.first()` → most recent prompt (lowest index)
- Paused: iterate forward `for &pos in &positions { if pos > current { ... } }` → older prompt (higher index)

## Task 2: Fix scroll_to_next_prompt direction

**File**: `src/tui/app/state_ui_runtime.rs`
**Priority**: P0 (user-facing bug)
**Root cause**: iterates forward seeking higher indices (older) when it should iterate backward seeking lower indices (newer).

**Action**: Restore to committed version from 275ea8ac:
- Iterate reversed `positions.iter().rev()` seeking `pos < current`
- Fallback to `follow_chat_top()` when no newer prompt exists

## Task 3: Fix scroll_to_recent_prompt_rank indexing

**File**: `src/tui/app/state_ui_runtime.rs`
**Priority**: P1 (user-facing bug for Ctrl+1/2/3 shortcuts)
**Root cause**: `positions.len().saturating_sub(rank)` gives rank-from-end instead of rank-from-start.

**Action**: Restore to `(rank - 1).min(positions.len() - 1)` for 0-based newest-first indexing.

## Task 4: Fix test regression — prompt_preview line number

**File**: `src/tui/app/tests/scroll_copy_03.rs`
**Priority**: P2 (test regression)
**Root cause**: Line 155 assertion `"Intro line 20"` was part of ba88521b's fix but was reverted by smaller agent.

**Action**: Restore assertion to `"Intro line 19"` matching the wrapped-line rendering at 40-char test width.

## Task 5: Improve test coverage for prompt navigation

**File**: `src/tui/app/tests/scroll_copy_03.rs`
**Priority**: P2 (future-proofing)
**Gap**: No test exercises scroll_to_prev_prompt, scroll_to_next_prompt, or scroll_to_recent_prompt_rank.

**Action**: Add tests:
- `test_scroll_prev_next_prompt_roundtrip` — Ctrl+[ then Ctrl+] returns to original offset
- `test_scroll_prev_from_auto_follow` — settings positions[0] (newest) works
- `test_scroll_recent_prompt_rank_1` — rank 1 goes to positions[0]
