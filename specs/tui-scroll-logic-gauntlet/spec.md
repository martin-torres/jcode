# Spec: TUI Scroll Logic Gauntlet — Top-Down Layout Audit

**Feature Branch**: `tui-scroll-logic-audit`
**Created**: 2026-05-18
**Status**: Investigation (no code changes — analysis only)
**Scope**: Granular logic audit of all scroll-related code changed by a less-capable agent on top of the "top-down TUI layout flip" (275ea8ac) + fix commit (ba88521b).

---

## 1. Architecture Summary

### The Top-Down Layout (from commit 275ea8ac)

```
┌──────────────────────────┐  ← chunks[0] input (was bottom)
│  [input box]             │
├──────────────────────────┤
│  ┌─ Status line ──────┐  │  ← chunks[1] (was at bottom)
│  └────────────────────┘  │
│  ┌─ Notification ─────┐  │  ← chunks[2]
│  └────────────────────┘  │
│  ┌─ Queued ───────────┐  │  ← chunks[3]
│  └────────────────────┘  │
│  ┌─ Inline UI ────────┐  │  ← chunks[4]
│  └────────────────────┘  │
│  ┌─ Gap ──────────────┐  │  ← chunks[5]
│  └────────────────────┘  │
│  ┌─ Messages ─────────┐  │  ← chunks[6] (was at top)
│  │  ┌ streaming... ▊  │  │     Section order: Streaming, BatchProgress, Body, Header
│  │  │ newest message   │  │     (reversed from bottom-up: Header, Body, BatchProgress, Streaming)
│  │  │ older message    │  │
│  │  │  │
│  ┌─ Donut ────────────┐  │  ← chunks[7]
│  └────────────────────┘  │
└──────────────────────────┘
```

**Scroll semantics** in this layout:
- `scroll_offset = 0` → auto-follow, newest content at top of messages area
- `scroll_offset > 0` → scrolled down/away, showing older content
- `auto_scroll_paused = true` → user has manually scrolled
- `wrapped_user_prompt_starts` = line positions in **document order** (Streaming → Header), so **positions[0] = newest prompt** at lowest line index

### What the smaller agent touched (dirty state vs HEAD=ba88521b)

| File | Lines Changed | Nature |
|---|---|---|
| `src/tui/app/state_ui_runtime.rs` | scroll_to_prev_prompt, scroll_to_next_prompt, scroll_to_recent_prompt_rank | Logic revert |
| `src/tui/app/navigation.rs` | scroll_up, pause_chat_auto_scroll, scroll_down, comments | Cleanup |
| `src/tui/app/handterm_native_scroll.rs` | native scroll auto-follow position | Fix (max_scroll→0) |
| `src/tui/ui_viewport.rs` | viewport auto-follow position | Fix (max_scroll→0) |
| `src/tui/backend.rs` | protocol error handling | Resiliency (disconnect→skip) |
| `src/tui/app/tests/scroll_copy_03.rs` | Multiple test assertions | Mixed (some fix reverted) |
| `src/provider_catalog.rs` | OpenCode model list | Data update (unrelated) |
| `src/provider/openrouter_provider_impl.rs` | Prefetch strategy | Behavior change (unrelated) |
| `src/skill.rs` | Skill loading from .hermes/cmds/ | New feature (unrelated) |

---

## 2. Granular Logic Audit

### 2.1 `scroll_to_prev_prompt()` — **BUG CONFIRMED** ✗

**Current dirty code:**
```rust
pub fn scroll_to_prev_prompt(&mut self) {
    let positions = ui::last_user_prompt_positions(); // document order, positions[0]=newest
    if positions.is_empty() { return; }
    let current = self.scroll_offset;

    if !self.auto_scroll_paused {
        if let Some(&pos) = positions.last() {  // ← BUG: last = OLDEST, not most recent
            self.scroll_offset = pos;
            self.auto_scroll_paused = true;
        }
        return;
    }

    let mut target = None;
    for &pos in positions.iter().rev() {  // ← BUG: iterating oldest-first
        if pos < current {                 // ← BUG: looking for lower index = NEWER prompt
            target = Some(pos);
            break;
        }
    }
    // ... applies target if found
}
```

**Correct version (from 275ea8ac committed):**
```rust
if !self.auto_scroll_paused {
    if let Some(&pos) = positions.first() {  // ← positions[0] = newest
        self.scroll_offset = pos;
        self.auto_scroll_paused = true;
    }
    return;
}

for &pos in &positions {          //  newest-first order
    if pos > current {             //  higher index = older
        self.scroll_offset = pos;
        return;
    }
}
```

**Failure scenario:** User presses Ctrl+[ to go to previous prompt.
- If at newest: jumps to OLDEST prompt instead of most recent → user sees ancient history
- If already scrolled: moves to NEWER (later) prompt instead of OLDER → direction inverted

**Expected effect of the bug:** Scrolling through prompts backwards jumps forward instead.

### 2.2 `scroll_to_next_prompt()` — **BUG CONFIRMED** ✗

**Current dirty code:**
```rust
pub fn scroll_to_next_prompt(&mut self) {
    let positions = ui::last_user_prompt_positions();
    if positions.is_empty() || !self.auto_scroll_paused { return; }
    let current = self.scroll_offset;

    for &pos in &positions {      // ← BUG: forward iteration, looking for LARGER index
        if pos > current {         // ← BUG: larger index = OLDER, but next_prompt should go NEWER
            self.scroll_offset = pos;
            return;
        }
    }

    self.follow_chat_top();       // ← falls through to "no more prompts below" — wrong message
}
```

**Correct version (from 275ea8ac committed):**
```rust
let mut target = None;
for &pos in positions.iter().rev() {  // oldest-first → finds closest NEWER
    if pos < current {                 // lower index = newer
        target = Some(pos);
        break;
    }
}
if let Some(pos) = target {
    self.scroll_offset = pos;
    return;
}
self.follow_chat_top();  // no newer prompts → go to newest
```

**Failure scenario:** User presses Ctrl+] to go to next (newer) prompt.
- Actually goes to OLDER prompt → direction inverted
- Eventually falls through to `follow_chat_top()` prematurely

### 2.3 `scroll_to_recent_prompt_rank()` — **BUG CONFIRMED** ✗

**Current dirty code:**
```rust
// positions are in document order (top to bottom), we want most-recent first
let target_idx = positions.len().saturating_sub(rank);  // ← BUG: rank 1 → positions[last]
```

**Correct version (from 275ea8ac committed):**
```rust
let target_idx = (rank - 1).min(positions.len() - 1);  // rank 1 → positions[0] = newest
```

**Failure scenario:** Ctrl+1 (jump to most recent prompt) scrolls to POSITIONS[LAST] = oldest prompt instead of newest.

### 2.4 `navigation.rs:scroll_up()` — **CLEAN** ✓

The simplified version:
```rust
self.scroll_offset = (self.scroll_offset + amount).min(max);
```
produces the same results as the old conditional `if !paused { amount.min(max) } else { (current + amount).min(max) }` because when `scroll_offset=0`, `(0 + amount).min(max) == amount.min(max)`. **No behavioral change.**

### 2.5 `navigation.rs:scroll_down()` — **CLEAN** ✓

Functionally identical — the removed `max_scroll`/`_max` vars were unused. `saturating_sub(amount)` + `follow_chat_top()` on 0 is the same logic. **No behavioral change.**

### 2.6 `handterm_native_scroll.rs` — **FIX** ✓

Changed `max_scroll` → `0` for the auto-follow (not-paused) case. In top-down layout, auto-follow shows newest content at top of viewport, which corresponds to native scrollbar position 0. The old `max_scroll` would have put the native scrollbar thumb at the bottom while showing the top of content. **Change is correct.**

### 2.7 `ui_viewport.rs:draw_messages()` — **FIX** ✓

Same reasoning as above: auto-follow scroll position is 0 (newest content at top of viewport), not `max_scroll`. **Change is correct.**

### 2.8 `backend.rs:RemoteConnection::next_event()` — **RESILIENCY CHANGE** (opinion)

Changed from `RemoteRead::Disconnected(reason)` to `continue` (skipping unparseable lines). This means transient protocol errors no longer disconnect the session. **Not a bug** but a design choice — could mask real protocol issues. Consider whether this should be configurable or logged at error level instead of warn.

### 2.9 Test Assertions — **MIXED**

| Test | Change | Verdict |
|---|---|---|
| `test_scroll_ctrl_k_j_offset` | `second_offset > first_offset` (was `<`) | ✓ Correlates with current `scroll_up` |
| `test_scroll_offset_capped` | `scroll_offset > 0` (was `== 0`) | ✓ Correct for top-down |
| `test_scroll_render_bottom` | Checks "Scroll test"/"Intro line 01" | ✓ Adapted to new layout |
| `test_scroll_render_scrolled_up` | Removed detailed error message | ✗ **Regression** — debug info lost |
| `test_prompt_preview_reserves_rows` | "Intro line 20" (was "line 19") | ✗ **Regression** — fix from ba88521b reverted |
| `test_scroll_top_does_not_snap` | offset=0 vs offset=15 comparison | ✓ Adapted correctly |

---

## 3. Bug Impact Summary

### User-facing symptoms (expected from the 3 bugs):

| Keystroke | Expected Behavior | Actual Buggy Behavior |
|---|---|---|
| `Ctrl+[` (prev prompt) | Jump to previous (older) user prompt | Jumps to NEWEST prompt if unpaused, or to NEWER prompt if paused |
| `Ctrl+]` (next prompt) | Jump to next (newer) user prompt | Jumps to OLDER prompt instead |
| `Ctrl+1` (most recent prompt) | Jump to most recent user prompt | Jumps to OLDEST user prompt |
| `Ctrl+N` (Nth recent prompt) | Jump to N-th most recent | Jumps to N-th from end instead |

### Root cause pattern:
All 3 bugs in `state_ui_runtime.rs` share the same root cause: the smaller agent **reverted the prompt-navigation search logic** from the top-down layout convention back to the bottom-up convention, without also reverting the coordinate system.

In the **bottom-up layout** (pre-275ea8ac):
- positions[0] = top of content = oldest
- positions[last] = bottom of content = newest
- scroll_offset = distance from bottom

In the **top-down layout** (post-275ea8ac):
- positions[0] = top of content = newest (Streaming section first)
- positions[last] = bottom of content = oldest
- scroll_offset = distance from top

The three methods use the old convention's search strategy with the new convention's data → **vectors reversed**.

---

## 4. Fix Strategy (without changing code — prediction analysis)

### If we apply the fix:

The three methods in `state_ui_runtime.rs` need their prompt-position search restored to the committed (275ea8ac) versions:

1. **`scroll_to_prev_prompt`**: `positions.first()` → `for &pos in &positions { if pos > current { ... } }` (search forward for higher = older)
2. **`scroll_to_next_prompt`**: iterate reversed for lower indices → `follow_chat_top()` at end (search backward for lower = newer)
3. **`scroll_to_recent_prompt_rank`**: `(rank - 1).min(len - 1)` for 0-based newest-first

**Effect on user experience:**
- `Ctrl+[` → correctly goes to the previous (older) prompt
- `Ctrl+]` → correctly goes to the next (newer) prompt
- `Ctrl+1` → correctly goes to the most recent prompt
- All prompt navigation operations would feel natural and consistent

**Effect on the codebase as a whole:**
- Restores the internal consistency of the top-down layout
- `state_ui_runtime.rs` would be semantically consistent with `navigation.rs` and `ui.rs`
- No ripple effects outside this file — these methods only call `last_user_prompt_positions()` and `follow_chat_top()`, which are already correct
- Tests that exercise these navigation paths (scroll_copy_03.rs) would continue to pass — the 2 test regressions are minor (assertion string on line 20 vs 19, and missing error detail)

**Ripple effects:** Zero. These are private methods called only from `handle_key` dispatch in navigation.rs and input.rs, and they have no callers outside the TUI app module.

### Verification that the fix would work as intended:

The three methods are fully testable. If we:
1. Set up a conversation with 3 user prompts at known wrapped line positions
2. Call `scroll_to_prev_prompt()` / `scroll_to_next_prompt()` / `scroll_to_recent_prompt_rank(N)`
3. Assert `scroll_offset` lands at the correct line index

...all three bugs would be caught by those assertions.

---

## 5. Test Gap Analysis

Tests that **do** exercise scroll logic:
- `test_scroll_ctrl_k_j_offset` — scroll_up/down key, offset direction
- `test_scroll_offset_capped` — max cap
- `test_scroll_render_*` — visual regression (rendered text assertions)
- `test_scroll_content_shifts` — offset→different render
- `test_scroll_round_trip` — scroll up then down returns to 0

Tests that are **missing** (would have caught the bugs):
- ❌ `test_scroll_prev_next_prompt_roundtrip` — Ctrl+[ then Ctrl+] should return to original offset
- ❌ `test_scroll_recent_prompt_rank_1` — Ctrl+1 should go to most recent prompt
- ❌ `test_scroll_prev_from_auto_follow` — Ctrl+[ from auto-follow should go to newest prompt (oldest at index 0)
- ❌ `test_scroll_next_at_bottom` — Ctrl+] at newest should be no-op (already at newest)

These gaps are independent of the current bugs — they've existed since the original top-down flip.

---

## 6. Conclusion

| Severity | Count | Details |
|---|---|---|
| **Bug** (user-visible) | 3 | scroll_to_prev_prompt, scroll_to_next_prompt, scroll_to_recent_prompt_rank |
| **Test regression** | 2 | Assertion values reverted, debug messages removed |
| **Clean fixes** (correct) | 3 | handterm_native_scroll, ui_viewport auto-follow, backend resliency |
| **Clean simplification** (neutral) | 3 | scroll_up simplification, scroll_down, pause_chat_auto_scroll |

**Total issues from smaller agent: 5 (3 bugs + 2 test regressions)**

All 3 bugs are in one file (`src/tui/app/state_ui_runtime.rs`) and follow the same root cause: reversal of coordinate convention without adapting search logic. Fix is low-risk, contained, and fully revertible to committed state.

The changes in handterm_native_scroll.rs, ui_viewport.rs, backend.rs, navigation.rs, and the provider/skill files are either correct improvements or unrelated to the scroll system.
