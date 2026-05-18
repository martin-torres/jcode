# Data Model: Streaming Content Auto-Scroll

**Phase**: 1 | **Date**: 2026-05-18

## Key State Fields on `App`

These fields already exist in the App struct in `src/tui/app.rs`. This document describes the invariants that must hold for the push-down behavior.

### `scroll_offset: usize`

- Absolute line index from the start of the combined PreparedChatFrame [Streaming, BatchProgress, Body, Header]
- `0` = newest content (Streaming section), any value above is older
- Must always be ≤ `max_scroll` at render time (capped in `draw_messages`)
- Invariant: When `auto_scroll_paused=true` and streaming grows, `scroll_offset` is NOT adjusted — the viewport naturally shows older content as streaming pushes down

### `auto_scroll_paused: bool`

- `false`: Viewport pegged to newest content. `scroll_offset` must be 0.
- `true`: User has manually scrolled. `scroll_offset` is user's chosen position.
- Transitions:
  - `scroll_up()` / `scroll_down()` → sets `auto_scroll_paused = true`
  - `follow_chat_top()` → sets `auto_scroll_paused = false`, `scroll_offset = 0`
  - `pause_chat_auto_scroll()` → sets `auto_scroll_paused = true` (offset unchanged)

### `streaming_text: String`

- The in-progress assistant response text. Grows via `append_streaming_text()`.
- When empty or `!is_processing()`: the Streaming section is empty (no prepared lines).
- When non-empty: processed by `prepare_streaming_cached()` → added as Streaming section.

### `is_processing: bool`

- True while an assistant response is being generated.
- `is_processing() && !streaming_text.is_empty()` → streaming section is non-empty.
- `is_processing()` can be true even when `streaming_text` is empty (waiting for first token).

## Data Flow During Key Events

### User scrolls down while streaming

```
Event: scroll_down()
→ auto_scroll_paused = true
→ scroll_offset -= amount (clamped at 0)
→ follow_chat_top() if offset hits 0 (resumes)
→ render:
    PreparedChatFrame = [Streaming+N, BatchProgress, Body, Header]
    viewport_start = scroll_offset (user's old position, now shows older content)
    ↑ indicator = N newer lines above (streaming section grown)
```

### Streaming finishes (commit)

```
Event: commit_pending_streaming_assistant_message()
→ streaming_text moved to display_messages
→ streaming_text = ""
→ render:
    PreparedChatFrame = [Streaming=0, BatchProgress, Body+N, Header]
    viewport_start = scroll_offset (same index, now into body section)
    If auto_following: user sees newly committed message at top
```

### Submit new prompt

```
Event: submit_input()
→ follow_chat_top() [offset=0, paused=false]
→ commit_pending_streaming_assistant_message()
→ send message + start new response [is_processing=true]
→ render:
    PreparedChatFrame = [Streaming=empty, BatchProgress, Body, Header]
    scroll_offset = 0 → newest content at top
    → new response starts streaming:
    PreparedChatFrame = [Streaming+growing, BatchProgress, Body, Header]
    scroll_offset = 0 → streaming visible at top
    completed body content pushed down
```

## Validation Invariants

1. After `follow_chat_top()`: `scroll_offset == 0 && auto_scroll_paused == false`
2. After `scroll_up(amount)`: `scroll_offset <= last_max_scroll()` (capped)
3. After `scroll_down(amount)`: `scroll_offset` never underflows (saturating_sub)
4. After `commit_pending_streaming_assistant_message()`: `streaming_text.is_empty()`
5. During streaming at `auto_scroll_paused=true`: `scroll_offset` unchanged by streaming growth
6. During streaming at `auto_scroll_paused=false`: `scroll_offset` remains 0
