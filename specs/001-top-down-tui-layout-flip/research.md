# Phase 0 — Research: Top-Down TUI Layout Flip

**Branch**: `001-top-down-tui-layout-flip` | **Date**: 2026-05-18 | **Spec**: `specs/001-top-down-tui-layout-flip/spec.md`

## Problem Statement

The spec defines a **top-down document-style** layout where:
- Input box is at the **top**
- Newest content appears **right below the input**
- Older content gets **pushed downward**
- Auto-scroll anchors at `scroll = 0` (top = newest)

The initial implementation (commit `275ea8ac`) attempted this flip but introduced bugs in scroll semantics and section ordering that make the layout behave incorrectly.

---

## Code Analysis — All Files Involved

### 1. Layout — `src/tui/ui.rs` (lines 1904–1928)

**Status: ✅ Correct**

The layout constraints put input at `chunks[0]` (top) and messages at `chunks[6]` (below all status/inline UI). This is correct.

```rust
chunks: [
    0: Input (top)          ✅
    1: Status line
    2: Notification
    3: Queued messages
    4: Inline UI
    5: Gap
    6: Messages area        ✅ (below input)
    7: Donut animation
]
```

### 2. Section Order — `src/tui/ui_prepare.rs` (lines 458–463)

**Status: ❌ WRONG — still old (pre-inversion) order**

The current order is the **original bottom-up order**:

```rust
PreparedChatFrame::from_sections(vec![
    (PreparedSectionKind::Header, header_prepared),          // 0 — oldest
    (PreparedSectionKind::Body, body_prepared),              // 1
    (PreparedSectionKind::BatchProgress, batch_progress_prepared), // 2
    (PreparedSectionKind::Streaming, streaming_prepared),    // 3 — newest
])
```

**Spec says it should be** `[Streaming, BatchProgress, Body, Header]` (newest-first). This means Body messages still iterate oldest-first, and the newest streaming content ends up at the **end** (bottom) of the content array.

This is the **root cause** of "text comes from the bottom feeding up into the box" — the content array puts the newest stuff at the end, so even though the layout has input at the top, the actual text lines for new content are at the bottom.

### 3. Scroll Viewport — `src/tui/ui_viewport.rs` (lines 246–250)

**Status: ❌ WRONG — auto-follow shows OLDEST content**

```rust
let scroll = if app.auto_scroll_paused() {
    user_scroll.min(max_scroll)
} else {
    max_scroll   // ← BUG: auto-follow shows max_scroll = oldest content
};
```

When not paused (auto-scroll active), it uses `max_scroll` which points to the **last lines** (oldest content). In the inverted layout, auto-follow should show `scroll = 0` (newest content at top).

### 4. Handterm Native Scroll — `src/tui/app/handterm_native_scroll.rs` (lines 121–126)

**Status: ❌ WRONG — same issue**

```rust
let position = if self.auto_scroll_paused {
    self.scroll_offset.min(max_scroll)
} else {
    max_scroll   // ← BUG: should be 0 for auto-follow
};
```

Same bug — auto-follow reports position at `max_scroll` (oldest) instead of `0` (newest).

### 5. `scroll_up` — `src/tui/app/navigation.rs` (lines 1053–1068)

**Status: ❌ WRONG — confused inversion math**

```rust
pub(super) fn scroll_up(&mut self, amount: usize) {
    let max_scroll = super::super::ui::last_max_scroll();
    // ...
    if !self.auto_scroll_paused {
        let current_abs = max.saturating_sub(self.scroll_offset);  // invert
        self.scroll_offset = current_abs.saturating_sub(amount);   // subtract from inverted
    } else {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }
    self.auto_scroll_paused = true;
}
```

The `!auto_scroll_paused` branch does a double-inversion (`max - offset` then `result - amount`). This was a hack to handle the coordinate system flip. The spec says:
- `scroll_up` should **increase** offset (move toward older content = larger line indices)
- `scroll_down` should **decrease** offset (move toward newer content = smaller line indices)

### 6. `scroll_down` — `src/tui/app/navigation.rs` (lines 1086–1100)

**Status: ❌ WRONG — adds instead of subtracts in inverted system**

```rust
pub(super) fn scroll_down(&mut self, amount: usize) {
    // ...
    self.scroll_offset = (self.scroll_offset + amount).min(max);
    if self.scroll_offset >= max {
        self.follow_chat_top();  // resumes at scroll = 0
    }
}
```

In the inverted layout, scrolling down should **reduce** `scroll_offset` (move toward 0 = newest). Currently it **adds**, which moves toward `max` (oldest). The `follow_chat_top()` triggers when it reaches `max`, but this is inverted logic that only works because of the section order bug.

### 7. `pause_chat_auto_scroll` — `src/tui/app/navigation.rs` (lines 1070–1084)

**Status: ❌ WRONG — inverts the offset**

```rust
pub(super) fn pause_chat_auto_scroll(&mut self) {
    // ...
    self.scroll_offset = max.saturating_sub(self.scroll_offset.min(max));
    self.auto_scroll_paused = true;
}
```

This inverts the offset (`max - offset`) when pausing. This was needed because of the coordinate system confusion. In a correctly inverted system, `scroll_offset` should be a direct index with `0 = newest`, no inversion needed.

### 8. `follow_chat_top` — `src/tui/app/navigation.rs` (lines 1102–1105)

**Status: ✅ Correct**

```rust
pub(super) fn follow_chat_top(&mut self) {
    self.scroll_offset = 0;
    self.auto_scroll_paused = false;
}
```

`scroll_offset = 0` correctly represents "newest content at top of viewport."

### 9. `debug_scroll_bottom` / `debug_scroll_top` — `src/tui/app/navigation.rs` (lines 1115–1122)

**Status: ❌ Naming is confusing**

```rust
pub(super) fn debug_scroll_top(&mut self) {
    self.scroll_offset = 0;        // = newest
    self.auto_scroll_paused = true;
}
pub(super) fn debug_scroll_bottom(&mut self) {
    self.follow_chat_top();        // = newest, unpaused
}
```

"Top" = newest (scroll_offset=0), "Bottom" = same. These names are semantically ambiguous.

### 10. Test expectations — `src/tui/app/tests/scroll_copy_03.rs`

**Status: ❌ WRONG — tests assume old semantics**

The test `test_scroll_ctrl_k_j_offset` says:
```rust
// Scroll up (switches to absolute-from-top mode)
app.handle_key(up_code.clone(), up_mods).unwrap();
assert!(app.auto_scroll_paused);
let first_offset = app.scroll_offset;

app.handle_key(up_code.clone(), up_mods).unwrap();
let second_offset = app.scroll_offset;
assert!(second_offset < first_offset, "scrolling up should decrease offset");
```

With the current buggy code, "scrolling up" **decreases** offset toward 0 (= newest). This means the tests match the current (buggy) behavior. After fixing, scrolling up should **increase** offset toward older content.

### 11. `state_ui_messages.rs` — compacted history load (line 380)

**Status: ⚠️ Needs review**

```rust
if self.scroll_offset > COMPACTED_HISTORY_LOAD_SCROLL_THRESHOLD {
```

Checks if user has scrolled far enough from newest to load compacted history. If offset is now higher = older, this threshold logic may need adjustment.

---

## Summary of Bugs

| # | File | Line(s) | Bug | Fix |
|---|------|---------|-----|-----|
| B1 | `ui_prepare.rs` | 458-463 | Section order is `[Header, Body, Batch, Stream]` — oldest first | Reverse to `[Stream, Batch, Body, Header]` |
| B2 | `ui_viewport.rs` | 246-250 | Auto-follow shows `max_scroll` (oldest content) | Change to `scroll = 0` when not paused |
| B3 | `handterm_native_scroll.rs` | 122-126 | Native scrollbar shows `max_scroll` for auto-follow | Change to `0` when not paused |
| B4 | `navigation.rs` | 1053-1068 | `scroll_up` has inverted `max - offset` math | Direct: scroll_up = increase offset |
| B5 | `navigation.rs` | 1086-1100 | `scroll_down` adds instead of subtracting | Direct: scroll_down = decrease offset |
| B6 | `navigation.rs` | 1070-1084 | `pause_chat_auto_scroll` inverts offset | Remove inversion; offset is already absolute |
| B7 | `scroll_copy_03.rs` | 22-25 | Test expects `scroll_up` to **decrease** offset | Flip assertions |

## Root Cause

The original commit `275ea8ac` **partially** applied the inversion. It reversed the layout (input at top) and reversed the sections order in the initial implementation, but later `navigation.rs` and `ui_viewport.rs` got out of sync. The section order was kept at the old `[Header, Body, BatchProgress, Streaming]` while the scroll functions tried to compensate with inversion hacks, creating a dual-inversion mess.
