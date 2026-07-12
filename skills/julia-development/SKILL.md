---
name: julia-development
description: This skill should be used when the user asks to "write Julia code",
  "refactor Julia", "explain multiple dispatch", "optimize Julia performance",
  "Julia types", "Julia Pkg management", or "idiomatic Julia". Covers Julian
  style, type system, performance, Pkg, environment detection, repld-first
  execution, optional MCP server integration (Kaimon.jl and julia-mcp), and
  JETLS for static analysis.
  Not for non-Julia tasks. For package docs, load docs-style-preferences.
version: 4.1.3
tags: [Julia, MultipleDispatch, Types, Performance, Environment, Pkg, repld, MCP, JETLS]
---

# Julia Development Skill

Write idiomatic Julia code using Julian programming style (multiple dispatch, composition).

---

## 1. Julian vs OOP Style

Shift from class inheritance to multiple dispatch:

| OOP | Julian |
|-----|--------|
| Methods belong to classes | Functions operate on data |
| Single dispatch (`this`) | Multiple dispatch (all args) |
| Inheritance hierarchies | Composition + abstract types |
| Encapsulation | Modules + naming conventions |

```julia
# Separate data from behavior
abstract type Animal end
struct Dog <: Animal name::String end
struct Cat <: Animal name::String end

# Multiple dispatch — behavior is external
speak(a::Dog) = "$(a.name) says Woof!"
speak(a::Cat) = "$(a.name) says Meow!"
```

---

## 2. Multiple Dispatch

Define behavior based on **all argument types**, not just the first:

```julia
# Generic fallback
collide(a, b) = "generic collision"

# Specific optimizations via type annotations
collide(a::Circle, b::Circle) = "circle-circle"
collide(a::Circle, b::Rectangle) = "circle-rect"
collide(a::Rectangle, b::Circle) = collide(b, a)  # symmetry
```

### Method Ambiguity

When two methods are equally specific, define the intersection:

```julia
# AMBIGUOUS — neither is more specific
f(x::Any, y::String) = ...
f(x::String, y::Any) = ...
f("a", "b")  # Error!

# FIX: Define intersection
f(x::String, y::String) = ...
```

### Parametric Methods

```julia
same_type(x::T, y::T) where {T} = true   # Diagonal dispatch
same_type(x, y) = false
process(x::T) where {T<:Number} = ...     # Type bounds
```

---

## 3. Type System & Performance

→ See `references/language-deep-dive.md` for type annotations, type stability, broadcasting fusion, loop optimization, and `Val` types.

**Key rules**:
- Default to immutable `struct`; use `mutable struct` only when mutation is required
- Keep functions type-stable: return types should be inferable from input types, not from unpredictable runtime values
- Use dotted calls or `@.` for fused broadcasting when it improves clarity and avoids unnecessary temporaries
- Annotate types for dispatch, invariants, or documentation; concrete argument annotations rarely improve performance by themselves

---

## 4. Common Patterns

### Bang Convention

Functions modifying arguments end with `!`:

- `map(f, arr)` → returns new array
- `map!(f, dest, arr)` → modifies `dest`
- `push!(arr, x)` → modifies `arr`

### Duck Typing

```julia
# Works with any type supporting the required operations
magnitude(v) = sqrt(sum(x^2 for x in v))
# Works for: Vector, Tuple, SVector, custom types...
```

### do-block Syntax

```julia
open("file.txt", "w") do io
    write(io, data)
end  # Auto-closed

map(collection) do x
    complex_transform(x)
end
```

### Keyword Arguments

```julia
function process(data; normalize=true, scale=1.0)
    normalize && (data = data ./ maximum(data))
    return data .* scale
end

process([1,2,3])                    # defaults
process([1,2,3]; normalize=false)   # override
```

---

## 5. REPL & Package Management

### REPL Modes

| Mode | Enter | Purpose |
|------|-------|---------|
| `julia>` | Default | Execute code |
| `help?>` | `?` | Documentation |
| `shell>` | `;` | System commands |
| `pkg>` | `]` | Package management |

### Essential Pkg Commands

```julia
pkg> st              # Status
pkg> add Package     # Install
pkg> dev Package     # Track local development version
pkg> rm Package      # Remove
pkg> up              # Update all
pkg> instantiate     # Restore from Manifest.toml
```

→ See `references/testing-and-repl.md` for Pkg API usage, Test.jl, TestEnv, and Revise.jl.

---

## 6. Common Mistakes (OOP → Julia)

→ See `references/oop-migration.md` for detailed examples of OOP→Julia pitfalls and their fixes.

**Quick checklist**:
- Don't put methods inside `struct` — use external functions with type annotations
- Don't over-specify types (`Int64`) — let Julia specialize generically
- Avoid type piracy: do not extend functions you don't own on types you don't own. Extending Base or package functions for your own types is normal Julia style.
- Default to `struct` (immutable) — use `mutable struct` only when needed

---

## 7. Execution Tooling

Prefer `repld` for Julia execution when available. It gives long-lived named sessions, works well with Revise, survives ordinary code edits, and makes test iteration reproducible from shell commands. MCP servers are optional supplements for introspection or environments where `repld` is unavailable.

### repld First Workflow

Use one warm named session per task or package:

```bash
# Start once
repld --fresh --session mypkg julia --project=MyPkg -e 'using Revise; using MyPkg; println("ready")'

# Reuse for checks
repld --session mypkg julia -e 'using Revise; Revise.revise(); import Pkg; Pkg.test(; coverage=false)'
repld --session mypkg julia -e 'using Revise; Revise.revise(); include("test/unit/foo.jl")'

# Diagnose or recover
repld trace --session mypkg
repld interrupt --session mypkg

# Close the task session after an implementation or testing phase
repld close --session mypkg
```

Use `--fresh` when starting a task, after struct/type layout changes, module reorganization, dependency changes, or repeated Revise/world-age symptoms. For ordinary function-body edits, test tweaks, doc edits, and most method additions, reuse the warm session and load/call Revise before checks.

### repld Session Lifecycle

Close task-scoped `repld` sessions at the end of each implementation or testing phase, after recording any needed trace output and final command result. This prevents stale Julia processes, old Revise state, loaded manifests, and package images from leaking into the next Comet/OpenSpec phase or unrelated task.

Use:

```bash
repld sessions
repld close --session mypkg
```

Keep sessions open only while they are actively useful for a tight edit/test loop. Prefer `repld close --session NAME` for task cleanup; reserve daemon-wide `repld stop` for explicit all-session cleanup or when the user asks to stop the daemon.

### Optional MCP Servers

Kaimon.jl and julia-mcp can still be useful, especially for live introspection, method lookup, profiling, or environments where shell-backed `repld` is not available.

→ See `references/mcp-servers.md` for setup, full tool parameters, GateTool registration, and integration patterns.

| | repld | Kaimon.jl | julia-mcp | JETLS (LSP) |
|---|-------|-----------|-----------|-------------|
| Type | Long-lived process runner | MCP server + Gate | MCP server | Language Server |
| Best for | Package tests, examples, scripts, warm Revise loops | Deep introspection/debugging/profiling | Basic fallback execution | Static analysis/navigation |
| Default choice | Yes | If available and useful | Fallback | Complementary |

**Decision**: `repld` available → use it for execution. Kaimon available → use it for introspection/debugging in addition to `repld`. julia-mcp available → fallback for quick eval or bootstrapping. JETLS is complementary — use alongside any execution path.

**Tool quick reference**:
- `repld`: `--fresh --session NAME julia --project=... -e '...'`, `--session NAME julia -e '...'`, `trace`, `interrupt`.
- **Kaimon**: `kaimon_ex(e="code")`, `kaimon_run_tests(pattern="...")`, `kaimon_type_info()`, `kaimon_search_methods()`, `kaimon_code_typed()`, `kaimon_goto_definition()`.
- **julia-mcp**: `julia_eval(code, env_path?, timeout?)`, `julia_restart(env_path?)`, `julia_list_sessions()`.
- **JETLS**: diagnostics, hover, go-to-definition, find-references, rename, code actions, inlay hints, formatting.

---

## 8. Tool Sanity Probe

On skill activation for nontrivial work, run a lightweight sanity probe for the execution path. Prefer `repld`; do not require julia-mcp, Kaimon, or JETLS.

### Quick Probe

Run shell probes in parallel when possible:

```bash
command -v repld
repld --fresh --session julia-sanity julia --startup-file=no --project=. -e 'println("repld+julia ready: ", VERSION)'
repld close --session julia-sanity
```

Pass condition for ordinary Julia package work: `repld` is available and can run the one-line Julia command in the current project. If this passes, proceed with the task; do not install or update tools.

Optional probes, only when the task needs that capability:

| Capability | Probe |
|------------|-------|
| Kaimon introspection | `kaimon_ex(e="1+1", q=true)` if the MCP tool is exposed, otherwise `command -v kaimon` |
| julia-mcp fallback | `julia_list_sessions()` if the MCP tool is exposed |
| JETLS diagnostics | `command -v jetls && jetls --help 2>&1 \| head -5` |

If the preferred probe fails but plain `julia` works, continue with shell Julia or MCP fallback and mention the limitation once. Open install/update recipes only when a required tool is missing, a requested optional tool is missing, or a tool failure points to a known setup issue.

### Capability Matrix

| Execution backend | Kaimon | JETLS | What you can do |
|-------------------|--------|-------|-----------------|
| repld | ✅ | ✅ | Full: warm execution + introspection + static analysis + navigation |
| repld | ❌ | ✅ | Warm execution + LSP. No deep introspection/debugging. |
| repld | ❌ | ❌ | Warm execution only. Suggest JETLS/Kaimon only when static analysis or introspection would materially help. |
| MCP fallback | any | any | Use MCP execution/introspection fallback when `repld` is unavailable. |
| none | any | any | Cannot run Julia code through this skill; use shell Julia if allowed or install `repld`/MCP. |

### Install and Update Recipes

Keep install details out of the main workflow. Use them only after a sanity probe fails or the user explicitly asks to configure a tool:

- `repld`, julia-mcp, Kaimon, and JETLS: see `references/mcp-servers.md`.
- Revise/TestEnv and package testing: see `references/testing-and-repl.md`.

### When Tools Are Missing — Guidance

| Missing | Impact | Tell the user |
|---------|--------|---------------|
| repld | No warm session runner | "`repld` is unavailable, so I will use shell Julia or MCP fallback. Warm Revise loops and traces are limited." |
| julia | Blocking for local shell execution unless MCP provides Julia execution | "Julia is not in PATH and no Julia execution backend is available." |
| Kaimon | No optional introspection/debugging | "Kaimon is not connected, so deep live introspection is unavailable. I can continue with `repld`; setup details are in `references/mcp-servers.md` if needed." |
| JETLS | No optional JETLS diagnostics/navigation | "JETLS is not installed, so JETLS diagnostics/navigation are unavailable. I can continue with execution-based checks; setup details are in `references/mcp-servers.md` if needed." |

Report missing required tools, or optional tools that were attempted and unavailable, once per session. Do not repeat on every turn.

---

## 9. Environment & Dependency Workflow

Before executing Julia code, determine the correct `env_path` and dependency permissions.

### Classification

| Context | `env_path` | Add deps? |
|---------|-----------|-----------|
| Pkg in development (`src/` exists) | Pkg root | ⛔ Ask user |
| Project env (Project.toml, no `src/`) | Project dir | ⚠️ Ask user |
| Under `test/` of a pkg | Pkg root | ⛔ Ask user |
| Under `examples/` or `docs/` | Own Project.toml or pkg root | ✅ Quick confirm |
| Standalone / no Project.toml | `nothing` (temp env) | ✅ Free |
| Ephemeral / one-off task | `nothing` (temp env) | ✅ Free |

### Rules

1. **Use Pkg APIs for dependency graph changes.** Prefer `Pkg.add()` / `Pkg.rm()` / `Pkg.dev()` over hand-editing dependency entries or `Manifest.toml`. Manual `Project.toml` edits are acceptable for package metadata and explicit user-requested fields.
2. **Do not add deps to a package without approval.** Changing a pkg's dep graph affects downstream users unless the user explicitly requested that dependency change. If a dep is only needed for exploration, use a temp env instead.
3. **Ephemeral tasks → temp env** (`env_path=nothing`). Don't pollute a pkg's deps for one-off work.

### Detecting Execution Backend

1. `repld` present → preferred execution path.
2. `kaimon_` tools present → optional introspection/debugging.
3. `julia_eval` present → MCP fallback.
4. Plain `julia` in PATH → acceptable shell fallback when `repld`/MCP are absent.

---

## 10. Revise.jl and Session Lifecycle

Use Revise in warm sessions, especially with `repld`. Trust Revise for normal function-body edits, tests, examples, documentation, and many new method definitions. Do **not** restart automatically just because a core function changed.

Restart with `repld --fresh` when:
- struct fields, type parameters, constants, or generated-function assumptions change;
- module includes/exports are reorganized;
- dependencies or Project/Manifest state changes;
- Revise reports failures or repeated world-age/stale-method behavior appears;
- a large API refactor changes many call paths and the cost of stale state exceeds restart cost.

→ See `references/mcp-servers.md` for the full Revise limits table and restart commands per server.

---

## 11. Visualization & User Verification

Large Julia projects follow an **agent-generate → inspect what is inspectable → user-verify** workflow. Live interactive plots often need user inspection, while saved images or screenshots can sometimes be checked directly by the agent when local visual tools are available.

### Rule: Inspect or Hand Off Visual Output

When code produces plots, figures, or visualizations:
1. **Generate the output** (run the plotting code via `repld` or MCP).
2. **Inspect saved artifacts when possible** — use local image/browser/screenshot tools for static files or served HTML when available.
3. **Tell the user what remains visual** — describe what was generated, where it is, and what still needs human inspection.
4. **Never claim success** for visual properties that were not actually inspected.

### WGLMakie for Agent-Accessible Visualization

Use **WGLMakie** (web-based Makie backend) when the agent needs to produce plots the user can inspect in a browser:

```julia
using Makie
import WGLMakie as WM  # Ensure WGLMakie is available
import Bonito  # use Bonito to open browser tabs
Bonito.Page(listen_port=8000)  # Explicitly specify port if needed
WM.activate!()  # Set WGLMakie as the active backend
# WGLMakie opens a browser window, serves via Bonito/JSServe, or saves HTML
fig = scatter(rand(100), rand(100))
display(fig)  # Renders in browser — user can inspect
save("plot.html", fig)  # Static HTML handoff when supported by the active backend
```

Key points:
- WGLMakie renders to a browser (no display server needed on remote/headless).
- For `repld` sessions: prefer saving static HTML with Bonito/WGLMakie when possible, report the absolute output path, and inspect generated HTML/logs for obvious render failures when debugging.
- For Kaimon sessions: the user's Julia process serves the plot; user views in their browser.
- For julia-mcp sessions: works if the Julia process can open a port; otherwise save to file.
- **Fallback**: Use `CairoMakie` to save static images (`save("plot.png", fig)`) and tell the user to open the file.

### Verification Checklist

| Output type | Agent can verify? | Action |
|-------------|-------------------|--------|
| Numerical results | Yes | Check inline |
| Text/strings | Yes | Check inline |
| Plots/figures | Sometimes | Inspect saved images/screenshots when tools are available; otherwise tell user to inspect |
| Interactive widgets | Partially | Smoke-check load/render when possible; tell user to inspect interactions |
| Data saved to file | Partially (file exists) | Tell user to open and verify |

---

## 12. Documentation

→ **Load `docs-style-preferences` skill** for Julia package documentation (Documenter.jl, docstrings).

Key Documenter.jl specifics:
- No parentheses in headers (breaks cross-references)
- Inline math: double backticks ``x = y``
- Display math: `math` code block
- Code symbols: single backticks `DynamicsProblem`
- **Draft builds**: use `draft = true` in `makedocs()` to skip `@example` execution. Wrap with an ENV variable for easy switching:
  ```julia
  const _draft = get(ENV, "DOCUMENTER_DRAFT", "false") == "true"
  makedocs(; ..., draft = _draft)
  ```
  Then run fast syntax-only builds with `DOCUMENTER_DRAFT=true julia --project=docs docs/make.jl`.

---

## References

- [Julia Manual](https://docs.julialang.org/en/v1/manual/)
- [Style Guide](https://docs.julialang.org/en/v1/manual/style-guide/)
- [Performance Tips](https://docs.julialang.org/en/v1/manual/performance-tips/)
- [Pkg.jl](https://pkgdocs.julialang.org/v1/)
- [Documenter.jl](https://documenter.juliadocs.org/stable/)
- [Kaimon.jl](https://github.com/kahliburke/Kaimon.jl) — feature-rich MCP server with deep Julia introspection
- [julia-mcp](https://github.com/aplavin/julia-mcp) — minimalist agent-managed MCP server
- [JETLS.jl](https://aviatesk.github.io/JETLS.jl/release/) — Julia language server with JET.jl-powered static analysis
