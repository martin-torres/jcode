# Feature Specification: Top-Down TUI Layout Flip

**Feature Branch**: `001-top-down-tui-layout-flip`  
**Created**: 2026-05-13  
**Status**: Draft  
**Input**: User description: "Flip the JCode TUI from bottom-up agent harness interaction model to a top-down document-style layout. Input box moves to the top of the screen. Content order reverses so newest output appears at the top of the messages area (just below the input). Auto-scroll anchors at scroll=0 to show the newest content. Scroll direction semantics invert. This is a full architectural inversion of the interactive mechanism, not just a layout swap."

## User Scenarios & Testing

### User Story 1 — Type at Top, See Response Right Below (Priority: P1)

The user opens JCode. The input bar is at the very top of the terminal, always visible. They type a prompt and press Enter. Their prompt appears in the conversation just below the input. The model's response streams out character by character, appearing immediately below the prompt. As the conversation continues, each new turn appears at the top of the conversation (just below the input) and pushes older turns downward.

**Why this priority**: This is the core inversion that defines the entire user experience. Without this, nothing else matters.

**Independent Test**: Launch JCode on the `input-top-layout` branch. Verify the input bar is at the top of the terminal. Type "hello" and press Enter. Confirm the prompt text appears just below the input, not at the bottom of the screen. Wait for the model to respond — confirm the streaming output appears below the prompt, filling downward.

**Acceptance Scenarios**:

1. **Given** JCode is running on the top-down layout branch, **When** the user opens a new session, **Then** the input bar is positioned at the top of the terminal window, below any session header but above the conversation area.
2. **Given** the input bar is at the top, **When** the user types a message and presses Enter, **Then** the message appears as the first visible content in the conversation area, immediately below the input/status area.
3. **Given** the model is generating a response, **When** streaming output arrives, **Then** each new character appears below the previous content, filling the conversation area from top to bottom.

---

### User Story 2 — Scroll Up to See Older Content (Priority: P1)

As the conversation grows long, the most recent exchanges remain visible at the top of the conversation area (just below the input). Older content scrolls out of view below. The user can scroll up (or press the up-arrow/page-up keys) to see older content further down in the conversation. Scrolling down returns them to the most recent content at the top.

**Why this priority**: Scroll direction must be intuitive. The user expects scroll_up to reveal older (chronologically earlier) content, just like every other text-based UI.

**Independent Test**: Scroll upward after a long conversation. Confirm that older messages appear further "up" (i.e., the viewport moves toward content that was generated earlier). Scroll back down and confirm the newest content is visible at the top of the conversation area.

**Acceptance Scenarios**:

1. **Given** a conversation with more content than the visible area, **When** the user presses Page Up or scrolls upward (mouse/trackpad), **Then** the viewport reveals content that was generated earlier (further from the input).
2. **Given** the user has scrolled away from the most recent content, **When** they press the "scroll to latest" key (or equivalent), **Then** the viewport returns to showing the newest content at the top of the conversation area, just below the input.
3. **Given** the model is streaming new output while the user is scrolled away, **When** new content arrives, **Then** the viewport stays at the user's current scroll position (auto-scroll is paused while the user has manually scrolled).

---

### User Story 3 — Prompt Navigation Works in Reverse Order (Priority: P2)

The user can jump between prompts using keyboard shortcuts (e.g., Ctrl+[5-9] or Ctrl+[/]). In the flipped layout, "most recent prompt" means the prompt closest to the input (at the top of the content), and "previous prompt" means the next prompt further down in the conversation.

**Why this priority**: Prompt navigation is a power-user feature. Its direction must match the flipped layout so keyboard shortcuts remain intuitive.

**Independent Test**: Press Ctrl+5 to jump to the most recent prompt. Confirm the viewport moves to show that prompt at the top of the conversation area. Press Ctrl+6 to jump to the second-most-recent prompt, and confirm the viewport moves to show that prompt (further down in content).

**Acceptance Scenarios**:

1. **Given** a multi-turn conversation, **When** the user presses Ctrl+5 (most recent prompt), **Then** the viewport jumps to show the most recent user prompt at the top of the conversation area.
2. **Given** the user is viewing a prompt, **When** they press Ctrl+[ (previous prompt in document order = older), **Then** the viewport moves to show the immediately preceding prompt (chronologically before the current one).

---

### User Story 4 — Auto-Scroll Follows Newest Content at Top (Priority: P2)

When auto-scroll is active (user hasn't manually scrolled), the viewport automatically shows the most recent content at the top of the conversation area. When new output streams in, it appears right at the top of the viewport (below the input), and older content shifts downward.

**Why this priority**: Auto-scroll is critical for a smooth streaming experience. Without correct behavior, the user would either miss new output or have to manually scroll constantly.

**Independent Test**: With auto-scroll active (default), send a prompt and watch the model stream a response. Confirm the streaming text appears at the top of the conversation area (right below the input) and fills downward. Do not touch any scroll keys — confirm the viewport stays anchored at the top showing the latest output.

**Acceptance Scenarios**:

1. **Given** auto-scroll is active (user has not scrolled), **When** the model streams new output, **Then** the output appears at the top of the conversation area (first line of content just below the input area) and each subsequent line appears below the previous one.
2. **Given** auto-scroll is active, **When** a tool call produces output, **Then** the tool output message appears below the current assistant response, continuing the top-to-bottom flow.

---

### Edge Cases

- **What happens when there is no content (empty welcome screen)?** The welcome/suggestion screen should appear below the input, centered or left-aligned in the conversation area, matching the top-down layout.
- **How does the system handle rapid streaming with tool call interleaving?** Each new section (streaming text, batch progress, tool output, user prompt) should appear in chronological order from top to bottom, with the most recently initiated section at the top.
- **What happens when the user scrolls away during streaming?** Auto-scroll pauses. New content continues to stream but the viewport does not jump. When the user scrolls back to the top, auto-scroll resumes.
- **How does prompt preview interact with the flipped layout?** The prompt preview feature (showing context from above the viewport) should adapt to the flipped scroll direction — it shows content that is chronologically earlier (further from input).
- **How do pinned diagrams and side panels interact?** Their position relative to the chat area is unchanged — they occupy a right column or top/bottom section independent of the chat's internal layout.

## Requirements

### Functional Requirements

- **FR-001**: The input bar MUST be positioned at the top of the terminal window, below any session header line but above all conversation content.
- **FR-002**: The conversation area MUST be positioned below the input/status area, filling the remainder of the terminal height.
- **FR-003**: The content order within the conversation area MUST be reversed: newest messages, streaming output, and batch progress MUST appear BEFORE older messages.
- **FR-004**: The auto-scroll anchor MUST be `scroll = 0` (top of content), not `scroll = max_scroll` (bottom of content).
- **FR-005**: When auto-scroll is active and new content arrives, the viewport MUST stay at `scroll = 0`, showing the newest content at the top of the conversation area.
- **FR-006**: Scrolling UP must reveal chronologically older content (content further from the input), and scrolling DOWN must reveal chronologically newer content (closer to the input).
- **FR-007**: The `follow_chat_bottom()` / `follow_chat_top()` function MUST be replaced (or renamed) to anchor at `scroll = 0` instead of `scroll = max_scroll`.
- **FR-008**: Prompt navigation (Ctrl+[5-9], Ctrl+[/], Ctrl+]) MUST operate in the reversed content order — "most recent" is the prompt closest to the input (line index 0).
- **FR-009**: The `pause_chat_auto_scroll()` scroll_offset inversion math MUST be updated to work with `scroll = 0` as the auto-follow target.
- **FR-010**: The welcome/empty state screen MUST render below the input bar, not at the center of the entire terminal.
- **FR-011**: All section positions in `prepare_messages_inner` MUST be reversed: `[Streaming, BatchProgress, Body, Header]` instead of `[Header, Body, BatchProgress, Streaming]`.
- **FR-012**: All absolute line offsets throughout the prepared chat frame (user prompt positions, image regions, edit tool ranges, copy targets) MUST be recalculated to match the reversed section order.
- **FR-013**: Copy badges, copy targets, and any other feature that depends on absolute line indices MUST continue to function correctly after the reversal.
- **FR-014**: `scroll_up()` MUST still reduce `scroll_offset` (moving to smaller line indices), which now points to CHRONOLOGICALLY NEWER content. If this is counterintuitive, the scroll direction semantics must be inverted so that `scroll_up` shows older content (larger line indices).

### Key Entities

- **Layout**: The vertical arrangement of input area, status bar, conversation area, and any additional panes (diagram pane, side panel).
- **Section Order**: The sequence in which content sections (Streaming, BatchProgress, Body, Header) are assembled into the prepared chat frame. Currently `[Header, Body, BatchProgress, Streaming]`. Must become `[Streaming, BatchProgress, Body, Header]`.
- **Scroll Offset**: The integer index into the prepared content lines representing which line is at the top of the viewport. Currently `scroll = max_scroll` for auto-follow. Must become `scroll = 0`.
- **Content Direction**: The mapping between chronological message order and line index order. Currently index 0 = oldest, index N = newest. Must become index 0 = newest, index N = oldest.
- **Auto-Scroll State**: Whether the viewport follows new content automatically. Currently pauses when user scrolls away from `max_scroll`. Must pause when user scrolls away from `0`.

## Success Criteria

### Measurable Outcomes

- **SC-001**: When JCode starts with no conversation, the input bar is at the top and the welcome content (suggestions) appears below it, filling the remaining space.
- **SC-002**: After typing a prompt and receiving a response, the prompt and its response are visible in the conversation area directly below the input (within the first 5 lines of content).
- **SC-003**: Scrolling up (Page Up / arrow up) reveals content that was generated earlier in the conversation. Scrolling down returns to the newest content.
- **SC-004**: Auto-scroll stays anchored to the newest content at the top of the conversation area during streaming. No manual intervention is needed to see new output.
- **SC-005**: After a multi-turn conversation (5+ turns), the oldest turn is at the bottom of the content (scrollable away), and the newest turn is at the top (just below the input).
- **SC-006**: All existing copy/selection features work correctly with the flipped line indices.
- **SC-007**: All existing test suites pass after the changes, with appropriate updates to test expectations.

## Assumptions

- The Rust compiler, Cargo, and all project dependencies are already configured and working.
- The `input-top-layout` git branch already exists and contains the starting codebase state.
- The underlying ratatui rendering framework does not require changes — only the content order and scroll behavior within the application logic.
- Copy targets, image regions, edit tool ranges, and user prompt positions are all recalculated correctly via the existing `from_sections` helper with reversed input.
- The user already has the JCode source code cloned and can build/run it locally.
- No changes are needed to the provider layer, session management, or config system — this is purely a TUI rendering and interaction change.
