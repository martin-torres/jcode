# Jcode Constitution

## What — The Purpose

Jcode is a next-generation coding agent harness — a blazing-fast, TUI-first
tool that lets you run AI coding agents interactively in the terminal.

It replaces tools like Claude Code, Codex CLI, GitHub Copilot CLI, Cursor Agent,
OpenCode, and Pi by being dramatically faster, more memory-efficient, and more
capable across every dimension:

- **10–245× faster** boot and rendering than competitors
- **2–20× less RAM** per session, scaling efficiently to 10+ concurrent sessions
- **Multi-provider** — Claude, OpenAI, Gemini, Copilot, Azure, OpenRouter,
  OpenCode, Ollama, LM Studio, and 20+ more — all in one harness
- **Swarm coordination** — spawn multiple agents in the same repo with
  automatic conflict notifications and messaging
- **Agent memory** — semantic vector embeddings of every turn, automatic
  recall, consolidation, and drift detection
- **Self-development** — the agent can modify, build, test, and reload its own
  source code autonomously
- **30+ built-in tools** — browser automation, file editing, grep, web search,
  Gmail, PDF, MCP, session search, and more
- **Session resume** across harnesses — pick up where Claude Code, Codex,
  OpenCode, or Pi left off
- **Cross-platform** — Linux, macOS, Windows, with an iOS client coming

## Why — The Motivation

Existing coding agent harnesses share a set of problems that jcode exists to
solve:

1. **They're slow.** Claude Code takes 3.4 seconds to render its first frame.
   Jcode does it in 14 milliseconds. This matters when you're iterating fast.

2. **They're memory hogs.** Competitors use 5–20× more RAM per session.
   With 10 concurrent sessions, Claude Code uses 2.3 GB; jcode uses 260 MB.
   This makes multi-session workflows impractical on normal hardware.

3. **They lock you into one provider.** Most harnesses are tied to a single
   model backend. Jcode lets you use any provider, switch mid-session, and
   fall back when one runs out of tokens.

4. **They can't collaborate.** No other harness lets multiple agents work in
   the same repo simultaneously with automatic conflict awareness and
   inter-agent messaging.

5. **They can't improve themselves.** Jcode's self-dev mode means the tool
   evolves with you — the agent you're talking to can edit its own engine,
   rebuild, and reload without losing context.

6. **They waste your context.** Competitors load all skills on startup or use
   naive RAG. Jcode embeds every turn as a semantic vector and retrieves
   only relevant skills and memories, automatically.

The goal: raise the skill ceiling for what coding agents can do, remove every
bottleneck (speed, memory, provider lock-in, isolation, self-improvement), and
make multi-session, multi-agent workflows the default — not an edge case.

## The 50 Crates — What They Are

Jcode is composed of **50 workspace crates** (+ root crate), each with a single
responsibility. Together they form the complete harness:

### Core Engine (4)
- `jcode-core` — Core engine and orchestration
- `jcode-agent-runtime` — Agent execution loop
- `jcode-protocol` — Protocol layer (tool calling, I/O)
- `jcode-build-support` — Build scripts and support

### LLM Provider Layer (5)
- `jcode-provider-core` — Abstract provider interface
- `jcode-provider-openai` — OpenAI / GPT adapter
- `jcode-provider-gemini` — Google Gemini adapter
- `jcode-provider-openrouter` — OpenRouter multi-model gateway
- `jcode-provider-metadata` — Provider registry and metadata

### Tool System (2)
- `jcode-tool-core` — Tool execution engine
- `jcode-tool-types` — Tool definitions and schemas

### Memory & Data (4)
- `jcode-memory-types` — Memory abstraction types
- `jcode-compaction-core` — Context compaction (token optimization)
- `jcode-overnight-core` — Background/overnight processing
- `jcode-embedding` — Embedding vector support

### Types Layer (13)
One types crate per domain, ensuring zero type pollution across boundaries:
- `jcode-session-types`, `jcode-message-types`, `jcode-task-types`
- `jcode-batch-types`, `jcode-background-types`, `jcode-usage-types`
- `jcode-gateway-types`, `jcode-side-panel-types`, `jcode-ambient-types`
- `jcode-auth-types`, `jcode-selfdev-types`, `jcode-config-types`
- `jcode-storage`

### Swarm (1)
- `jcode-swarm-core` — Multi-agent swarm coordination

### Import & Integrations (3)
- `jcode-import-core` — Code/project import engine
- `jcode-notify-email` — Email notification
- `jcode-azure-auth` — Azure auth provider

### TUI System (11)
- `jcode-tui-core` — TUI lifecycle and event loop
- `jcode-tui-render` — Rendering pipeline
- `jcode-tui-style` — Styling and theming
- `jcode-tui-markdown` — Markdown rendering
- `jcode-tui-mermaid` — Mermaid diagram renderer
- `jcode-tui-messages` — Chat message display
- `jcode-tui-tool-display` — Tool call/response display
- `jcode-tui-session-picker` — Session switcher UI
- `jcode-tui-account-picker` — Account switcher UI
- `jcode-tui-usage-overlay` — Usage stats overlay
- `jcode-tui-workspace` — Workspace/file tree UI

### Platform Binaries (3)
- `jcode-desktop` — Desktop app binary
- `jcode-mobile-core` — Mobile platform library
- `jcode-mobile-sim` — Mobile simulator

### Utilities (4)
- `jcode-update-core` — Self-update mechanism
- `jcode-terminal-launch` — Terminal spawning
- `jcode-pdf` — PDF handling
- `jcode-plan` — Planning engine

## Governance

- This constitution defines the **what** and **why** of jcode — its purpose,
  motivation, and scope. It is the north star for all development decisions.
- Any feature, refactor, or change must be justifiable against this document.
  If it does not serve the purpose stated here, it does not belong.
- Amendments require a documented rationale and version bump.
- **Version**: 1.0.0 | **Ratified**: 2026-05-16 | **Last Amended**: 2026-05-16
