# Phase 1 — Design: Top-Down TUI Layout Flip

**Branch**: `001-top-down-tui-layout-flip` | **Date**: 2026-05-18 | **Spec**: `specs/001-top-down-tui-layout-flip/spec.md`
**Input**: `spec.md`, `research.md`

---

## Data Model

### `scroll_offset` — Single, Direct, Absolute

```
scroll_offset = 0     → auto-follow mode, showing newest content at top of viewport
scroll_offset = N     → user has scrolled, Line N is the first visible line
scroll_offset = max   → showing oldest content at top of viewport
```

There is **one** `scroll_offset` field (`App.scroll_offset: usize`). It is used directly — no inversion. All navigation and rendering code reads it as-is.

### Section Order (content array)

```
content[0]   → PreparedSectionKind::Streaming         (newest)
content[1]   → PreparedSectionKind::BatchProgress
content[2]   → PreparedSectionKind::Body
content[N]   → PreparedSectionKind::Header              (oldest)
```

### States

| State | `scroll_offset` | `auto_scroll_paused` | Behavior |
|-------|----------------|---------------------|----------|
| Auto-following | `0` | `false` | New content streams at top of viewport. Viewport stays at 0. |
| User scrolled down | `> 0` | `true` | New content still arrives but viewport is frozen. `↑N` indicator visible. |
| User scrolled back to top | `0` | `false` (via `follow_chat_top()`) | Auto-follow resumes. |

### Core Functions

```
scroll_up(amount)   → scroll_offset = min(scroll_offset + amount, max_scroll)
                        auto_scroll_paused = true
                        // Moves toward older content (higher line index)

scroll_down(amount) → scroll_offset = scroll_offset.saturating_sub(amount)
                        if scroll_offset == 0 → follow_chat_top()
                        // Moves toward newer content (lower line index)

follow_chat_top()   → scroll_offset = 0
                        auto_scroll_paused = false
                        // Resume auto-follow at newest content

pause_chat_auto_scroll()
                    → auto_scroll_paused = true
                        // Freeze current position. No inversion math needed.

follow_chat_top_for_typing()
                    → if !typing_scroll_lock → follow_chat_top()
```

---

## Key Design Decisions

### D1. No Inversion Math Anywhere

The previous code had inversion math in `scroll_up`, `scroll_down`, `pause_chat_auto_scroll`, `ui_viewport.rs`, and `handterm_native_scroll.rs`. This was the root cause of all bugs.

**Rule**: `scroll_offset` is always a direct line index. `0` = newest content at the top of the viewport. Higher = older content. No `max - offset` anywhere.

### D2. All Dependent Features Map Correctly

| Feature | Before (broken) | After (fixed) |
|---------|----------------|---------------|
| Auto-follow | `max_scroll` (oldest) | `0` (newest) |
| Viewport default | User's scroll_offset | `0` when not paused |
| Native scrollbar position | `max_scroll` when not paused | `0` when not paused |
| Scroll up | Decreased offset (toward newest) — wrong | Increases offset (toward older) — correct |
| Scroll down | Increased offset (toward oldest) — wrong | Decreases offset (toward newest) — correct |
| Section order | `[Header, Body, Batch, Stream]` | `[Stream, Batch, Body, Header]` |
| Compacted history threshold | Same logic — needs verification | Same threshold, offset meaning unchanged |
| Prompt preview | Triggers when `scroll > 0` | Same — offset > 0 means scrolled away from newest |

### D3. Section Order is the Foundation

All absolute line offsets (copy targets, edit tool ranges, user prompt positions, image regions) are recalculated by `from_sections()` based on section order. Changing the section order automatically fixes all absolute position calculations. No manual offset updates needed in individual features.

---

## Architecture Changes

### File 1: `src/tui/ui_prepare.rs` (~4 lines changed)

**Change**: In `prepare_messages_inner()`, reverse the section order:

```rust
// BEFORE (broken — oldest first)
PreparedChatFrame::from_sections(vec![
    (PreparedSectionKind::Header, header_prepared),
    (PreparedSectionKind::Body, body_prepared),
    (PreparedSectionKind::BatchProgress, batch_progress_prepared),
    (PreparedSectionKind::Streaming, streaming_prepared),
])

// AFTER (correct — newest first)
PreparedChatFrame::from_sections(vec![
    (PreparedSectionKind::Streaming, streaming_prepared),
    (PreparedSectionKind::BatchProgress, batch_progress_prepared),
    (PreparedSectionKind::Body, body_prepared),
    (PreparedSectionKind::Header, header_prepared),
])
```

### File 2: `src/tui/app/navigation.rs` (~20 lines changed)

**Change**: Simplify scroll functions — remove inversion math.

```rust
// scroll_up — move toward older content
pub(super) fn scroll_up(&mut self, amount: usize) {
    let max_scroll = super::super::ui::last_max_scroll();
    let max = if max_scroll > 0 { max_scroll } else { self.scroll_max_estimate() };
    self.scroll_offset = (self.scroll_offset + amount).min(max);
    self.auto_scroll_paused = true;
    self.maybe_queue_compacted_history_load();
}

// scroll_down — move toward newer content
pub(super) fn scroll_down(&mut self, amount: usize) {
    if !self.auto_scroll_paused { return; }
    self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    if self.scroll_offset == 0 {
        self.follow_chat_top();
    }
}

// pause_chat_auto_scroll — just mark paused, no inversion
pub(super) fn pause_chat_auto_scroll(&mut self) {
    if self.auto_scroll_paused { return; }
    self.auto_scroll_paused = true;
}
```

`follow_chat_top()` stays as-is — it already correctly sets `scroll_offset = 0`.

### File 3: `src/tui/ui_viewport.rs` (~3 lines changed)

**Change**: Auto-follow uses `scroll = 0` instead of `max_scroll`.

```rust
// BEFORE
let scroll = if app.auto_scroll_paused() {
    user_scroll.min(max_scroll)
} else {
    max_scroll
};

// AFTER
let scroll = if app.auto_scroll_paused() {
    user_scroll.min(max_scroll)
} else {
    0  // auto-follow = newest content at top
};
```

### File 4: `src/tui/app/handterm_native_scroll.rs` (~3 lines changed)

**Change**: Same fix — auto-follow position is `0` not `max_scroll`.

```rust
// BEFORE
let position = if self.auto_scroll_paused {
    self.scroll_offset.min(max_scroll)
} else {
    max_scroll
};

// AFTER
let position = if self.auto_scroll_paused {
    self.scroll_offset.min(max_scroll)
} else {
    0
};
```

### File 5: `src/tui/app/tests/scroll_copy_03.rs` (~5 lines changed)

**Change**: Fix test expectations. After the fix, scrolling up INCREASES offset.

```rust
// BEFORE
assert!(second_offset < first_offset, "scrolling up should decrease offset");

// AFTER
assert!(second_offset > first_offset, "scrolling up should increase offset (moves toward older)");
```

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Tests break | High — many tests assert scroll_offset behavior | Run full test suite; fix assertions systematically |
| Copy/badge coordinates | Medium — `from_sections` handles this, but verify | Manual visual test after fix |
| Scrolling when sections are empty | Low — empty sections produce zero lines, order doesn't matter | No special handling needed |
| Compacted history threshold | Low — same comparison direction | Verify after fix |
| Native scrollbar jumps | Medium — wrong position could cause scrollbar to jump | Test with scrolling in Ghostty |
| Prompt preview | Low — `scroll > 0` is correct semantic for "scrolled away" | Verify after fix |

---

## Verification Plan

1. **Section order**: Dump content line indices; verify Streaming is first, Header is last
2. **Auto-follow**: Start conversation; confirm new output appears right below input
3. **Scroll up**: Press Ctrl+J; confirm older content appears (offset increases)
4. **Scroll down**: Press Ctrl+K; confirm newer content appears (offset decreases toward 0)
5. **Pause/resume**: Scroll away; confirm auto-follow pauses; scroll back to 0; confirm it resumes
6. **Indicator**: Confirm `↑N` shows when scrolled away from newest
7. **All existing tests**: `cargo test` passes (with assertion fixes)
8. **Copy selection**: Select and copy text after fix; confirm correct ranges
