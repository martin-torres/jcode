# Implementation Plan: VS Code Extension — jcode Client

**Branch**: `002-vscode-extension` | **Date**: 2026-05-16 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/002-vscode-extension/spec.md`

## Summary

Build a VS Code extension that connects to the jcode server over its existing
Unix socket protocol (newline-delimited JSON), providing a native VS Code UI
for interacting with jcode agents — chat panel, session management sidebar,
inline file diffs with accept/reject, and full workspace integration.

The extension speaks the exact same protocol as the TUI client — no server
changes required.

## Technical Context

**Language/Version**: TypeScript 5.x, targeting VS Code 1.96+
**Primary Dependencies**: VS Code Extension API (webviews, TreeView, TextDocumentContentProvider)
**Storage**: Session state is server-side — extension is stateless beyond local UI state
**Testing**: VS Code extension test runner + simulated socket server for integration tests
**Target Platform**: VS Code Desktop (macOS, Linux, Windows); remote SSH support as follow-up
**Project Type**: VS Code extension (strictly a client, no server-side code)
**Performance Goals**: Sub-500ms from tool call completion to diff display; sub-1s extension activation
**Constraints**: Must not block VS Code startup; must auto-reconnect on server restart
**Scale/Scope**: Single extension with 3 main UI surfaces (chat webview, session TreeView, settings)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Alignment with jcode's purpose** (what):
- ✓ Extends jcode to VS Code users, the most popular code editor
- ✓ Preserves all existing jcode capabilities (memory, swarm, 30+ tools, multi-provider)
- ✓ Adds inline diff visualization — a capability the TUI cannot provide natively
- ✓ Enables multi-session workflows from within VS Code

**Alignment with jcode's motivation** (why):
- ✓ Solves "locked into one provider" — jcode remains the engine
- ✓ Solves "can't collaborate" — swarm works the same as in TUI
- ✓ Does not add complexity to the jcode server (zero server changes)
- ✓ Does not degrade TUI experience

**GATE status**: PASS. This feature extends jcode's reach without compromising its core.

## Architecture

### Communication Model

```
┌─────────────────────────────────────────────────────┐
│                    VS Code                           │
│  ┌──────────────────────────────────────────────┐   │
│  │         jcode Extension                      │   │
│  │                                              │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐   │   │
│  │  │Session   │  │ Chat     │  │Settings  │   │   │
│  │  │TreeView  │  │Webview   │  │Provider  │   │   │
│  │  └──────────┘  └──────────┘  └──────────┘   │   │
│  │        │            │              │         │   │
│  │  ┌─────┴────────────┴──────────────┴──────┐  │   │
│  │  │        SocketClient (singleton)         │  │   │
│  │  │   newline-delimited JSON over Unix sock │  │   │
│  │  └────────────────┬────────────────────────┘  │   │
│  └───────────────────┼────────────────────────────┘   │
└──────────────────────┼─────────────────────────────────┘
                       │
              Unix socket (e.g. /run/user/$UID/jcode.sock)
                       │
┌──────────────────────┼─────────────────────────────────┐
│              jcode serve (existing server)             │
│              ──────────────────────────────            │
│         No changes needed. Speaks same protocol.       │
└────────────────────────────────────────────────────────┘
```

### Protocol Wire Format

The extension sends/receives **newline-delimited JSON** over a Unix socket.

**Key request types** the extension will use:

| Request | When |
|---------|------|
| `Subscribe { working_dir }` | Connect to server, create/resume session |
| `GetHistory` | Get full conversation history on connect |
| `Message { content, images }` | Send user prompt |
| `Cancel` | Stop current generation |
| `SoftInterrupt { content }` | Interject mid-turn without canceling |
| `ResumeSession { session_id }` | Switch to a different session |
| `RenameSession { title }` | Rename a session from sidebar |
| `Clear` | Clear conversation |
| `Ping` | Health check / keepalive |

**Key event types** the extension must handle:

| Event | UI Action |
|-------|-----------|
| `TextDelta { text }` | Append streaming text to chat |
| `ToolStart { id, name }` | Show tool call in chat (loading state) |
| `ToolExec { id, name }` | Show tool as executing |
| `ToolDone { id, name, output, error }` | Show tool result; if edit tool, create diff |
| `Done { id }` | Mark turn complete |
| `History { messages, ... }` | Render full conversation (on connect/resume) |
| `SessionId { session_id }` | Store assigned session ID |
| `SessionRenamed { session_id, title }` | Update sidebar entry |
| `SwarmStatus { members }` | Update swarm indicators |
| `Notification { from_session, message }` | Show VS Code notification |
| `Error { id, message }` | Show error in chat |
| `Reloading` | Start reconnect loop |

### Edit Tool Interception (Diff Flow)

The key insight: the extension watches for `ToolStart` with `name: "edit"`
(or similar file-modifying tool), snapshots the file's current content before
the edit runs, and when `ToolDone` arrives with the output, it presents the
diff to the user.

```
User asks agent to edit file → Agent calls edit tool
                              → Extension sees ToolStart(name="edit")
                              → Extension snapshots current file content
                              → Tool completes (ToolDone)
                              → Extension compares snapshot vs new content
                              → Presents diff in VS Code with accept/reject
                              → User accepts → file is updated
                              → User rejects → file stays at snapshot
```

### Socket Path Discovery

The extension finds the jcode server socket through the registry:

1. Read `~/.jcode/servers.json` to find the `socket` path of the active server
2. If no server, spawn `jcode serve` as a background process and wait for socket
3. Connect and begin subscribe flow

## Project Structure

```text
vscode-jcode/
├── package.json                 # Extension manifest
├── tsconfig.json
├── src/
│   ├── extension.ts             # Activation entry point
│   ├── socketClient.ts          # Unix socket JSON protocol client
│   ├── protocol.ts              # TypeScript types mirroring Request/ServerEvent
│   ├── sessionManager.ts        # Session lifecycle management
│   ├── serverManager.ts         # Server process lifecycle (spawn, reconnect)
│   ├── chatProvider.ts          # WebviewPanel provider for chat UI
│   ├── chatHtml.ts              # HTML/CSS/JS for the chat webview
│   ├── sessionTreeProvider.ts   # TreeDataProvider for session sidebar
│   ├── editTracker.ts           # Captures file state before edits, presents diffs
│   ├── diffCommands.ts          # Accept/reject commands for file diffs
│   ├── settingsProvider.ts      # VS Code settings UI (providers, accounts)
│   └── util/
│       ├── socket.ts            # Raw Unix socket connection (net.Socket or node-ipc)
│       └── jsonStream.ts        # Newline-delimited JSON line parser
├── test/
│   ├── mockServer.ts            # Simulated jcode server for extension tests
│   ├── socketClient.test.ts
│   └── sessionManager.test.ts
└── resources/
    └── icon.png                 # Extension icon
```

## Phases

### Phase 1: Socket Client + Protocol Types (P1)
**Delivers**: A TypeScript client that connects to jcode, speaks the protocol,
and handles message framing.

Tasks:
- Define `Request` and `ServerEvent` types in TypeScript (mirroring the Rust enums)
- Implement Unix socket connection (using Node.js `net` module on macOS/Linux, named pipe on Windows)
- Implement newline-delimited JSON framing (readline-based stream parser)
- Implement `SocketClient` class with:
  - `connect(socketPath)`
  - `send(request)` with promise-based response tracking (map `id` → resolve/reject)
  - Event emitter for streaming events (`TextDelta`, `ToolStart`, etc.)
  - Reconnect with exponential backoff
- Implement `ServerManager` that:
  - Reads `~/.jcode/servers.json` to discover socket path
  - Auto-spawns `jcode serve` if no server running
  - Waits for socket file to appear

### Phase 2: Chat Webview (P1)
**Delivers**: A VS Code webview panel showing the agent chat with streaming text.

Tasks:
- Implement `ChatProvider` extending `vscode.WebviewViewProvider` (sidebar) or
  `vscode.CustomEditor` (tab)
- Build HTML/CSS chat UI with:
  - Message bubbles (user + assistant)
  - Streaming text insertion (append `TextDelta` chunks)
  - Markdown rendering with syntax highlighting (use `marked` or similar)
  - Collapsible tool call cards (name, status spinner, output)
  - Input box with multi-line support
  - Cancel button during generation
- Wire messages between webview and `SocketClient`
- Handle session history on connect: render `History` event messages

### Phase 3: Session Manager + Sidebar (P2)
**Delivers**: A VS Code TreeView in the activity bar showing all jcode sessions.

Tasks:
- Implement `SessionTreeProvider` extending `vscode.TreeDataProvider`
- Show sessions with:
  - Name and display title
  - Status icon (running/idle/error)
  - Last message preview
  - Session ID (tooltip)
- Support commands:
  - Click to switch (sends `ResumeSession`)
  - "+" button to create new session (sends `Subscribe`)
  - Right-click → Rename (sends `RenameSession`)
  - Right-click → Close (sends `Clear` + disconnect)
- Handle `SwarmStatus` events: show swarm indicators on sessions
- Persist session list locally for fast re-render, reconcile on `GetHistory`

### Phase 4: File Edit Diffs (P1)
**Delivers**: Intercepted file edits shown as VS Code diffs with accept/reject.

Tasks:
- Implement `EditTracker`:
  - Watch for `ToolStart` with `name === "edit"` (or `write`, `apply_patch`)
  - Snapshot file content from VS Code workspace
  - On `ToolDone`, compare snapshot with current file content
  - If changed, create a diff
- Present diff to user:
  - Option A: VS Code's native diff editor (`vscode.commands.executeCommand('vscode.diff')`)
  - Option B: Inline decorations in the chat webview with accept/reject buttons
  - Option C: Both — inline decoration for quick acceptance, diff editor for full view
- Implement accept command: apply the edit to the file
- Implement reject command: restore file to snapshot content
- Handle edge cases: file deleted during edit, file open in unsaved editor tab,
  concurrent edits from multiple turns

### Phase 5: Workspace Integration (P2)
**Delivers**: Full workspace context for the agent.

Tasks:
- Pass `working_dir` as VS Code workspace root in `Subscribe` request
- Expose open editor tabs as ambient context
- Ensure file tool operations (read, write, glob, grep) use correct workspace
- Add "Add file to context" command (right-click file → "Ask jcode about this")
- Support drag-and-drop files into chat input to include file paths/contents

### Phase 6: Settings & Provider Management (P3)
**Delivers**: VS Code settings view for jcode provider configuration.

Tasks:
- Add VS Code settings (`jcode.*`) for provider configuration
- Implement settings webview or use `vscode.SettingsView`
- Show configured providers, models, token usage
- Support initiating OAuth login flows from within VS Code
- Handle `AvailableModelsUpdated` events

### Phase 7: Polish & Edge Cases (P3)
**Delivers**: Production-ready robustness.

Tasks:
- Handle server reload (`Reloading` event → reconnect loop)
- Handle `vscode.window.state.onDidChangeWindowState` (suspend/resume)
- Error notifications for connection loss, auth failures
- Keyboard shortcuts (send with Cmd+Enter, cancel with Escape)
- Session auto-resume on VS Code restart
- Telemetry opt-in (if appropriate)
- Documentation: README, screenshots, quickstart

## Key Design Decisions

### 1. Webview vs Native VS Code UI

Chat rendering is best done in a webview (HTML/JS) for rich markdown + streaming.
The session list is best as a native TreeView for performance and VS Code integration.

**Decision**: Hybrid approach — webview for chat, TreeView for sessions.

### 2. Socket Connection Model

One persistent socket connection to the server multiplexes all requests for
the extension. Session switching is done via protocol messages (`ResumeSession`),
not by opening new connections.

**Decision**: Single `SocketClient` singleton per extension instance.

### 3. Edit Diff Strategy

VS Code's native diff editor (`vscode.diff`) is the best UX for reviewing
complex changes. For quick accept/reject of simple edits, use inline controls
in the chat webview.

**Decision**: Both — inline buttons for simple edits, "Open diff" link for
full comparison. Inline shows a summary; the diff editor shows everything.

### 4. Server Spawning

The extension should behave like the TUI client: if no server, spawn it.
But unlike the TUI, the extension should not block VS Code startup — it
should spawn the server in the background and connect asynchronously.

**Decision**: Background spawn with async readiness check. Show "connecting..."
state in the chat panel until connected.

## Complexity Tracking

No constitution violations. The extension is a pure client addition with
zero server-side changes. The only complexity is the TypeScript re-implementation
of the protocol types, which is straightforward serde JSON with tagged enums.

## Risks & Mitigations

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Protocol changes in jcode break extension | Low | Pin protocol version; use codegen or type-checking tests |
| Unix socket not available (Windows) | Medium | Named pipes already supported by jcode server; use `node-ipc` |
| VS Code webview performance with long conversations | Low | Virtual scrolling in chat webview; paginated history |
| Edit tool snapshot races with concurrent edits | Low | Sequential turn model prevents this; swarm edits are cross-session |
| User has conflicting jcode server versions | Low | Check server version in `History` event's `server_version` field |
