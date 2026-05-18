# Feature Specification: Streaming Content Auto-Scroll (Push-Down)

**Feature Branch**: `003-streaming-auto-scroll`  
**Created**: 2026-05-18  
**Status**: Draft  
**Input**: User description: "Completed response text should stay visible and be pushed down as a new streaming response enters at the top of the message area. The behavior must mimic a natural chat feed where new content at the top pushes older content downward."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Completed Response Stays Visible (Priority: P1)

A developer has just received a full assistant response. The text is complete and sitting directly below the input box at the top of the message area. They are reading it. They type a new question and press Enter. The completed response they were reading remains visible right below the new streaming text, pushed down naturally.

**Why this priority**: This is the core scroll-feel problem. If completed text vanishes or the viewport jumps around, the user loses their place and trust in the UI.

**Independent Test**: Start a conversation with an assistant. Wait for the response to complete. Send a second message. Verify the first completed response is still visible below the new streaming text without any viewport jump.

**Acceptance Scenarios**:

1. **Given** a completed assistant response is visible at the top of the message area, **When** the user submits a new prompt, **Then** the completed response remains visible at its current position and new streaming content appears above it (closer to the input box).
2. **Given** a completed assistant response is visible, **When** a new streaming response starts entering at the top, **Then** the completed response scrolls downward naturally as the streaming content grows in line count.
3. **Given** multiple prior turns of completed messages, **When** a new response streams in, **Then** all prior completed content scrolls down together without jitter or teleportation.

---

### User Story 2 - Auto-follow During Streaming (Priority: P1)

A developer is watching a long response stream in. The viewport stays pegged to the newest content (the streaming text at the top). As each new line is appended to the streaming section, the viewport automatically follows so the user always sees the latest text being typed.

**Why this priority**: If streaming text scrolls off-screen during generation, the user cannot read the assistant's response as it arrives — defeating the purpose of streaming output.

**Independent Test**: Trigger a long streaming response (50+ lines). Verify that each new line of streaming text appears at the top of the visible area and is immediately readable without manual scrolling.

**Acceptance Scenarios**:

1. **Given** auto-scroll is active (user is at newest content), **When** a new line of streaming text is appended, **Then** the viewport stays at the top of the message area showing the latest line of the streaming response.
2. **Given** auto-scroll is active, **When** the streaming section grows by N lines, **Then** scroll_offset remains at 0 and the streaming text remains fully visible at the top.

---

### User Story 3 - Manual Scroll During Streaming (Priority: P2)

A developer is watching a long response stream in. They want to scroll down and re-read something earlier in the conversation. They scroll away (Ctrl+J / scroll down) and the auto-follow pauses. The streaming continues in the background but does not jerk the viewport back.

**Why this priority**: Users need to reference earlier parts of the conversation while waiting, without being forced back to the top.

**Independent Test**: During a streaming response, press Ctrl+J to scroll down into older content. Verify the viewport stays at the user-chosen scroll position and does not snap back to the top when new streaming text arrives.

**Acceptance Scenarios**:

1. **Given** a streaming response is in progress and auto-scroll is active, **When** the user scrolls down (Ctrl+J) to read older content, **Then** auto-scroll pauses and the viewport stays at the user's scrolled position.
2. **Given** auto-scroll is paused and streaming continues, **When** new lines are appended to the streaming section, **Then** the viewport does NOT jump back to the top.
3. **Given** auto-scroll is paused, **When** the user scrolls back up to scroll_offset = 0, **Then** auto-follow resumes and the viewport re-pegs to the streaming text.

---

### User Story 4 - Submit Resets to Follow (Priority: P2)

A developer was scrolled back reading old history. They type a new question and press Enter. The viewport snaps to the top (newest content) so they can immediately see the new response streaming in.

**Why this priority**: If the user stays scrolled into history after submitting, they won't see the new response at all — a confusing dead-end.

**Independent Test**: Scroll deep into history with auto-scroll paused. Type a new prompt and press Enter. Verify the viewport immediately snaps to scroll_offset = 0 and the input area + streaming area are visible.

**Acceptance Scenarios**:

1. **Given** auto-scroll is paused at a deep scroll offset (user reading history), **When** the user submits a new prompt, **Then** the viewport snaps to scroll_offset = 0 and auto-scroll resumes.
2. **Given** the user submits a new prompt from any scroll position, **When** the first line of the new streaming response appears, **Then** it is visible at the top of the message area directly below the input box.

---

### Edge Cases

- What happens when the streaming content is very short (1-2 lines)? The completed response below it should not be pushed entirely out of the viewport — at least one prior message should stay visible.
- What happens when the user resizes the terminal during streaming? The viewport should maintain its peg to the top if auto-following, or maintain scroll_offset relative position if paused.
- What happens when streaming finishes and transitions to "completed" status? No visual jump — the text stays in place, only the typing cursor/animation disappears.
- What happens with prompt preview enabled? When scrolled away from newest, the sticky prompt preview should appear at the top of the message area as before.
- What happens when native scrollbar is enabled? The scrollbar thumb should be at position 0 (top) when auto-following, and at proportional offset when paused.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST render the streaming response at the top of the message area, directly below the input/chrome section, as the first content the user sees.
- **FR-002**: System MUST preserve the visual position of already-completed messages when new streaming text enters above them (push-down behavior).
- **FR-003**: System MUST maintain auto-follow to the top (scroll_offset = 0) during streaming when the user has not manually scrolled away.
- **FR-004**: System MUST pause auto-follow when the user scrolls away from the top during active streaming, and NOT forcibly snap back.
- **FR-005**: System MUST allow the user to re-engage auto-follow by scrolling back to scroll_offset = 0 (scroll up / Ctrl+K toward newest).
- **FR-006**: System MUST snap to scroll_offset = 0 and resume auto-follow when the user submits a new prompt (Enter).
- **FR-007**: System MUST render scroll position indicators (↑N, ↓N) correctly reflecting the top-down coordinate system: ↑ means "N newer lines above" (user scrolled down), ↓ means "N older lines below" (content extends past viewport).
- **FR-008**: The native scrollbar thumb position MUST reflect the top-down coordinate system: position 0 = newest content at top, max position = oldest content at bottom.

### Key Entities

- **scroll_offset**: The first visible line index in the message area. 0 = newest content (Streaming section). Higher values = older content (toward Header section).
- **auto_scroll_paused**: Boolean flag. False = viewport tracks newest content. True = user has scrolled away manually.
- **Streaming section**: The topmost section of the prepared chat frame, containing in-progress assistant text that is growing line by line.
- **Body section**: The middle section containing completed message turns (both user and assistant messages).
- **Header section**: The bottommost section containing compacted history or context header.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A completed assistant response remains visible at the top of the message area after the user submits a follow-up prompt (no content vanishes from view).
- **SC-002**: Streaming text is always visible at the top of the viewport when auto-scroll is active — users never need to manually scroll to see incoming text.
- **SC-003**: Users who scroll away during streaming see zero viewport jumps back to the top unless they explicitly scroll back or submit a new prompt.
- **SC-004**: Submitting a new prompt from any scroll position always results in the viewport showing the top of the message area within one render frame.
- **SC-005**: Scroll indicators (↑N, ↓N) accurately report the user's position relative to newest/oldest content in all streaming, completed, and paused states.

## Assumptions

- The top-down layout flip (spec 001) is complete and the section order is Streaming → BatchProgress → Body → Header, with scroll_offset = 0 meaning newest content at top.
- The `follow_chat_top()` transition at submit time is already implemented and correctly sets scroll_offset = 0 / auto_scroll_paused = false.
- The rendering engine in `ui_prepare.rs` / `ui_viewport.rs` already builds the PreparedChatFrame in the correct section order.
- The prompt navigation bugs identified in the `tui-scroll-logic-gauntlet` spec are already resolved and do not interfere with this feature.
