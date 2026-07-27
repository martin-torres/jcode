# Jcode TUI Layout Specification

> Approved 2026-07-27. This is the authoritative reference for all TUI layout changes.

## The Screen Layout

The screen is divided into fixed pinned elements and a scrollable viewport. From top to bottom:

```
┌─────────────────────────────────────────────────────────────────┐
│ chunks[3] — QUEUED PROMPTS        (pinned, 0-3 rows)           │
│ chunks[0] — INPUT AREA            (pinned, wraps up to 10 rows)│
│ chunks[1] — STATUS LINE           (pinned, 1 row)              │
│ chunks[2] — NOTIFICATION          (pinned, 0 or 1 row)         │
│ chunks[4] — INLINE UI             (pinned, 0 height when off)  │
│ chunks[5] — GAP                   (0 or 1 row)                 │
│ chunks[6] — VIEWPORT              (scrollable, main content)   │
│ chunks[7] — DONUT                 (conditional, 0 or 14 rows)  │
└─────────────────────────────────────────────────────────────────┘
```

## Component Definitions

### chunks[3] — Queued Prompts (top, pinned)
Prompts you've queued with Ctrl+Enter while the agent is busy.
- Shows 0-3 rows
- Next-to-run prompt is at the bottom (closest to input)
- Last-queued prompt is at the top
- Numbered as `1]`, `2]`, etc.

### chunks[0] — Input Area (pinned)
The text box where you type your prompt.
- Always visible, never scrolls
- Prompt character changes by mode: `> ` chat, `$ ` shell, `… ` processing, `» ` skill active
- Wraps up to 10 lines, scrolls internally if more
- Hint line below shows shell mode or command suggestions

### chunks[1] — Status Line (pinned, 1 row)
Single row of live session status.
- Shows: status dot + label, token counts, tokens/second, KV cache hit %, cumulative cost
- States: Idle, Sending, Connecting, Thinking, Streaming, RunningTool, WaitingForNetwork

### chunks[2] — Notification (pinned, 0 or 1 row)
Temporary warning or info banner.
- Context limit warnings, compaction notices, provider errors
- 0 height when nothing to show

### chunks[4] — Inline UI (pinned, conditional)
Interactive selection menus (model picker, account picker).
- 0 height when inactive
- 1 or more rows when a picker is open

### chunks[5] — Gap
Visual spacing between inline UI and viewport.
- 1 row when inline UI is active, 0 otherwise

### chunks[6] — Messages Viewport (scrollable)
The main content area. Contains sections in this order (top to bottom):

1. **Streaming Section** — Live LLM text as it generates
2. **BatchProgress** — Tool execution status bars
3. **Body Section** — All committed messages (user prompts, assistant replies, tool results)
4. **Header** — Session info (model, provider, context usage)

### chunks[7] — Donut (conditional, bottom)
ASCII art spinner shown when idle.
- 14 rows when visible, 0 when not
- Dismisses when you start typing or a new response begins

## Streaming Text Behavior

This is the most critical behavior to get right.

**The rule (never changes):**
1. New tokens appear at the **top** of the streaming section
2. All existing text shifts **down**, unchanged
3. The viewport is a stack — new things go on top
4. Read **bottom-to-top** for the story in chronological order

**Visual example over time:**

```
T=0s:
│ ▐ There was once a boy named Jack.


T=1s:
│ ▐ He lived on a small farm with his mother.   ← NEW (top)
│ ▐ There was once a boy named Jack.              ← shifted down, unchanged


T=2s:
│ ▐ They were very poor and barely had enough     ← NEW (top)
│ ▐ to eat.
│ ▐ He lived on a small farm with his mother.     ← shifted down, unchanged
│ ▐ There was once a boy named Jack.                ← shifted down, unchanged


T=5s:
│ ▐ His mother was angry and threw the beans       ← NEW (top)
│ ▐ out the window.
│ ▐ They were very poor and barely had enough     ← shifted down, unchanged
│ ▐ to eat.
│ ▐ He lived on a small farm with his mother.     ← shifted down, unchanged
│ ▐ There was once a boy named Jack.                ← shifted down, unchanged


T=8s:
│ ▐ He and his mother lived happily ever after     ← NEW (top)
│ ▐ with the gold. The end.
│ ▐ His mother was angry and threw the beans       ← shifted down, unchanged
│ ▐ out the window.
│ ▐ They were very poor and barely had enough     ← shifted down, unchanged
│ ▐ to eat.
│ ▐ He lived on a small farm with his mother.     ← shifted down, unchanged
│ ▐ There was once a boy named Jack.                ← shifted down, unchanged
```

**Reading T=8 bottom-to-top, it's one story:**
> There was once a boy named Jack. He lived on a small farm with his mother. They were very poor and barely had enough to eat. His mother was angry and threw the beans out the window. He and his mother lived happily ever after with the gold. The end.

**Key principles:**
- Text is NEVER rewritten or rearranged
- Text is NEVER repeated
- Existing lines shift down unchanged when new text arrives
- The first sentence is always at the bottom
- The latest token is always at the top
- New tokens append at the bottom of the current streaming paragraph, but the entire paragraph stays at the top of the viewport (above older body content)

## Scroll Behavior

- `scroll = 0` = auto-follow (newest content visible at top of viewport)
- User scrolls up → reveals older content (moves toward Body section at bottom)
- User scrolls down → reveals newer content (moves toward Streaming section at top)
- `auto_scroll_paused = true` when user scrolls away from scroll = 0
- Prompt preview: read-only copy of old prompts shown when user has scrolled away

## Header Position

The Header (session info: model, provider, context usage) is a **fixed element outside the viewport**, placed between chunks[3] (queued prompts) and chunks[0] (input). It does NOT scroll with viewport content.

Updated layout with Header:
```
chunks[3] — Queued Prompts
Header     — Session info (model, provider, context)
chunks[0] — Input Area
chunks[1] — Status Line
chunks[2] — Notification
chunks[4] — Inline UI
chunks[5] — Gap
chunks[6] — Viewport (streaming + body, scrollable)
chunks[7] — Donut
```

## Contradictions Found in Existing Code

1. **Constitution says "scroll up = older"** — Code does "scroll up = newer". Code is correct per this spec.
2. **Header is inside viewport** — Should be moved to fixed position between queued prompts and input.
3. **Queued prompts are below input** — Should be moved above input.
