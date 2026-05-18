# Feature Specification: VS Code Extension — jcode Client

**Feature Branch**: `002-vscode-extension`
**Created**: 2026-05-16
**Status**: Draft
**Input**: User description: "Create a VS Code extension that connects jcode to VS Code, allowing the coding agent to operate directly within the VS Code workspace with inline file diffs, multi-session management in the sidebar, and full access to jcode's capabilities (memory, swarm, 30+ tools)."

## User Scenarios & Testing

### User Story 1 — Chat with jcode Agent Inside VS Code (Priority: P1)

The user opens VS Code, sees a jcode panel in the sidebar or a dedicated editor tab. They type a prompt like "refactor this function to use async/await" and press Enter. The agent begins working — streaming responses, reading files from the VS Code workspace, editing files, and showing diffs inline. The user can see everything happening in real time without leaving their editor.

**Why this priority**: This is the core value proposition. Without the chat interface, nothing else matters.

**Independent Test**: Install the extension, open a workspace, click the jcode icon in the activity bar. Type "hello" in the chat input. Confirm the agent responds in the chat panel. Verify the response streams in character by character.

**Acceptance Scenarios**:

1. **Given** VS Code is open with an active workspace, **When** the user opens the jcode panel and types a prompt, **Then** the prompt appears in the chat and the agent begins streaming a response.
2. **Given** the agent is generating a response, **When** streaming output arrives from the jcode server, **Then** each new chunk appears in the chat panel in real time.
3. **Given** the user has an active jcode session, **When** they close and reopen VS Code, **Then** the session is preserved and can be resumed.

---

### User Story 2 — Agent Edits Files with Inline Diffs (Priority: P1)

The user asks the agent to make a code change. The agent modifies a file using the edit tool. Instead of just seeing a text log, VS Code shows the diff inline — green additions, red removals — exactly like a git diff. The user can accept or reject the change, or open the diff view to see the full comparison.

**Why this priority**: File editing is what a coding agent does most. Without inline diff visualization, the extension is just a chat client, not a coding tool.

**Independent Test**: Ask the agent "add a comment to the top of my main file." Observe that the file change appears as a diff in VS Code's native diff view or inline with accept/reject buttons.

**Acceptance Scenarios**:

1. **Given** the agent uses the edit tool to modify a file, **When** the tool call completes, **Then** VS Code shows the file change as a diff with added lines (green) and removed lines (red).
2. **Given** a diff is shown, **When** the user clicks "accept," **Then** the change is applied to the file permanently.
3. **Given** a diff is shown, **When** the user clicks "reject," **Then** the file reverts to its original state.

---

### User Story 3 — Session Management in the Sidebar (Priority: P2)

The user can see all their active jcode sessions in the VS Code sidebar. Each session shows its name, status (running/idle/error), and a preview of the last message. The user can click to switch between sessions, create new ones, or close old ones. Sessions persist across VS Code restarts.

**Why this priority**: jcode's strength is multi-session workflows. Without session management, the user is stuck in a single chat.

**Independent Test**: Open the jcode sidebar view. Create two sessions by clicking "+". Click between them. Verify each session has independent conversation history and the agent context is not shared.

**Acceptance Scenarios**:

1. **Given** the jcode sidebar is open, **When** the user clicks the "+" button, **Then** a new session is created and appears in the session list.
2. **Given** multiple sessions exist, **When** the user clicks a different session, **Then** the chat panel switches to that session's conversation.
3. **Given** a session exists, **When** the user hovers over it, **Then** they see a preview of the last message and options to rename or close it.

---

### User Story 4 — Agent Has Full Workspace Context (Priority: P2)

When the agent reads files, lists directories, or searches code, it should see the same workspace that VS Code has open. The file tree, open editors, workspace root — all of this should be available to the agent without extra configuration.

**Why this priority**: An agent without workspace context is blind. It needs to see the project structure to be useful.

**Independent Test**: Open a project with multiple files in VS Code. Ask the agent "list all the files in this project." Confirm the agent can see and enumerate the workspace contents. Ask "read the main entry point" and confirm it reads the correct file.

**Acceptance Scenarios**:

1. **Given** VS Code has a workspace open, **When** the agent calls the file read tool, **Then** it can read any file within the workspace.
2. **Given** the user has specific files open in tabs, **When** the agent queries open editors, **Then** it can see which files the user is currently viewing.
3. **Given** the workspace root changes, **When** the agent starts a new session, **Then** it operates within the correct workspace directory.

---

### User Story 5 — Login and Provider Switching from VS Code (Priority: P3)

The user can log in to their jcode providers (Claude, OpenAI, Gemini, etc.) directly from the VS Code extension without opening a terminal. The settings UI shows configured providers, lets the user add new ones, and shows token usage and remaining credits.

**Why this priority**: This removes friction, but the user can still log in via terminal as a workaround.

**Independent Test**: Open VS Code settings for jcode. Click "Add Provider." Complete the OAuth flow. Verify the provider appears in the configured list and the agent can use it.

**Acceptance Scenarios**:

1. **Given** the jcode settings view is open, **When** the user clicks "Add Provider" and selects Claude, **Then** the OAuth flow opens in the browser and completes.
2. **Given** a provider is configured, **When** the user views provider details, **Then** they see the model, token usage, and status.

---

### Edge Cases

- **What happens when jcode server is not running?** The extension should auto-spawn `jcode serve` as a background process (same as the TUI client does on first launch) and connect to it.
- **What happens when the server reloads?** The client should auto-reconnect with exponential backoff, the same way the TUI client handles `/reload`.
- **How does the extension handle multiple VS Code windows?** Each window is a separate client connecting to the same server, sharing sessions — like multiple terminal windows today.
- **What happens when the user closes VS Code while a session is running?** Sessions persist on the server (idle timeout). Reopening VS Code resumes them.
- **How does the extension behave offline?** If the jcode server is running but has no provider connection, the extension shows an error state. If the server itself is not running, the extension prompts to start it.
- **How do swarm commands work from the VS Code client?** Swarm commands (spawn, DM, broadcast, await-members) should be exposed as VS Code commands or slash commands in the chat panel, same as the TUI.

## Requirements

### Functional Requirements

#### Socket Connection
- **FR-001**: The extension MUST connect to the jcode server over the Unix socket at the standard path (`/run/user/$UID/jcode.sock` on Linux, `~/jcode.sock` on macOS/Windows).
- **FR-002**: The extension MUST speak the newline-delimited JSON protocol defined in `jcode-protocol`.
- **FR-003**: The extension MUST auto-spawn `jcode serve` as a background child process if no server is running on the expected socket path.
- **FR-004**: The extension MUST implement reconnect logic with exponential backoff (1s, 2s, 4s ... up to 30s) when the connection drops.

#### Chat Panel
- **FR-005**: The extension MUST render agent responses in a chat panel using a VS Code webview or TreeView, supporting streaming text, code blocks, and tool call/response display.
- **FR-006**: The chat panel MUST support markdown rendering (including code blocks with syntax highlighting, lists, and inline formatting).
- **FR-007**: The chat panel MUST show tool calls and their results in a collapsible, visually distinct format (not interleaved as raw text).
- **FR-008**: The user MUST be able to submit prompts via the chat input and cancel an in-progress agent turn.
- **FR-009**: The extension MUST support text input, multi-line input, and file attachment (drag-and-drop a file to include its content/path in the prompt).

#### File Edits
- **FR-010**: When the agent uses the edit tool to modify a file, the extension MUST capture the original content (before edit) and show the diff to the user.
- **FR-011**: The extension MUST provide "accept" and "reject" controls for each edit, either inline in the chat or via VS Code's native diff editor.
- **FR-012**: When the user accepts an edit, the extension MUST apply the change to the file in the VS Code workspace. When they reject, the file MUST remain unchanged.
- **FR-013**: If the user has the file open in an editor tab, the extension MUST update the displayed content to reflect accepted edits.

#### Session Management
- **FR-014**: The extension MUST provide a sidebar view listing all active jcode sessions.
- **FR-015**: Each session entry MUST display its name, status icon (running/idle/error), and a preview of the most recent message.
- **FR-016**: The user MUST be able to create a new session, switch between sessions, rename sessions, and close sessions from the sidebar.
- **FR-017**: Sessions MUST persist on the server across VS Code restarts and be listed in the sidebar upon reconnection.

#### Workspace Integration
- **FR-018**: The extension MUST pass the VS Code workspace root as the working directory when creating or resuming a session.
- **FR-019**: When the agent uses file system tools (read, write, edit, grep, glob), the extension MUST ensure those operations happen within the VS Code workspace directory.
- **FR-020**: The extension SHOULD expose the list of currently open editor tabs to the agent when relevant (as ambient context).

#### Provider & Settings
- **FR-021**: The extension MUST allow the user to configure jcode providers through VS Code settings UI.
- **FR-022**: The extension MUST display provider status (connected, disconnected, token usage) in the settings view.
- **FR-023**: The extension SHOULD support initiating provider login flows (OAuth) from within VS Code.

### Key Entities

- **Socket Connection**: A persistent connection to the jcode server Unix socket, sending newline-delimited JSON requests and receiving streaming JSON events.
- **Chat Panel**: A VS Code webview panel or sidebar view that renders the conversation history with streaming support, markdown rendering, and tool call display.
- **Session**: A server-side conversation runtime. Identified by a session ID. Can be in running, idle, or error state. Has an associated conversation history and provider state.
- **Edit Proposal**: A file change suggested by the agent, consisting of the original file content, the new file content, and a diff. Presented to the user for accept/reject.
- **Workspace Context**: Information about the VS Code workspace (root path, open files, file tree) that is provided to the agent for tool execution.

## Success Criteria

### Measurable Outcomes

- **SC-001**: User can install the extension, open a workspace, and send a prompt to the agent within 30 seconds of installation (assuming jcode is already installed).
- **SC-002**: Agent file edits appear as diffs in VS Code within 500ms of the edit tool call completing.
- **SC-003**: User can manage 5+ concurrent sessions from the sidebar without noticeable performance degradation.
- **SC-004**: Reconnection after server reload completes within 10 seconds.
- **SC-005**: All existing jcode capabilities (memory, swarm, 30+ tools, browser automation, MCP) work identically from the VS Code client.
- **SC-006**: The extension does not increase VS Code startup time by more than 1 second when the jcode server is already running.

## Assumptions

- The user has jcode installed and available on their PATH (or the extension bundles/installs it).
- The jcode server binary is compatible with the extension's protocol version.
- VS Code's extension API (webviews, TreeView, TextDocumentContentProvider) is sufficient to implement the chat UI, session list, and diff visualization.
- The extension is initially developed for desktop VS Code (not vscode.dev or GitHub Codespaces), though remote SSH support may follow.
- The existing jcode protocol does not need modification — the extension adapts as a client, same as the TUI.
- Provider login flows that require browser interaction will open the user's default browser from within VS Code.
- Security: The Unix socket is already protected by filesystem permissions (same user). No additional auth layer between the extension and the jcode server is needed.
