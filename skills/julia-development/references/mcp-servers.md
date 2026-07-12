# Julia Execution Tool Reference

Detailed reference for integrating Julia development with `repld`, MCP servers (julia-mcp, Kaimon.jl), and the JETLS language server.

---

## Overview

| | repld | julia-mcp | Kaimon.jl | JETLS |
|---|-------|-----------|-----------|-------|
| **Repo/tooling** | Local `repld` skill/runtime | [aplavin/julia-mcp](https://github.com/aplavin/julia-mcp) | [kahliburke/Kaimon.jl](https://github.com/kahliburke/Kaimon.jl) | [aviatesk/JETLS.jl](https://github.com/aviatesk/JETLS.jl) |
| **Type** | Persistent process runner | MCP server | MCP server + Gate | Language Server (LSP) |
| **Server language** | Multi-language daemon | Python | Julia | Julia |
| **Session management** | Named or environment-keyed sessions | Fully automatic | User-managed (Gate.serve()) | Editor-managed |
| **Setup** | Agent/runtime supplied; verify with a tiny `repld julia` eval | Clone + register with agent MCP config | `]app add Kaimon` + start server + `Gate.serve()` + register | `Pkg.Apps.add()` → editor LSP config |
| **Revise.jl** | Explicitly load/call in Julia commands | Auto-loaded, auto-revises before each eval | Revise-aware | N/A |
| **Transport** | Local daemon/socket | stdio | stdio + ZMQ (Gate) | stdio / TCP socket |
| **Julia version** | Uses local Julia | Any | 1.12+ | 1.12.2+ (not 1.12.1, not 1.13+) |

---

## repld

### Setup and Verify

`repld` is usually supplied by the agent environment or the separate `repld` skill. Prefer verifying before installing or changing anything:

```bash
command -v repld
repld --fresh --session julia-check julia -e 'println(VERSION)'
```

Use a session name that describes the package or task, e.g. `rible-cr`, `solver-debug`, or `docs-build`.

### Common Commands

```bash
repld --fresh --session mypkg julia --project=. -e 'using Revise; using MyPkg'
repld --session mypkg julia -e 'using Revise; Revise.revise(); import Pkg; Pkg.test(; coverage=false)'
repld trace --session mypkg
repld interrupt --session mypkg
repld sessions
```

Use `--fresh` after type layout changes, dependency changes, module reorganization, or repeated Revise/world-age symptoms. Reuse the warm session for ordinary function-body edits and test tweaks.

---

## julia-mcp

### Setup

```bash
# Clone anywhere
git clone https://github.com/aplavin/julia-mcp.git /path/to/julia-mcp

# Server command (register this with your agent's MCP config):
uv run --directory /path/to/julia-mcp python server.py
```

Register with your agent:

| Agent | How |
|-------|-----|
| OpenCode | Add to `.opencode.json` → `mcpServers` or global config |
| Claude Code | `claude mcp add --scope user julia -- uv run --directory /path/to/julia-mcp python server.py` |
| Gemini CLI | Add to `.gemini/settings.json` → `mcpServers` |
| Codex | Add to MCP config per Codex docs |

**Requirements**: `uv` (Python package manager), `julia` binary in PATH.

### Tools

#### `julia_eval`

Execute Julia code in a persistent session.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `code` | string | Yes | — | Julia code to evaluate |
| `env_path` | string \| null | No | `null` | Project directory. `null` = temp session |
| `timeout` | float \| null | No | `60` | Seconds. Auto-disabled for Pkg operations |
| `julia_cmd` | string \| null | No | `null` | Custom Julia command (e.g., `"julia +1.11"`) |

**Behavior**:
- Creates a new Julia process per unique `env_path` (lazy, on first call).
- Subsequent calls reuse the same process (state persists).
- Automatically loads Revise.jl and calls `Revise.revise()` before each evaluation.
- When `env_path` ends in `/test/`, auto-activates TestEnv.

#### `julia_restart`

Restart a Julia session, clearing all state.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `env_path` | string \| null | No | Environment to restart. Omit for temp session |

**When to use**: Major API refactoring, type layout changes, dependency changes. Not needed for small edits (Revise handles those).

#### `julia_list_sessions`

List all active Julia sessions and their environments. No parameters.

### Session Isolation

```
env_path="/project/a"  →  Julia process A
env_path="/project/b"  →  Julia process B
env_path=null          →  Temporary process
```

Each session is fully isolated. Variables, loaded packages, and functions persist only within their session.

### Custom Julia Flags

Default flags: `--startup-file=no --threads=auto`. Override by appending flags to the server command when registering with your agent, e.g.:
```bash
uv run --directory /path/to/julia-mcp python server.py --threads=1 --startup-file=yes
```

---

## Kaimon.jl

### Setup

**1. Install the app** (installs `kaimon` executable to `~/.julia/bin/`):
```julia
# In Julia REPL
]app add Kaimon
```

**2. Start the MCP server** (from terminal):
```bash
kaimon
```

First run opens a setup wizard (security mode, API key, port). After that, the terminal dashboard launches. From the dashboard:
- Press **`i`** in Config tab to auto-write MCP config for Claude Code, Cursor, VS Code, or Gemini CLI
- Press **`g`** to add a Gate snippet to `~/.julia/config/startup.jl` (auto-connect every session)

**3. In your Julia session, open the gate:**
```julia
using Kaimon
Gate.serve()
```

**4. Register with your agent** (if not using dashboard auto-config):

Server command: `kaimon`

| Agent | How |
|-------|-----|
| OpenCode | Add to `.opencode.json` → `mcpServers` or global config |
| Claude Code | `claude mcp add --scope user kaimon -- kaimon` |
| Gemini CLI | Add to `.gemini/settings.json` → `mcpServers` |
| Codex | Add to MCP config per Codex docs |

**Requires**: Julia 1.12+, ZMQ.

### The Gate Pattern

The Gate is a ZMQ bridge connecting your live Julia session to Kaimon's MCP server:

```
┌─────────────────┐    ZMQ     ┌─────────────────┐
│  Julia Session   │◄────────►│  Kaimon Server   │
│  (Gate.serve())  │   socket  │  (MCP stdio)     │
└─────────────────┘           └─────────────────┘
                                        │
                                        ▼
                                ┌─────────────────┐
                                │  AI Agent        │
                                │  (Claude Code)   │
                                └─────────────────┘
```

The Julia session is **yours** — Kaimon connects to it via the Gate. The AI agent sees your variables, your loaded packages, and your state.

### Tool Categories (32+ tools)

#### Code Execution

| Tool | Purpose |
|------|---------|
| `kaimon_ex(e="code", q=true)` | Execute Julia code. `q=true` suppresses output. |
| `kaimon_manage_repl(action="...")` | Manage REPL state |

#### Introspection

| Tool | Purpose |
|------|---------|
| `kaimon_investigate_environment()` | Inspect environment state |
| `kaimon_search_methods(query="...")` | Search for methods |
| `kaimon_type_info(name="...")` | Get type information |
| `kaimon_list_names(module="...")` | List names in scope |
| `kaimon_workspace_symbols()` | List workspace symbols |
| `kaimon_document_symbols()` | List document symbols |
| `kaimon_macro_expand(code="...")` | Expand macros |

#### Code Analysis

| Tool | Purpose |
|------|---------|
| `kaimon_code_lowered(code="...")` | Get lowered IR |
| `kaimon_code_typed(code="...")` | Get typed IR |
| `kaimon_format_code(code="...")` | Format Julia code |
| `kaimon_lint_package()` | Lint package code |

#### Navigation

| Tool | Purpose |
|------|---------|
| `kaimon_goto_definition(name="...")` | Jump to definition |
| `kaimon_navigate_to_file(path="...")` | Navigate to file |

#### VS Code Integration

| Tool | Purpose |
|------|---------|
| `kaimon_execute_vscode_command(cmd="...")` | Execute VS Code command |
| `kaimon_list_vscode_commands()` | List available VS Code commands |

#### Debugging (Infiltrator.jl)

| Tool | Purpose |
|------|---------|
| `kaimon_debug_ctrl(action="status")` | Control debugger (`status`, `continue`, `step`, `finish`) |
| `kaimon_debug_eval(code="...")` | Evaluate in debug context |
| `kaimon_debug_exfiltrate(expr="...")` | Extract values from debug context |
| `kaimon_debug_inspect_safehouse()` | Inspect stored debug values |
| `kaimon_debug_clear_safehouse()` | Clear stored debug values |

#### Package & Testing

| Tool | Purpose |
|------|---------|
| `kaimon_pkg_add(packages=["Name"])` | Add packages |
| `kaimon_pkg_rm(packages=["Name"])` | Remove packages |
| `kaimon_run_tests(pattern="...")` | Run tests matching pattern |
| `kaimon_profile_code(code="...")` | Profile code execution |
| `kaimon_stress_test(code="...")` | Stress test code |

#### Semantic Code Search (requires Qdrant)

| Tool | Purpose |
|------|---------|
| `kaimon_qdrant_search_code(query="...")` | Semantic code search |
| `kaimon_qdrant_index_project()` | Index project for search |
| `kaimon_qdrant_sync_index()` | Sync search index |
| `kaimon_qdrant_list_collections()` | List search collections |
| `kaimon_qdrant_collection_info(name="...")` | Get collection info |
| `kaimon_qdrant_browse_collection(name="...")` | Browse collection |
| `kaimon_qdrant_reindex_file(path="...")` | Reindex specific file |

#### Info & Server

| Tool | Purpose |
|------|---------|
| `kaimon_ping()` | Server health check |
| `kaimon_usage_instructions()` | Get usage instructions |
| `kaimon_usage_quiz()` | Usage quiz |
| `kaimon_tool_help(tool="...")` | Get help for specific tool |

### Custom Tool Registration

Register your own functions as MCP tools via GateTool:

```julia
using Kaimon.Gate: GateTool, serve

"""Analyze data with a configurable threshold."""
function analyze_data(path::String, threshold::Float64=0.95)
    data = load(path)
    filter(x -> x.score > threshold, data)
end

serve(tools=[GateTool(analyze_data)])
```

The AI agent can then call `kaimon_analyze_data` with `path` and `threshold` parameters.

### Security Modes

| Mode | Description |
|------|-------------|
| Strict | Highest security, limited operations |
| Relaxed | Balanced security and functionality |
| Lax | Full access to all capabilities |

---

## Using repld, MCP, and JETLS Together

`repld`, julia-mcp, Kaimon, and JETLS are complementary. Prefer `repld` for repeated package execution and tests. Use MCP tools when they are available and add value for introspection or fallback execution.

### Pattern 1: repld for Dev, Kaimon for Introspection, JETLS for Analysis

Use `repld` for the main warm development session. Add Kaimon when live introspection or debugging is useful. Use JETLS for inline diagnostics and navigation in your editor.

```
# Main warm execution session
repld --session mypkg julia --project=. -e 'using Revise; Revise.revise(); using MyPackage; run_analysis()'

# Optional live introspection when Kaimon tools are available
kaimon_ex(e="MyPackage.some_loaded_state", q=false)

# Optional fallback experiment via julia-mcp
julia_eval(code="using Plots; plot(sin, -π, π)")

# JETLS: inline in editor — diagnostics, hover, go-to-def, references
# (no MCP call needed, configured via editor LSP)
```

### Pattern 2: Open Gate via julia-mcp

If only julia-mcp is configured but the user wants Kaimon-style introspection:

```
# Start a Julia session and open the gate
julia_eval(code="using Kaimon; Gate.serve()", env_path="/my/project")
# Now Kaimon tools are available on that session
```

### Pattern 3: Investigation Workflow

```
1. repld --session mypkg julia -e 'using Revise; Revise.revise(); include("test/runtests.jl")' → Reproduce failure
2. kaimon_investigate_environment()   → Understand current state when Kaimon is available
3. kaimon_type_info(name="MyType")    → Inspect types
4. kaimon_search_methods(query="process") → Find relevant methods
5. kaimon_code_typed(code="process(x)")   → Check for type instability
6. kaimon_debug_ctrl(action="continue")   → Step through if needed
```

---

## JETLS (Julia Language Server)

JETLS is a Julia language server using JET.jl for experimental static-analysis-backed diagnostics. It is **not** an MCP server; it uses the LSP protocol and integrates via an editor's LSP client.

### Setup

**1. Install the `jetls` executable:**

```bash
# Requires Julia 1.12.2+ (not 1.12.1 or earlier, not 1.13+/nightly)
julia -e 'using Pkg; Pkg.Apps.add(; url="https://github.com/aviatesk/JETLS.jl", rev="release")'
```

This installs `jetls` to `~/.julia/bin/`. Ensure `~/.julia/bin` is on `PATH`.

**Verify**: `jetls --help` should display help text.

**2. Configure your editor:**

| Editor | Setup |
|--------|-------|
| VS Code | Install `jetls-client` extension from marketplace |
| Neovim (0.11+) | `vim.lsp.config("jetls", { cmd = {"jetls", "serve"}, filetypes = {"julia"} })` then `vim.lsp.enable("jetls")` |
| Emacs (eglot) | Add `("jetls" "serve" "--socket" :autoport)` to `eglot-server-programs` |
| Helix | `languages.toml`: `jetls = { command = "jetls", args = ["serve"] }` |
| Sublime | `LSP.sublime-settings`: `"command": ["jetls", "serve", "--socket=${port}"]` |
| Zed | See [aviatesk/zed-julia#avi/JETLS](https://github.com/aviatesk/zed-julia/tree/avi/JETLS) |

### Features

| Feature | Description |
|---------|-------------|
| Diagnostics | Type-sensitive error detection via JET.jl |
| Completion | Context-aware code completion |
| Hover | Type information and docs on hover |
| Go-to-definition | Macro-aware definition navigation |
| Find references | Find all references across project |
| Rename | Symbol renaming |
| Code actions | Quick fixes and refactoring |
| Inlay hints | Type annotations displayed inline |
| Semantic tokens | Semantic highlighting |
| Code lens | Reference counts, test runner integration |
| Formatting | Via JuliaFormatter.jl integration |
| TestRunner | Run tests from editor with inline results |
| Notebook support | Jupyter notebook integration |

### Configuration

JETLS supports configuration via editor settings. See [JETLS configuration docs](https://aviatesk.github.io/JETLS.jl/configuration/) for full reference.

### Update

```bash
# Update to latest
julia -e 'using Pkg; Pkg.Apps.add(; url="https://github.com/aviatesk/JETLS.jl", rev="release")'

# Pin a specific version
julia -e 'using Pkg; Pkg.Apps.add(; url="https://github.com/aviatesk/JETLS.jl", rev="2026-05-08")'
```

**Note**: JETLS is separate from LanguageServer.jl (used by julia-vscode). JETLS provides deeper type analysis via JET.jl. Both can coexist but may provide overlapping completions.

---

## Availability and Version Check Commands

Run these only when a tool is relevant to the task or appears broken. Avoid network checks during ordinary coding unless freshness matters.

| Tool | Check installed | Check latest |
|------|----------------|--------------|
| JETLS | `command -v jetls && jetls --help 2>&1 \| head -5` | Check [releases](https://github.com/aviatesk/JETLS.jl/releases) when updating |
| Kaimon | `command -v kaimon && kaimon --help 2>&1 \| head -5` | Check [releases](https://github.com/kahliburke/Kaimon.jl/releases) when updating |
| Revise.jl | `julia --startup-file=no -e 'println(Base.find_package("Revise") === nothing ? "not found" : "available")'` | Check [releases](https://github.com/timholy/Revise.jl/releases) when updating |
| TestEnv.jl | `julia --startup-file=no -e 'println(Base.find_package("TestEnv") === nothing ? "not found" : "available")'` | Check [releases](https://github.com/JuliaTesting/TestEnv.jl/releases) when updating |
| julia-mcp | `test -d ~/julia-mcp && git -C ~/julia-mcp log -1 --oneline` | `git -C ~/julia-mcp fetch origin && git -C ~/julia-mcp log HEAD..origin/master --oneline` when updating |
| Julia | `julia --version` | [Downloads](https://julialang.org/downloads) |

### Staleness Heuristics

| Tool | Threshold | Notes |
|------|-----------|-------|
| JETLS | Troublesome diagnostics or known fixed issue | JETLS moves quickly; do not interrupt normal work solely because a date string is old |
| Kaimon | Tool missing, incompatible, or known fixed issue | Check releases only when using Kaimon |
| Revise.jl | Revise failures or world-age/stale-method symptoms | Update only when relevant |
| TestEnv.jl | Test environment activation failures | Update only when relevant |
| julia-mcp | Server startup/tool failures or known fixed issue | Check commits only when using julia-mcp |
| Julia | Project requires a newer compatible Julia | Do not upgrade Julia just because latest stable exists |

### Out-of-Date Prompt Template

If a required tool is missing or a relevant optional tool appears outdated, prompt the user **once per session** using this template:

> "The following Julia tools have updates available:
> - **JETLS**: installed YYYY-MM-DD, latest is YYYY-MM-DD → update command
> - **Kaimon**: installed vX.Y.Z, latest is vA.B.C → update command
> - **Revise.jl**: installed vX.Y.Z, latest is vA.B.C → update command
> - **TestEnv.jl**: installed vX.Y.Z, latest is vA.B.C → update command
> - **julia-mcp**: N commits behind → update command
>
> Update now?"

**Rules**:
- Check versions only when relevant to the current task or failure mode, prompt **once per session only**.
- If user declines, don't prompt again.
- If only julia-mcp is available, update Julia packages via:
  ```
  julia_eval(code='using Pkg; Pkg.update(["Revise", "TestEnv", "Kaimon"])')
  ```

### Update Commands (Quick Reference)

| Tool | Install | Update |
|------|---------|--------|
| julia-mcp | `git clone https://github.com/aplavin/julia-mcp.git ~/julia-mcp` | `cd ~/julia-mcp && git pull` |
| Kaimon | `]app add Kaimon` (Julia REPL) | `]app add Kaimon` (re-run) |
| JETLS | `julia -e 'using Pkg; Pkg.Apps.add(; url="https://github.com/aviatesk/JETLS.jl", rev="release")'` | Re-run install command |
| Revise.jl | `]add Revise` (global env) | `]up Revise` (global env) |
| TestEnv.jl | `]add TestEnv` (global env) | `]up TestEnv` (global env) |

All Julia-based tools (`kaimon`, `jetls`) install executables to `~/.julia/bin/`. Ensure this directory is on your `PATH`.

### Updating via MCP (when only julia-mcp is available)

```julia_eval
julia_eval(code='using Pkg; Pkg.update(["Revise", "TestEnv", "Kaimon"])')
```

---

## Revise.jl Integration

### julia-mcp Behavior

- Revise.jl is loaded automatically at session start (`try; using Revise; catch; end`).
- `Revise.revise()` is called before every code evaluation.
- **No manual action needed** — file changes are picked up automatically.

### Kaimon Behavior

- Kaimon is Revise-aware — revised code is reflected immediately.
- Works with the user's existing Revise setup.

### When Revise is Not Enough

Revise handles method body changes well. It **cannot** handle:

| Change Type | Revise handles? | Action needed |
|-------------|-----------------|---------------|
| Function body | Yes | None |
| New method | Yes | None |
| Docstring update | Yes | None |
| Changed function signature | Partially | Restart recommended |
| New exported symbol | No | Restart |
| Struct field changes | No | Restart |
| Module reorganization | No | Restart |
| Dependency changes | No | Restart |
| World age edge cases | Sometimes | Restart if issues |

**Rule of thumb**: For a single function edit, trust Revise. For refactoring touching multiple files or changing APIs, restart proactively.
