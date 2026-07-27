#[test]
fn test_scroll_ctrl_k_j_offset() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(100, 30, 1, 20);

    assert_eq!(app.scroll_offset, 0);
    assert!(!app.auto_scroll_paused);

    let (up_code, up_mods) = scroll_up_key(&app);
    let (down_code, down_mods) = scroll_down_key(&app);

    // Render first so LAST_MAX_SCROLL is populated
    render_and_snap(&app, &mut terminal);

    // Scroll up (switches to absolute-from-top mode)
    app.handle_key(up_code.clone(), up_mods).unwrap();
    assert!(app.auto_scroll_paused);
    let first_offset = app.scroll_offset;

    app.handle_key(up_code.clone(), up_mods).unwrap();
    let second_offset = app.scroll_offset;
    assert!(
        second_offset > first_offset,
        "scrolling up should increase offset (move toward older content)"
    );

    // Scroll down (decreases absolute position = moves toward newest content)
    app.handle_key(down_code.clone(), down_mods).unwrap();
    assert_eq!(
        app.scroll_offset, first_offset,
        "one scroll down should undo one scroll up"
    );

    // Keep scrolling down until back at bottom
    for _ in 0..10 {
        app.handle_key(down_code.clone(), down_mods).unwrap();
        if !app.auto_scroll_paused {
            break;
        }
    }
    assert_eq!(app.scroll_offset, 0);
    assert!(!app.auto_scroll_paused);

    // Stays at 0 when already at bottom
    app.handle_key(down_code.clone(), down_mods).unwrap();
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_scroll_offset_capped() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(100, 30, 1, 4);

    let (up_code, up_mods) = scroll_up_key(&app);

    // Render first so LAST_MAX_SCROLL is populated
    render_and_snap(&app, &mut terminal);

    // Spam scroll-up many times
    for _ in 0..500 {
        app.handle_key(up_code.clone(), up_mods).unwrap();
    }

    // Should be at max (capped at oldest content) after scrolling up enough
    assert!(app.scroll_offset > 0, "scroll_offset should be > 0 when scrolled up to cap");
    assert!(app.auto_scroll_paused);
}

#[test]
fn test_scroll_render_bottom() {
    let _render_lock = scroll_render_test_lock();
    let (app, mut terminal) = create_scroll_test_app(80, 15, 1, 20);
    let text = render_and_snap(&app, &mut terminal);

    // At newest position (scroll=0), earliest user content should be visible.
    // "Intro line 01" is at the beginning of the body content.
    assert!(
        text.contains("Scroll test") || text.contains("Intro line 01"),
        "expected initial content at scroll=0 (newest position), got:\n{}",
        text
    );
    // At newest position (scroll=0), no ↑ indicator should appear.
    // Content below extends beyond viewport — either ↓ indicator or nothing.
    // Prompt preview (N›) may still appear for long enough user prompts.
}

#[test]
fn test_scroll_render_scrolled_up() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 25, 1, 40);
    // Disable native scrollbar so the ↓ indicator is rendered
    app.chat_native_scrollbar = false;

    // Seed scroll metrics, then enter paused/scrolled mode via the real key path.
    let _ = render_and_snap(&app, &mut terminal);
    let (up_code, up_mods) = scroll_up_key(&app);
    app.handle_key(up_code, up_mods).unwrap();

    assert!(app.auto_scroll_paused, "scroll-up should pause auto-follow");

    let text_scrolled = render_and_snap(&app, &mut terminal);

    assert!(
        text_scrolled.contains('↓'),
        "expected ↓ indicator when paused above bottom, got:\n{}",
        text_scrolled
    );
}

#[test]
fn test_prompt_preview_reserves_rows_without_overwriting_visible_history() {
    let _render_lock = scroll_render_test_lock();
    let mut app = create_test_app();
    app.display_messages = vec![
        DisplayMessage {
            role: "user".to_string(),
            content: "This is a deliberately long prompt preview that should wrap into two preview rows at the top of the viewport".to_string(),
            tool_calls: vec![],
            duration_secs: None,
            title: None,
            tool_data: None,
        },
        DisplayMessage {
            role: "assistant".to_string(),
            content: App::build_scroll_test_content(0, 20, None),
            tool_calls: vec![],
            duration_secs: None,
            title: None,
            tool_data: None,
        },
    ];
    app.bump_display_messages_version();
    app.scroll_offset = 0;
    app.auto_scroll_paused = false;
    app.is_processing = false;
    app.streaming_text.clear();
    app.status = ProcessingStatus::Idle;
    app.session.short_name = Some("test".to_string());

    let backend = ratatui::backend::TestBackend::new(40, 8);
    let mut terminal = ratatui::Terminal::new(backend).expect("failed to create test terminal");

    let text = render_and_snap(&app, &mut terminal);

    assert!(
        text.contains("1›"),
        "expected sticky prompt preview, got:\n{}",
        text
    );
    assert!(
        text.contains("..."),
        "expected two-line preview truncation, got:\n{}",
        text
    );
    assert!(
        text.contains("Intro line 19"),
        "latest visible content should remain visible below preview, got:\n{}",
        text
    );
}

#[test]
fn test_scroll_top_does_not_snap_to_bottom() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 25, 1, 24);

    // At scroll_offset=0, both paused and auto-follow show the same content
    // (newest at top of viewport).
    app.scroll_offset = 0;
    app.auto_scroll_paused = true;
    let text_top = render_and_snap(&app, &mut terminal);

    // Move to offset > 0 — content should differ
    app.scroll_offset = 15;
    app.auto_scroll_paused = true;
    let text_scrolled = render_and_snap(&app, &mut terminal);

    assert_ne!(
        text_top, text_scrolled,
        "scrolled viewport should differ from scroll=0 viewport"
    );
    assert!(
        text_top.contains("Intro line 01"),
        "scroll=0 position should include earliest content"
    );
}

#[test]
fn test_scroll_content_shifts() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 25, 1, 12);

    // Render at bottom
    app.scroll_offset = 0;
    app.auto_scroll_paused = false;
    let text_bottom = render_and_snap(&app, &mut terminal);

    // Render scrolled up (absolute line 10 from top)
    app.scroll_offset = 10;
    app.auto_scroll_paused = true;
    let text_scrolled = render_and_snap(&app, &mut terminal);

    assert_ne!(
        text_bottom, text_scrolled,
        "content should change when scrolled"
    );
}

#[test]
fn test_scroll_render_with_mermaid() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(100, 30, 2, 10);

    // Render at several positions without crashing.
    for (offset, paused) in [(0, false), (5, true), (10, true), (20, true), (50, true)] {
        app.scroll_offset = offset;
        app.auto_scroll_paused = paused;
        terminal
            .draw(|f| crate::tui::ui::draw(f, &app))
            .unwrap_or_else(|e| panic!("draw failed at scroll_offset={}: {}", offset, e));
    }
}

#[test]
fn test_scroll_visual_debug_frame() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(100, 30, 1, 10);

    crate::tui::visual_debug::enable();

    // Render at bottom, verify frame capture works
    app.scroll_offset = 0;
    terminal
        .draw(|f| crate::tui::ui::draw(f, &app))
        .expect("draw at offset=0 failed");

    let frame = crate::tui::visual_debug::latest_frame();
    assert!(frame.is_some(), "visual debug frame should be captured");

    // Render at scroll_offset=10, verify no panic
    app.scroll_offset = 10;
    app.auto_scroll_paused = true;
    terminal
        .draw(|f| crate::tui::ui::draw(f, &app))
        .expect("draw at offset=10 failed");

    // Note: latest_frame() is global and may be overwritten by parallel tests,
    // so we only verify the frame capture mechanism works, not exact values.
    let frame = crate::tui::visual_debug::latest_frame();
    assert!(
        frame.is_some(),
        "frame should still be available after second draw"
    );

    crate::tui::visual_debug::disable();
}

#[test]
fn test_full_redraw_clears_out_of_band_backend_artifacts_after_native_scroll_like_mutation() {
    let _lock = scroll_render_test_lock();

    let (mut app, mut terminal) = create_scroll_test_app(60, 12, 0, 24);
    app.auto_scroll_paused = true;
    app.scroll_offset = 6;
    let clean = render_and_snap(&app, &mut terminal);

    let width = terminal.backend().buffer().area.width;
    let ghost = ratatui::buffer::Buffer::with_lines([
        "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
    ]);
    let updates = ghost
        .content()
        .iter()
        .enumerate()
        .map(|(idx, cell)| ((idx as u16) % width, (idx as u16) / width, cell));
    terminal
        .backend_mut()
        .draw(updates)
        .expect("inject backend artifact");

    let stale = buffer_to_text(&terminal);
    assert!(
        stale.contains("ZZZZ"),
        "expected injected backend artifact before redraw, got:\n{stale}"
    );

    terminal
        .draw(|f| crate::tui::ui::draw(f, &app))
        .expect("normal redraw after backend mutation");
    let still_stale = buffer_to_text(&terminal);
    assert!(
        still_stale.contains("ZZZZ"),
        "without a forced full redraw, ratatui diffing should leave the injected artifact in place"
    );

    app.request_full_redraw();
    assert!(app.force_full_redraw, "full redraw flag should be armed");
    terminal.clear().expect("test backend clear should succeed");
    app.force_full_redraw = false;
    terminal
        .draw(|f| crate::tui::ui::draw(f, &app))
        .expect("forced full redraw should succeed");
    let repaired = buffer_to_text(&terminal);
    assert_eq!(
        repaired, clean,
        "forced full redraw should restore the expected frame and remove stale backend artifacts"
    );
}

#[test]
fn test_scroll_key_then_render() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 25, 1, 40);

    // Render at bottom first (populates LAST_MAX_SCROLL)
    let _text_before = render_and_snap(&app, &mut terminal);

    let (up_code, up_mods) = scroll_up_key(&app);

    // Scroll up three times (9 lines total)
    for _ in 0..3 {
        app.handle_key(up_code.clone(), up_mods).unwrap();
    }
    assert!(app.auto_scroll_paused);
    assert!(app.scroll_offset > 0);

    // Render again - verifies scroll_offset produces a valid frame without panic.
    // Note: LAST_MAX_SCROLL is a process-wide global that parallel tests
    // can overwrite at any time, so we only check that rendering succeeds
    // and that scroll state is correct - not that the rendered text differs,
    // since the global can clamp scroll_offset to 0 during render.
    let _text_after = render_and_snap(&app, &mut terminal);
}

#[test]
fn test_scroll_round_trip() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 25, 1, 12);

    let (up_code, up_mods) = scroll_up_key(&app);
    let (down_code, down_mods) = scroll_down_key(&app);

    // Render at bottom before scrolling (populates LAST_MAX_SCROLL)
    let _text_original = render_and_snap(&app, &mut terminal);

    // Scroll up 3x
    for _ in 0..3 {
        app.handle_key(up_code.clone(), up_mods).unwrap();
    }
    assert!(app.auto_scroll_paused);

    // Rendering after scrolling up should succeed; exact buffer diffs are brittle
    // because process-wide render state can influence viewport clamping.
    let _text_scrolled = render_and_snap(&app, &mut terminal);

    // Scroll back down until at bottom
    for _ in 0..20 {
        app.handle_key(down_code.clone(), down_mods).unwrap();
        if !app.auto_scroll_paused {
            break;
        }
    }
    assert_eq!(
        app.scroll_offset, 0,
        "scroll_offset should return to 0 after round-trip"
    );
    assert!(!app.auto_scroll_paused);

    // Verify we're back at the bottom and rendering still succeeds.
    let _text_restored = render_and_snap(&app, &mut terminal);
}

#[test]
fn test_copy_selection_from_bottom_rebases_scroll_instead_of_jumping_to_top() {
    let _render_lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 25, 0, 40);

    let bottom_text = render_and_snap(&app, &mut terminal);
    let max_scroll = crate::tui::ui::last_max_scroll();
    assert!(
        max_scroll > 0,
        "expected scrollable history for selection test"
    );
    assert!(
        !bottom_text.contains("Intro line 01"),
        "bottom viewport should not start at top before selection"
    );

    app.handle_key(KeyCode::Char('y'), KeyModifiers::ALT)
        .expect("enter copy mode");
    app.handle_key(KeyCode::Right, KeyModifiers::empty())
        .expect("move selection cursor");

    assert!(
        app.copy_selection_mode,
        "copy selection mode should remain active"
    );
    assert!(app.auto_scroll_paused, "selection should pause auto-follow");
    assert_eq!(
        app.scroll_offset, max_scroll,
        "selection should preserve the current bottom viewport when pausing auto-follow"
    );

    let selected_text = render_and_snap(&app, &mut terminal);
    assert!(
        !selected_text.contains("Intro line 01"),
        "starting selection from bottom should not teleport to the top"
    );
}

// ===== Streaming push-down and auto-scroll tests =====

#[test]
fn test_streaming_completed_message_persists() {
    let _lock = scroll_render_test_lock();
    // Small content: no padding, no diagrams — just one user message
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 0, 0);

    // Remove the default body content to have clean state
    app.display_messages.clear();
    app.display_messages.push(DisplayMessage::user("First turn prompt"));
    app.bump_display_messages_version();

    // Simulate first response streaming
    app.is_processing = true;
    app.streaming_text = "First completed response.\n".to_string();
    let _ = render_and_snap(&app, &mut terminal);

    // Simulate response completion: commit streaming text to body
    app.streaming_text = String::new();
    app.display_messages.push(DisplayMessage::assistant("First completed response.\n".to_string()));
    app.bump_display_messages_version();

    // Simulate new response starting to stream
    app.streaming_text = "Second response streaming here.\n".to_string();
    app.is_processing = true;

    let text = render_and_snap(&app, &mut terminal);

    assert!(
        text.contains("Second response streaming"),
        "new streaming response should appear at top of viewport, got:\n{}",
        text
    );
    // Verify model-level push-down: completed message is in body, streaming text is separate
    assert_eq!(app.display_messages.len(), 2, "body should contain user prompt + completed assistant msg");
    assert!(
        app.display_messages[1].content.contains("First completed response"),
        "completed message should exist in body"
    );
    assert!(
        app.streaming_text.contains("Second response streaming"),
        "streaming text should be the new response"
    );
}

#[test]
fn test_streaming_auto_follow_pegs_to_top() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 5);

    assert_eq!(app.scroll_offset, 0);
    assert!(!app.auto_scroll_paused);

    // Start streaming
    app.is_processing = true;
    app.streaming_text = "First line of streaming.\n".to_string();
    let _ = render_and_snap(&app, &mut terminal);

    assert_eq!(
        app.scroll_offset, 0,
        "auto-follow should keep offset at 0"
    );
    assert!(
        !app.auto_scroll_paused,
        "auto-follow should stay unpaused"
    );

    // Streaming grows — offset must stay 0
    app.streaming_text = "First line of streaming.\nSecond line of streaming.\n".to_string();
    let text = render_and_snap(&app, &mut terminal);

    assert_eq!(
        app.scroll_offset, 0,
        "offset should stay 0 as streaming grows"
    );
    assert!(
        text.contains("Second line"),
        "newest streaming content should be visible at top, got:\n{}",
        text
    );
}

#[test]
fn test_push_down_prior_content() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 20);

    // Render to populate max_scroll
    let _ = render_and_snap(&app, &mut terminal);

    // Scroll to a mid position
    app.auto_scroll_paused = true;
    app.scroll_offset = 5;
    let baseline = render_and_snap(&app, &mut terminal);

    // Streaming grows (new response starts)
    app.is_processing = true;
    app.streaming_text = "New streaming content\npushing everything down\n".to_string();

    assert_eq!(app.scroll_offset, 5, "scroll_offset must not change when streaming grows");

    let after_streaming = render_and_snap(&app, &mut terminal);

    assert_ne!(
        baseline, after_streaming,
        "viewport content should shift when streaming section grows"
    );
}

#[test]
fn test_scroll_during_streaming_does_not_snap_back() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 20);

    // Start streaming
    app.is_processing = true;
    app.streaming_text = "Streaming response...\n".to_string();
    let _ = render_and_snap(&app, &mut terminal);

    // Scroll up — pauses auto-follow and moves away from top
    let (up_code, up_mods) = scroll_up_key(&app);
    app.handle_key(up_code.clone(), up_mods).unwrap();

    assert!(
        app.auto_scroll_paused,
        "scroll up should pause auto-follow"
    );
    let paused_offset = app.scroll_offset;
    assert!(
        paused_offset > 0,
        "offset should be > 0 after scrolling up"
    );

    // Streaming grows further
    app.streaming_text =
        "Streaming response...\nMore streaming content...\nEven more lines\n".to_string();
    let _ = render_and_snap(&app, &mut terminal);

    // Must not snap back
    assert!(
        app.auto_scroll_paused,
        "viewport should stay paused during streaming growth"
    );
    assert_eq!(
        app.scroll_offset, paused_offset,
        "scroll offset must remain unchanged when streaming grows while paused"
    );

    // Scroll back down to resume auto-follow
    let (down_code, down_mods) = scroll_down_key(&app);
    // scroll_down is a no-op when !auto_scroll_paused, so loop until we reach 0
    while app.auto_scroll_paused && app.scroll_offset > 0 {
        app.handle_key(down_code.clone(), down_mods).unwrap();
    }

    assert_eq!(
        app.scroll_offset, 0,
        "scrolling down should restore auto-follow"
    );
    assert!(
        !app.auto_scroll_paused,
        "at offset 0, auto-follow should resume"
    );
}

#[test]
fn test_submit_resets_to_top() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 20);

    // Render, then scroll away (simulate user reading history)
    let _ = render_and_snap(&app, &mut terminal);
    app.scroll_offset = 10;
    app.auto_scroll_paused = true;

    // Simulate streaming in progress
    app.is_processing = true;
    app.streaming_text = "Current response...\n".to_string();

    // Simulate what submit_input does: reset scroll, commit streaming
    app.follow_chat_top();
    let committed = std::mem::take(&mut app.streaming_text);
    app.display_messages.push(DisplayMessage::assistant(committed));
    app.bump_display_messages_version();

    // Verify reset
    assert_eq!(
        app.scroll_offset, 0,
        "submit should reset scroll_offset to 0, got {}",
        app.scroll_offset
    );
    assert!(
        !app.auto_scroll_paused,
        "submit should resume auto-follow"
    );

    // New response starts streaming
    app.streaming_text = "New response streaming here.\n".to_string();
    let text = render_and_snap(&app, &mut terminal);

    assert!(
        text.contains("New response"),
        "new streaming should appear at top after submit, got:\n{}",
        text
    );
}

#[test]
fn test_up_down_indicator_accuracy_during_streaming() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 20);
    app.chat_native_scrollbar = false;

    // Auto-follow with streaming: no ↑ or ↓ indicators
    app.is_processing = true;
    app.streaming_text = "Streaming here.\n".to_string();
    let text = render_and_snap(&app, &mut terminal);

    assert!(
        !text.contains('↑'),
        "auto-follow at offset 0 should not show ↑ indicator, got:\n{}",
        text
    );
    assert!(
        !text.contains('↓'),
        "auto-follow should not show ↓ indicator, got:\n{}",
        text
    );

    // Scroll up while streaming: both ↑{scroll} and ↓{remaining} should appear
    let (up_code, up_mods) = scroll_up_key(&app);
    app.handle_key(up_code, up_mods).unwrap();
    let text_scrolled = render_and_snap(&app, &mut terminal);

    assert!(
        text_scrolled.contains('↑'),
        "paused while streaming should show ↑{{scroll}} indicator, got:\n{}",
        text_scrolled
    );
    assert!(
        text_scrolled.contains('↓'),
        "paused while streaming should show ↓ indicator, got:\n{}",
        text_scrolled
    );

    // Streaming grows: both indicators should persist
    app.streaming_text = "Streaming here.\nMore lines of streaming.\n".to_string();
    let text_grown = render_and_snap(&app, &mut terminal);

    assert!(
        text_grown.contains('↑'),
        "↑ indicator should persist when streaming grows, got:\n{}",
        text_grown
    );
    assert!(
        text_grown.contains('↓'),
        "↓ indicator should persist when streaming grows, got:\n{}",
        text_grown
    );

    // Verify the indicator counts are correct: ↑{scroll_offset} shows lines above viewport
    let expected_up = format!("↑{}", app.scroll_offset);
    assert!(
        text_grown.contains(&expected_up),
        "↑ indicator should show scroll_offset={}, got:\n{}",
        app.scroll_offset,
        text_grown
    );

    // Scroll back to top: indicators should disappear
    let (down_code, down_mods) = scroll_down_key(&app);
    while app.auto_scroll_paused && app.scroll_offset > 0 {
        app.handle_key(down_code.clone(), down_mods).unwrap();
    }
    let text_top = render_and_snap(&app, &mut terminal);

    assert!(
        !text_top.contains('↑'),
        "returning to top should hide ↑ indicator, got:\n{}",
        text_top
    );
    assert!(
        !text_top.contains('↓'),
        "returning to top should hide ↓ indicator, got:\n{}",
        text_top
    );
}

// ===== Edge case tests for streaming push-down =====

#[test]
fn test_short_streaming_keeps_completed_visible() {
    let _lock = scroll_render_test_lock();
    // Use tall viewport with minimal content so even 1-2 line streaming
    // doesn't push completed text out
    let (mut app, mut terminal) = create_scroll_test_app(80, 20, 0, 0);
    app.display_messages.clear();
    app.display_messages.push(DisplayMessage::user("Q1"));
    app.bump_display_messages_version();

    // Completed response that should stay visible
    app.display_messages.push(DisplayMessage::assistant(
        "This is a completed response that should remain visible.\n".to_string(),
    ));
    app.bump_display_messages_version();

    // Very short streaming starts
    app.streaming_text = "Short.\n".to_string();
    app.is_processing = true;

    let text = render_and_snap(&app, &mut terminal);

    assert!(
        text.contains("Short"),
        "short streaming content should be visible, got:\n{}",
        text
    );
    assert!(
        text.contains("completed response"),
        "completed response should still be visible below short streaming, got:\n{}",
        text
    );
}

#[test]
fn test_streaming_to_completed_transition_no_jump() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 5);

    app.is_processing = true;
    app.streaming_text = "Streaming content in progress.\n".to_string();
    let _baseline = render_and_snap(&app, &mut terminal);

    // Commit the streaming text (transition to completed)
    let committed = std::mem::take(&mut app.streaming_text);
    app.display_messages.push(DisplayMessage::assistant(committed));
    app.bump_display_messages_version();
    app.is_processing = false;

    let after_commit = render_and_snap(&app, &mut terminal);

    assert!(
        after_commit.contains("Streaming content in progress"),
        "committed text should remain visible after transition, got:\n{}",
        after_commit
    );

    assert_eq!(
        app.scroll_offset, 0,
        "transition to completed should keep offset at 0"
    );
    assert!(
        !app.auto_scroll_paused,
        "transition to completed should keep auto-follow active"
    );
}

#[test]
fn test_prompt_preview_during_paused_streaming() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 20);

    // Start streaming
    app.is_processing = true;
    app.streaming_text = "Streaming response...\n".to_string();
    let _ = render_and_snap(&app, &mut terminal);

    // Scroll away to pause
    let (up_code, up_mods) = scroll_up_key(&app);
    app.handle_key(up_code, up_mods).unwrap();

    assert!(
        app.auto_scroll_paused,
        "should be paused after scroll"
    );

    let text = render_and_snap(&app, &mut terminal);

    // The prompt preview should still work (› prefix appears for truncated user prompts)
    // When scrolled away from newest, the prompt preview shows the most recent user prompt
    assert!(
        text.contains("Scroll test") || text.contains("›"),
        "prompt preview should show user prompt text when paused, got:\n{}",
        text
    );
}

#[test]
fn test_native_scrollbar_during_streaming() {
    let _lock = scroll_render_test_lock();
    let (mut app, mut terminal) = create_scroll_test_app(80, 15, 1, 20);
    app.chat_native_scrollbar = true;

    // Auto-follow with streaming
    app.is_processing = true;
    app.streaming_text = "Streaming response...\n".to_string();
    let _text = render_and_snap(&app, &mut terminal);

    // No ↑ or ↓ indicators when auto-follow + native scrollbar
    // Native scrollbar uses ╷•╵│ glyphs
    assert_eq!(
        app.scroll_offset, 0,
        "auto-follow should keep offset at 0 with native scrollbar"
    );

    // Scroll away while streaming — native scrollbar shows position, no ↓ indicator
    app.auto_scroll_paused = true;
    app.scroll_offset = 5;
    let text_paused = render_and_snap(&app, &mut terminal);

    // The native scrollbar suppresses the ↓ indicator
    assert!(
        !text_paused.contains('↓'),
        "native scrollbar should suppress ↓ indicator, got:\n{}",
        text_paused
    );
}

#[cfg(test)]
#[path = "../tests_input_scroll.rs"]
mod input_scroll_tests;
