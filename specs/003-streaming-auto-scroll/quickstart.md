# Quickstart: Streaming Content Auto-Scroll

**Phase**: 1 | **Date**: 2026-05-18

## What This Feature Changes

This feature adds NO new code. The push-down behavior is emergent from the existing architecture. What it adds is **verification tests** to confirm the invariants hold, and potentially minor guardrails in existing methods.

## Files to Modify

### 1. `src/tui/app/tests/scroll_copy_03.rs` — Add 3 tests

Tests confirming push-down behavior:

```rust
#[test]
/// Streaming growth at auto_follow: scroll_offset stays 0, content shifts
fn test_streaming_push_down_while_auto_follow() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 8);

    // Render baseline
    let baseline = render_and_snap(&app, &mut terminal);

    // Attach streaming text (simulating new response starting)
    app.is_processing = true;
    app.streaming_text = "Streaming: line 01\nStreaming: line 02\nStreaming: line 03\n".to_string();

    assert_eq!(app.scroll_offset, 0);
    assert!(!app.auto_scroll_paused);

    let with_streaming = render_and_snap(&app, &mut terminal);

    assert!(
        with_streaming.contains("Streaming: line 01"),
        "streaming text should appear at top when auto-following, got:\n{}",
        with_streaming
    );
}
```

### 2. `src/tui/app/tests/scroll_copy_03.rs` — Add 2 more tests

```rust
#[test]
/// Streaming growth while paused: scroll_offset unchanged, content shifts down
fn test_streaming_push_down_while_paused() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 20);

    // Render to populate max_scroll
    let _ = render_and_snap(&app, &mut terminal);
    let max_before = crate::tui::ui::last_max_scroll();

    // Pause and scroll to position
    app.auto_scroll_paused = true;
    app.scroll_offset = max_before.saturating_sub(3); // 3 lines from bottom

    let baseline = render_and_snap(&app, &mut terminal);

    // Simulate streaming growth
    app.is_processing = true;
    app.streaming_text = "new\nlines\n".to_string();

    // scroll_offset unchanged
    assert_eq!(app.scroll_offset, max_before.saturating_sub(3));

    let after_streaming = render_and_snap(&app, &mut terminal);

    // The content should differ (pushed down) but scroll_offset same
    assert_ne!(baseline, after_streaming, "content should shift when streaming grows");
}

#[test]
/// Commit streaming then new streaming: push-down persists across transition
fn test_streaming_commit_then_new_streaming() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 5);

    app.is_processing = true;
    app.streaming_text = "First response streaming...".to_string();

    let with_first_stream = render_and_snap(&app, &mut terminal);

    // Commit the streaming text
    app.commit_pending_streaming_assistant_message();
    app.is_processing = true; // re-enable since commit clears it
    app.streaming_text = "Second response streaming...".to_string();

    let after_second = render_and_snap(&app, &mut terminal);

    assert!(
        after_second.contains("Second response streaming"),
        "new streaming should appear at top, got:\n{}",
        after_second
    );
    // The old content should still be visible (pushed down)
    // "Intro line" comes from body content
    assert!(
        after_second.contains("First response"),
        "committed first response should still be visible below streaming, got:\n{}",
        after_second
    );
}
```

### 3. `src/tui/ui_viewport.rs` — Verify scroll indicator direction

No code change needed. Just verify that `↑N` and `↓N` indicators use the correct comparison for top-down layout checked above in research.md.

## Implementation Sequence

1. Add 3 test cases to `scroll_copy_03.rs`
2. Run `cargo test test_scroll -p jcode` — all should pass
3. Verify indicators manually or add indicator-specific snapshot assertions if needed

## Validation

- `cargo check` — no new compilation errors
- `cargo test test_streaming_push_down -p jcode` — new tests pass
- `cargo test test_scroll -p jcode` — all 14 existing tests still pass
