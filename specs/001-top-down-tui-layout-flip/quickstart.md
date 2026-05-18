# Quickstart — Fix Top-Down Layout Bugs

## Prerequisites

```bash
cd /Users/lomalinda007yahoo.com/jcode-src
cargo build 2>&1 | tail -3
```

## Files to Change (in order)

### 1. `ui_prepare.rs` — Reverse section order

`src/tui/ui_prepare.rs` lines 458-463:

- Swap `(PreparedSectionKind::Header, header_prepared)` to last position
- Move `(PreparedSectionKind::Streaming, streaming_prepared)` to first position
- Oldest → Newest becomes Newest → Oldest

### 2. `navigation.rs` — Remove inversion math

`src/tui/app/navigation.rs` lines 1053-1122:

- `scroll_up`: remove `max - offset` inversion, add to offset directly
- `scroll_down`: remove `+ amount`, use `saturating_sub` instead
- `pause_chat_auto_scroll`: remove inversion, just set `auto_scroll_paused = true`
- `follow_chat_top`: keep as-is (already correct)

### 3. `ui_viewport.rs` — Fix auto-follow anchor

`src/tui/ui_viewport.rs` line 249:
- Change `max_scroll` → `0`

### 4. `handterm_native_scroll.rs` — Fix native scrollbar

`src/tui/app/handterm_native_scroll.rs` line 125:
- Change `max_scroll` → `0`

### 5. `scroll_copy_03.rs` — Fix assertions

`src/tui/app/tests/scroll_copy_03.rs`:
- `second_offset < first_offset` → `second_offset > first_offset`
- Update related comments

## Verify

```bash
cargo test -p jcode 2>&1 | tail -30   # All tests pass
# Visual verification:
jcode                                 # Launch TUI
# - Input at top?
# - New output appears right below input?
# - Ctrl+J scrolls to older content?
# - Ctrl+K scrolls back to newest?
```
