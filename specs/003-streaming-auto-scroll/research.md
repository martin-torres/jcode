# Research: Streaming Content Auto-Scroll Push-Down

**Phase**: 0 | **Date**: 2026-05-18 | **For**: specs/003-streaming-auto-scroll/spec.md

## 1. Section Transition Flicker (streaming → completed)

**Question**: When `commit_pending_streaming_assistant_message()` moves text from the Streaming section (raw wrapping) to the Body section (markdown rendering), could the line count differ by 1-2 lines causing a visible jump?

**Investigation**:
- `prepare_streaming_cached` in `ui_prepare.rs` wraps `streaming_text` directly via `textwrap::wrap()` at `content_width`. It produces raw wrapped lines.
- `prepare_body` calls `crate::tui::markdown::plain_lines()` for the full markdown pipeline, which also wraps at `content_width` but may produce different line breaks due to markdown transformations.
- However, the streaming text is *already rendered as plain text* (it's being typed in real-time, not markdown-parsed). When committed, `push_display_message(DisplayMessage::assistant(content))` stores the raw string. On the next render, `prepare_body` processes it through the full pipeline.

**Risk**: For simple text (no markdown), line wrapping should be identical. For markdown content (code blocks, tables), the pre-commit streaming render is a best-effort approximation, and the post-commit body render is the canonical version. There could be a 1-line shift during the transition.

**Mitigation**: This is a pre-existing behavior in the top-down layout, not introduced by this feature. The push-down auto-scroll behavior is orthogonal to the streaming→body transition. Any flicker from section transition would happen regardless of scroll state.

**Decision**: Section transition flicker is out of scope. Accept the existing behavior. Focus only on scroll-offset management during streaming growth and at transition points.

---

## 2. Auto-scroll Resume Timing on Submit

**Question**: `follow_chat_top()` is called in `submit_input` (input.rs:1841) *before* `commit_pending_streaming_assistant_message()` (input.rs:1847). Does this cause a visual flash of "committed content at top" between render frames?

**Investigation**: Both operations run synchronously in the same event handler. The next `App::tick()` (which triggers a draw) happens after the submit event is fully processed. By then:
1. `follow_chat_top()` → scroll_offset = 0 ✓
2. `commit_pending_streaming_assistant_message()` → old streaming moved to body ✓
3. The send action queues the message → `is_processing` stays true ✓
4. Next render: streaming section is empty (no response yet), so the user sees the committed old response at the top of the body section at scroll_offset=0.

**Result**: No flash. The user sees the committed response at the top of the body, which is correct. When the new response starts streaming, it appears above the committed content in the streaming section.

**Decision**: No work needed. The timing is already correct.

---

## 3. Streaming Growth While Paused

**Question**: When `auto_scroll_paused=true` and the streaming section grows, does the viewport jitter?

**Investigation**: The render loop rebuilds the entire `PreparedChatFrame` from scratch each frame (all 4 sections: Streaming, BatchProgress, Body, Header). When the streaming section grows:
- `total_lines` increases by the new streaming lines
- `max_scroll` increases by the same amount
- `scroll_offset` is unchanged (user's chosen position)
- The lines at `scroll_offset` now reference *different content* — the content the user was looking at has shifted down by `streaming_delta_lines`

**This is the core push-down behavior**: When `auto_scroll_paused=false`, `scroll_offset=0` and the new streaming lines appear at the top of the viewport. When `auto_scroll_paused=true`, the user's selected offset stays the same, meaning the content they're looking at shifts down.

**Existing check**: `draw_messages` uses `scroll = user_scroll.min(max_scroll)` (ui_viewport.rs:247) which caps at the new max. If `user_scroll > new_max` (shouldn't happen normally, since max increases), it clamps.

**Decision**: The current `PreparedChatFrame` model naturally implements push-down because the sections are concatenated in order [Streaming, BatchProgress, Body, Header]. When streaming grows, all subsequent sections shift. `scroll_offset` is absolute from the start of the combined sections, so keeping it unchanged while the streaming prefix grows means the visible offset moves relatively. This is exactly the desired push-down behavior.

**No code change needed for the core push-down mechanism** — it already works architecturally. The only QA is ensuring scroll_offset doesn't exceed max_scroll (already capped).

---

## 4. Indicator Correctness (`↑N` / `↓N`)

**Question**: Do the scroll indicators correctly reflect position in the top-down layout?

**Current code** (ui_viewport.rs:654):
- `↑N` when `scroll > 0` (user scrolled away from newest; N = lines above viewport)
- `↓N` when `auto_scroll_paused && scroll < max_scroll` (older content below viewport; N = lines below)

In top-down layout: `scroll` is the line index from the start of the combined sections. `scroll=0` = newest (Streaming section). `scroll>0` = user has scrolled down toward older content.

- `↑N` = "N newer lines are above the viewport (user has scrolled down)" — but the arrow icon leans on the convention that `↑` means "scroll up to reach this". In the old bottom-up layout this was consistent. In top-down layout, "newer lines above" still makes sense visually — ↑ points toward the input box / newest content.
- `↓N` = "N older lines are below the viewport" — ↓ points toward the header / oldest content.

**Decision**: The indicators are semantically correct for the top-down layout. No change needed.

---

## 5. Test Infrastructure for Streaming Scroll

**Question**: How do we write tests that exercise streaming growth at various scroll states?

**Existing infrastructure**:
- `create_scroll_test_app(w, h, diagrams, padding)` builds an app with a user prompt + assistant response of N lines
- `render_and_snap(app, terminal)` renders a frame and returns text buffer
- Tests directly set `app.scroll_offset` and `app.auto_scroll_paused`
- `app.streaming_text = "..."` can be set directly

**Test strategy**:
1. Set streaming_text to N lines worth of content
2. Render
3. Append to streaming_text
4. Re-render
5. Assert scroll_offset unchanged but rendered content shifted

**Decision**: Direct field access is sufficient for unit testing. No additional test infrastructure needed.

---

## Summary of Findings

| Question | Verdict | Action |
|----------|---------|--------|
| Section transition flicker | Pre-existing, out of scope | Document as known limitation |
| Auto-scroll resume timing | Already correct | No change |
| Streaming growth while paused | Already correct by architecture | No change (push-down is emergent) |
| Indicator correctness | Already correct | No change |
| Test infrastructure | Direct field access sufficient | No new infrastructure needed |

**Overall finding**: The push-down behavior fundamentally works already because the section concatenation model [Streaming, Body, Header] means adding lines to the Streaming section shifts all subsequent sections. `scroll_offset` is absolute from the start and automatically references "further down" content when streaming grows. The main implementation work is:
1. Confirming the behavior with tests
2. Ensuring smooth auto-scroll management at the streaming→body transition
3. Verifying scroll indicator correctness during all states
