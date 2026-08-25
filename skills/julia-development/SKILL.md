---
name: julia-development
description: This skill should be used when the user asks to "write Julia code",
  "refactor Julia", "explain multiple dispatch", "optimize Julia performance",
  "Julia types", "Julia Pkg management", or "idiomatic Julia". Covers Julian
  style, type system, performance, Pkg, environment detection, repld-first
  execution, optional MCP server integration (Kaimon.jl and julia-mcp), and
  JETLS for static analysis.
  Not for non-Julia tasks. For package docs, load docs-style-preferences.
version: 4.2.0
tags: [Julia, MultipleDispatch, Types, Performance, Environment, Pkg, repld, MCP, JETLS, QA]
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
| Encapsulation | Modules + explicit `public` API declarations (Julia >= 1.11) |

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

### API Design With Dispatch

Prefer generic functions plus typed dispatch over public `Symbol`/`String` mode
branches, `Dict` routing, or large mode switches when behavior actually
changes. Prefer concrete marker or selector types—even zero-field types—for
distinct behavior. Keep symbol keywords only as compatibility wrappers or
user-facing convenience layers.

```julia
abstract type ProcessingMode end
struct FastMode <: ProcessingMode end
struct AccurateMode <: ProcessingMode end

process(data, ::FastMode) = fast_process(data)
process(data, ::AccurateMode) = accurate_process(data)

# Compatibility wrapper only when needed.
function process(data; mode::Symbol=:fast)
    selected = mode === :fast ? FastMode() :
               mode === :accurate ? AccurateMode() :
               throw(ArgumentError("unsupported mode: $mode"))
    return process(data, selected)
end
```

Use shallow abstract type hierarchies for open extension points, concrete
immutable structs for data, and parametric fields for storage/scalar/backend
types. Use traits when behavior depends on a capability rather than natural type
identity.

### State Ownership and Operation Names

A field owns the state named by that field. Do not use
`Base.getproperty`/`Base.setproperty!` forwarding to manufacture
pseudo-fields: it hides ownership, weakens local reasoning, and often
reintroduces symbol-based routing. Expose behavior through ordinary functions;
keep implementation fields descriptive rather than allowing them to become an
accidental public API.

Name operations for their actual direction and effect. A one-way update is not
a synchronization; a validation is not a mutation. Avoid vague nouns such as
`family` or `capability` when a concrete domain noun states what the
operation owns or consumes. Do not encode visibility with leading underscores
or retain compatibility aliases without a real compatibility requirement.

Introduce a coordinating struct only when it owns durable state, invariants, or
a meaningful lifecycle. Prefer an existing value, a constructor, a tuple, or
direct dispatch for short-lived coordination.

For Julia >= 1.11 packages, declare every supported type and generic-function
binding with `public` near the module header; a public generic function owns its
supported methods. `public` records API intent without importing the name for
`using Package`; use `export` only for names that users should receive
unqualified. Do not encode visibility through `_helper`, `PrivateType`,
`InternalType`, `TypeImpl`, or similar names when the distinction is merely
public versus internal. Julia has no hard private bindings: use descriptive
domain names, declare supported bindings `public`, and leave implementation
details undeclared as public.

```julia
public Dataset, load, configure!
export load
```

```julia
abstract type StorageBackend end
struct InMemory <: StorageBackend end
struct OnDisk <: StorageBackend end

struct Dataset{T,B<:StorageBackend}
    values::Vector{T}
    backend::B
end

load(d::Dataset{T,InMemory}) where {T} = d.values
load(d::Dataset{T,OnDisk}) where {T} = read_from_disk(d)
```

---

## 3. Type System & Performance

→ See `references/language-deep-dive.md` for type annotations, type stability, broadcasting fusion, loop optimization, and `Val` types.

**Key rules**:
- Put performance-critical code inside functions; avoid top-level script logic for reusable package behavior
- Default to immutable `struct`; use `mutable struct` only when mutation is required
- Keep functions type-stable: return types should be inferable from input types, not from unpredictable runtime values
- Avoid abstract fields and abstract element containers in performance-sensitive data; prefer parametric fields such as `Container{T}`
- Type the fields read in hot loops, including storage, keys, and source collections; avoid `Any` and heterogeneous convenience vectors there. Do not over-engineer a type hierarchy merely to type opaque backend or UI handles that are outside the hot path.
- Traverse the owning typed collection directly in a hot path. Do not first allocate a generic accessor result or source-reference vector merely to recover dynamic dispatch.
- When cardinality is known from construction or a query, allocate exact destination storage and use indexed writes. Do not use `push!`, `append!`, `resize!`, repeated growth, shrinking, or compaction during a fixed-layout refresh.
- Use dotted calls or `@.` for fused broadcasting when it improves clarity and avoids unnecessary temporaries
- Annotate types for dispatch, invariants, or documentation; concrete argument annotations rarely improve performance by themselves
- Declare supported package bindings with `public`; use `export` only when unqualified `using Package` access is intended. Direct field access should be public API only when explicitly declared and documented.

### Fixed Layouts, Batches, and Updates

Separate construction-time layout from refreshable state. Identities, topology,
resource choices, and capacities are fixed facts; positions, values, styles,
and other payloads are refreshable facts. An ordinary update overwrites
compatible dynamic state. A layout mismatch uses an explicit reconstruction
path—never silent reconciliation or resizing of a fixed object.

Batch by the concrete data/component/resource requirements consumed by an
operation, not by the source object that produced a row. A source is often an
owner rather than the unit of batched work.

Keep invalidation narrow: distinguish a layout or selection change from a
content change, update only affected nonempty consumers, and publish coupled
outputs together so observers cannot see mixed revisions.

---

## 4. Common Patterns

### Bang Convention

Use `!` only when a function mutates one of its arguments or other externally
visible state. Validation, lookup, construction, and pure conversion do not
take `!`.

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

### Constructors For Object Creation

Prefer constructors over extra constructor-like helper functions when the goal
is to build an instance of a type. Use outer constructors for alternate input
forms, defaults, conversion, and ergonomic APIs; use inner constructors only
when every instance must enforce an invariant.

```julia
struct Config{T}
    threshold::T
    name::String

    function Config(threshold::T, name::AbstractString) where {T<:Real}
        @assert threshold >= zero(T) "threshold must be nonnegative"
        return new{T}(threshold, String(name))
    end
end

Config(threshold::Real) = Config(threshold, "default")
Config(; threshold=1.0, name="default") = Config(threshold, name)
```

Avoid adding `make_config(...)`, `create_config(...)`, or
`build_config(...)` when `Config(...)` is the natural API. Use a named function
only when it performs a distinct operation, such as loading from disk,
discovering environment state, mutating an existing object, or running a
multi-step workflow.

### Views and Published Storage

Use `@view`/`@views` for internal, non-owning slice operations when a view
does not escape into an API with stricter storage expectations. Do not assume
every downstream package, plotting backend, foreign-function boundary, or
long-lived observable accepts a `SubArray` as a stable published value.
Materialize an exact concrete container only at that unavoidable boundary; keep
the fixed staging allocation and internal slice logic otherwise.

### Common Macros

Use macros intentionally:

- `@test`, `@testset`, `@test_throws`: test behavior in `test/`, not package logic.
- `@assert`: check internal invariants during development; do not use it as the only user-facing validation for recoverable input errors.
- `@views`: avoid copies from slicing when a view is intended.
- `@.`: fuse broadcasts when it improves clarity and avoids temporaries.
- `@time`, `@allocated`, `@code_warntype`: quick diagnostics; use `BenchmarkTools.@btime` for serious benchmarking.

Do not introduce macros just to make ordinary function calls look clever. Prefer
functions unless a macro is needed for syntax, generated code, source-location
information, testing, diagnostics, or benchmarking support.

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

→ See `references/testing-and-repl.md` for Pkg API usage, Test.jl, TestEnv, aggregate test groups, and Revise.jl.

---

## 6. Common Mistakes (OOP → Julia)

→ See `references/oop-migration.md` for detailed examples of OOP→Julia pitfalls and their fixes.

**Quick checklist**:
- Don't put methods inside `struct` — use external functions with type annotations
- Don't write `make_*` or `create_*` helpers when an outer constructor is the natural object-building API
- Don't over-specify types (`Int64`) — let Julia specialize generically
- Don't encode real polymorphism as large public `if mode == :foo` branches; use typed selectors, traits, or separate methods
- Don't hide state ownership with `getproperty`/`setproperty!` forwarding or call a one-way update `sync`
- Don't encode visibility with `_` prefixes or `Private`/`Internal`/`Impl` names; for Julia >= 1.11, declare supported bindings with `public` and use normal descriptive names
- Avoid type piracy: do not extend functions you don't own on types you don't own. Extending Base or package functions for your own types is normal Julia style.
- Default to `struct` (immutable) — use `mutable struct` only when needed
- Don't grow or resize a fixed-layout buffer during routine refresh; allocate known capacity and write by index
- Keep examples and skill guidance domain-neutral unless the user's current task is domain-specific; prefer examples from data processing, storage backends, plotting backends, parsers, optimizers, or numerical algorithms

---

## 7. Execution Tooling

Prefer `repld` for Julia execution when available. It gives long-lived named sessions, works well with Revise, survives ordinary code edits, and makes test iteration reproducible from shell commands. MCP servers are optional supplements for introspection or environments where `repld` is unavailable.

### Import Placement and Review-Case Scripts

Classify the file by the module in which its code executes, then place its
`using` and `import` statements accordingly:

| Scenario | Where dependency imports belong | Revise |
|----------|---------------------------------|--------|
| Package source or code included by a package module | At the top level of the innermost module that directly uses the dependency. An `include`d file executes in its including module, not in `Main`. | Do not add `Revise` to package source. |
| Temporary script executed as `julia script.jl` | At the top of the script, in `Main`, before setup or executable code. | Add only when the temporary interactive workflow needs hot loading. |
| Human-facing review-case script executed in `Main` | At the top of the script, before setup or case code. | The first dependency import is `using Revise`, before loading the reviewed package. |

For package code, an outer module's imports do not become bindings in a child
module. Put an import in the child module even when the parent also uses that
package.

```julia
# src/solver/step.jl, included by RiblePackage.Solver
using LinearAlgebra
import StaticArrays: SVector

# Solver implementation that uses dot, norm, and SVector
```

When a temporary script defines a module, imports used by that defined module
belong inside it; only imports used by the script runner remain in `Main`.

```julia
# temporary_script.jl, executed in Main
using Rible
import CairoMakie

# setup and one-off work
```

```julia
# review_cases/shell_preview.jl, executed in Main
using Revise
using Rible
import CairoMakie

# human-facing setup and review case
```

`Revise` is shared developer tooling, like JET. Do not add it to a reviewed
case's `Project.toml` or to the package's dependencies solely for this script.

### Environment Setup and Hot Reload

Never mix package-manager work with hot reload. First prepare the environment,
then begin the warm execution loop:

1. **Preparation:** activate the target environment and perform only necessary
   `Pkg` operations such as `instantiate`, `resolve`, `add`, `develop`, or an
   explicitly approved `update`. When preparing a test environment, call
   `TestEnv.activate()` before those `Pkg` operations. Do not load `Revise`,
   the package under development, tests, or source scripts in this phase.
2. **Warm loop:** once preparation is clean, start a fresh named `repld`
   session, activate its test or case environment before loading code, then
   `using Revise`. Do no further `Pkg` operations in that session; a dependency
   or environment change ends the loop and requires preparation plus a fresh
   session.

For normal test-driven package work, use the package's interactive test
environment. `TestEnv.activate()` without a `do` block keeps that environment
active for the warm session:

```bash
# Preparation only: no Revise, package, tests, or source files.
julia --project=MyPkg -e 'import Pkg, TestEnv; TestEnv.activate(); Pkg.instantiate(); Pkg.resolve()'

# Warm test-driven loop: TestEnv first, then Revise, then the package.
repld --fresh --session mypkg-test julia --project=MyPkg -e 'import TestEnv; TestEnv.activate(); using Revise; using MyPkg; println("ready")'
repld --session mypkg-test julia -e 'Revise.revise(); include("test/unit/foo.jl")'
```

When project policy selects a `Pkg.test()` gate, it is a separate fresh process
with the package project active. `Pkg.test()` creates and owns its temporary
test environment; never invoke it from a TestEnv-activated or Revise session:

```bash
repld --fresh --session mypkg-pkgtest julia --project=MyPkg -e 'import Pkg; Pkg.test(; coverage=false)'
```

For a human-facing review case, do not use `TestEnv`. Give the case its own
anonymous but persistent `Project.toml` (no package `name` or `uuid`) and
activate that case directory with `--project=<case-directory>`. Its `[sources]`
shall declare local packages under review by relative path. Complete its `Pkg`
setup before loading code, then use Revise in a fresh named session:

```bash
# Preparation only, in the case's own anonymous Project.toml.
julia --project=/absolute/path/review_cases/foo -e 'import Pkg; Pkg.instantiate(); Pkg.resolve()'

# No TestEnv and no further Pkg operations after this point.
repld --fresh --session case-foo julia --project=/absolute/path/review_cases/foo -e 'using Revise; using MyPkg; includet("/absolute/path/review_cases/foo_definitions.jl")'
repld --session case-foo julia -e 'Revise.revise(); include("/absolute/path/review_cases/run_foo.jl")'
```

Changes to package `src/` loaded with `using MyPkg` and to files loaded once
with `includet` are picked up in the warm loop. `--project=@temp` and
`Pkg.activate(; temp=true)` are for genuinely disposable environments, not
human-facing review cases.

Sessions created from Codex or Claude Code automatically attach to the agent harness and close when that harness exits. Do not pass `--owner-pid` manually for ordinary agent work. Autoclose is a fallback for interrupted or abandoned work, not a replacement for explicitly closing a completed task session. Use `repld free <id | --session=NAME>` only when a session must intentionally outlive the agent; it removes that ownership lease.

Use `--fresh` when starting a task, after module reorganization, dependency changes, thread-count or Julia-channel assumptions, or repeated Revise/world-age symptoms. Revise can usually hot-reload ordinary function-body edits, many method additions, and many changes in dev'd packages. Treat a struct/type redefinition as a fresh-session trigger by default: Julia 1.12+ can revise structs only when Revise's `revise_structs` preference is deliberately enabled, and that does not make pre-existing values or stale runtime state valid. Adding a new `using NewPkg` inside a module when `NewPkg` was absent from the project when the session started also requires a fresh session.

For a non-package definitions script, call `includet("path/to/definitions.jl")`
once in the warm session rather than using `include`; its default `:evalmeth` mode
tracks method definitions without rerunning arbitrary top-level work. Keep
side-effecting computations in a separate script and use ordinary `include` only
when deliberately rerunning those computations. Set `__revise_mode__ = :eval`
only when full top-level re-evaluation is wanted and safe.

Use `--trace smart` by default for user/project frames plus nearby boundary frames. Use `--trace full` when package internals, generated code, or Julia runtime frames matter. `repld trace --trace smart --session NAME` shows the last saved traceback without rerunning the failing command; `repld sessions` also exposes short session IDs accepted by `trace`, `interrupt`, and `close`.

Use `TestEnv.activate` only for interactive, focused test-file execution when
tests need `[extras]`, `[targets]`, or `test/Project.toml` dependencies that
are not available in the plain package environment. This applies to
package-owned test infrastructure such as `Aqua` and `SafeTestsets`, not
analyzers or formatters. For aggregate selectors, run the package test entry
point through `TestEnv.activate` during the warm loop and confirm with a fresh
project-selected `Pkg.test()` gate. Use `Pkg.test()` only in a fresh
package-project session; it may include functional tests, Aqua, and
SafeTestsets, but it must never invoke JET, ExplicitImports, Runic,
JuliaFormatter, or other shared QA tooling. Do not add TestEnv as a package
dependency; treat it as a developer tool available from the global/dev
environment. See `references/testing-and-repl.md` and
`references/package-quality-gates.md`.

ReTest is optional and conditional, not a default dependency. On Julia 1.12.x, ReTest 0.3.x is compatible and can be useful for regex-filtered testsets, deferred tests, shuffling, or parallel test execution when the package already uses or explicitly wants that style. ReTest 0.4.x requires Julia 1.13, so do not recommend it for the current Julia 1.12 fleet. Prefer plain `Test`, focused project test selection, and `TestEnv.activate` unless ReTest's filtering/deferred-test model materially improves the task.

### repld Session Lifecycle

Close task-scoped `repld` sessions at the end of each implementation or testing phase, after recording any needed trace output and final command result. This prevents stale Julia processes, old Revise state, loaded manifests, and package images from leaking into the next Comet/OpenSpec phase or unrelated task. Codex-created sessions also auto-close when the Codex harness exits; retain explicit closure for normal completion and use autoclose as interruption protection.

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

---

## 9. Julian Review Checklist

Before finishing nontrivial Julia changes, check:

- Does reusable behavior live in functions rather than top-level scripts?
- Does polymorphism use dispatch, typed selectors, or traits instead of large mode branches?
- Are public selectors typed when behavior changes, with `Symbol` wrappers only for compatibility?
- Are supported types and functions declared with `public`, with `export` reserved for intended unqualified imports rather than used as the only visibility signal?
- Are object-building APIs expressed as constructors instead of redundant `make_*` functions?
- Are structs immutable unless mutation is required?
- Are fields concrete or parametrically typed where performance matters?
- Does each struct expose actual ownership through fields rather than pseudo-fields forwarded through `getproperty`/`setproperty!`?
- Are `!` suffixes reserved for actual mutation, and do operation names state their real direction and effect?
- Does a fixed-layout object separate construction data from refreshable data and reject incompatible layout changes explicitly?
- Does each hot path traverse native typed collections and avoid `Any`, abstract fields, and heterogeneous temporary source vectors?
- Where cardinality is known, are buffers exact-sized and filled by indexed writes rather than grown, shrunk, or compacted during routine refresh?
- Are views confined to internal slice operations, with concrete containers materialized only at downstream publication boundaries that require them?
- Are batches grouped by compatible consumed data rather than by source ownership, and are selection/layout invalidation and content invalidation updated independently?
- Are signatures generic enough for `Real`, `AbstractVector`, `StaticArrays`, AD numbers, and unitful values when applicable?
- Are public APIs exposed through functions rather than undocumented field access?
- Are macros limited to appropriate roles such as tests, assertions, views, diagnostics, or benchmarking?
- Do performance-oriented tests verify durable semantics such as identity preservation, exact row counts, fixed-layout rejection, and active-only work rather than brittle global allocation ceilings?
- Are package-owned functional and test-only checks runnable through a selected `Pkg.test()` gate and, for focused iteration, through `TestEnv.activate`, with shared analyzers kept out of both test paths?
- Was `Revise.revise()` used before warm-session checks, and was a fresh session used after type/module/dependency changes?

If the preferred probe fails but plain `julia` works, continue with shell Julia or MCP fallback and mention the limitation once. Open install/update recipes only when a required tool is missing, a requested optional tool is missing, or a tool failure points to a known setup issue. For package-level QA gates, read `references/package-quality-gates.md`; `PkgTemplates` belongs only to new package/app scaffolding, not quality checks.

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

## 10. Environment & Dependency Workflow

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
2. **Separate test dependencies from analyzers.** A package-owned test suite may declare `Aqua` and `SafeTestsets` in `[extras]` and the `test` target; they are test-only dependencies and may run under `Pkg.test()`. Do not add `ExplicitImports`, `JET`, `JETLS`, `Runic`, `JuliaFormatter`, `BenchmarkTools`, or `Documenter` to the target package solely for agent QA. Install and run those tools from the active Julia version's default shared environment or a temporary environment, with approval for dependency changes.
3. **Ephemeral tasks → temp env** (`env_path=nothing`). Don't pollute a pkg's deps for one-off work.
4. **Keep analyzers outside `Pkg.test()`.** `Pkg.test()` runs package-owned tests and test-only dependencies. Run JET/JETLS diagnostics, ExplicitImports, Runic/JuliaFormatter, Documenter draft builds, and benchmarks as separate shared-tooling commands; do not hide them in a broad test invocation.

### Detecting Execution Backend

1. `repld` present → preferred execution path.
2. `kaimon_` tools present → optional introspection/debugging.
3. `julia_eval` present → MCP fallback.
4. Plain `julia` in PATH → acceptable shell fallback when `repld`/MCP are absent.

---

## 11. Revise.jl and Session Lifecycle

Use Revise in warm sessions, especially with `repld`. Trust Revise for normal function-body edits, tests, examples, documentation, and many new method definitions. Do **not** restart automatically just because a core function changed.

Restart with `repld --fresh` when:
- struct fields, type parameters, constants, or generated-function assumptions change;
- module includes/exports are reorganized;
- dependencies or Project/Manifest state changes;
- Revise reports failures or repeated world-age/stale-method behavior appears;
- a large API refactor changes many call paths and the cost of stale state exceeds restart cost.

→ See `references/mcp-servers.md` for the full Revise limits table and restart commands per server.

---

## 12. Visualization & User Verification

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

## 13. Documentation

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
