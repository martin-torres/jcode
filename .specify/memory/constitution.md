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

## General Governance

- This constitution defines the **what** and **why** of jcode — its purpose,
  motivation, and scope. It is the north star for all development decisions.
- Any feature, refactor, or change must be justifiable against this document.
  If it does not serve the purpose stated here, it does not belong.
- Amendments require a documented rationale and version bump.
- **Version**: 1.0.0 | **Ratified**: 2026-05-16 | **Last Amended**: 2026-05-16

---

## Feature Constitution: Top-Down TUI Layout Flip

### I. Full Architectural Inversion — Not a Layout Swap
The input box moves to the TOP of the screen. Messages flow downward from the input — newest content sits at the top (line index 0), oldest at the bottom. This reverses the content ordering, scroll anchoring, auto-scroll logic, prompt navigation direction, and absolute line index systems. Every subsystem that references "newest = bottom, oldest = top" must be inverted.

### II. Keep Rendering Order Intact, Only Reorder Layout + Sections
The message rendering functions (how individual messages are drawn, how streaming text fills, how batch progress is displayed) remain unchanged. Only the **constraint order** in the `Layout` definition and the **section assembly order** in `prepare_messages_inner` change. The rendering primitives themselves do not need new logic.

### III. Scroll Semantics Must Be Intuitive
After inversion, `scroll = 0` shows the newest content (just below the input). `scroll = max_scroll` shows the oldest content. Scrolling UP must reveal chronologically OLDER content (content further from the input). If the existing `scroll_up`/`scroll_down` functions move in the wrong direction for this, their direction semantics must be inverted — not just the anchor point.

### IV. Auto-Scroll Anchors at Top (scroll = 0)
Auto-follow targets `scroll = 0` instead of `scroll = max_scroll`. When the user manually scrolls away from `scroll = 0`, auto-scroll pauses. When they scroll back to `scroll = 0`, it resumes. No changes to the pause/resume mechanism itself — only the anchor value.

### V. All Absolute Index Systems Recalculate
Any feature that depends on absolute line indices must be recalculated: copy targets, image regions, edit tool ranges, user prompt positions, prompt-jump keyboard shortcuts. These use the `from_sections` helper — pass the reversed section order and verify all offsets are correct.

### VI. No Changes Outside src/tui/ui.rs
The provider layer, session management, config system, input rendering, message rendering functions, scrollbar drawing, and `src/tui/viewport.rs` all stay untouched. The change is scoped to `draw_inner()` in `ui.rs` — specifically the layout constraints, section order, index references, and scroll anchor.

### VII. Test-First Validation
After the flip, verify against the spec's acceptance scenarios before considering the task done. Launch the app, type a prompt, confirm input is at top, confirm content appears just below input, confirm scroll direction is intuitive, confirm auto-scroll works during streaming, confirm prompt navigation jumps to correct positions.

## Feature Scope Boundaries

### In Scope
- Layout constraint order in `draw_inner()` — move input constraint to position 0
- Section assembly order in `prepare_messages_inner` — reverse `[Streaming, BatchProgress, Body, Header]`
- Auto-scroll anchor — change from `max_scroll` to `0`
- Scroll direction semantics — invert if `scroll_up` points toward newer content
- All `chunks[N]` index references — renumber for shifted positions
- `capture.layout.input_area` — update to `chunks[0]`
- `record_layout_snapshot` — update to `chunks[0]`
- FR-014 resolution — decide whether to invert scroll functions or accept counterintuitive mapping

### Out of Scope
- Message rendering functions (text formatting, streaming display, batch progress visuals)
- Input rendering in `src/tui/ui_input.rs`
- Scrollbar logic in `src/tui/viewport.rs`
- Provider layer, session management, config system
- Side panels, pinned diagrams, image rendering
- Git branch management or CI/CD changes

## Feature Development Workflow

1. Switch to existing `input-top-layout` branch or create new branch from current `main`
2. Make all layout/scroll changes in `src/tui/ui.rs` following the spec
3. Build with `cargo build` after every change block
4. Test against spec's acceptance scenarios (User Stories 1-4) by running JCode locally
5. Fix any regressions in copy, selection, prompt navigation immediately
6. Commit with clear messages per change block

## Feature Governance

- The spec (specs/001-top-down-tui-layout-flip/spec.md) is the authoritative reference for all decisions
- This feature constitution supersedes any general coding conventions that conflict with the inversion requirements
- Changes outside the in-scope boundaries require documentation and re-approval
- After implementation, run the full test suite (`cargo test`) and verify no existing functionality regressed
- The `input-top-layout` branch may contain partial work from prior attempts — inspect before assuming it's the correct starting point

**Feature Version**: 1.0.0 | **Ratified**: 2026-05-13 | **Last Amended**: 2026-05-13
