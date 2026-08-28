# Testing & REPL Reference

Detailed reference for Julia testing patterns and REPL-based package management.

---

## Test.jl

### Basic Assertions

```julia
using Test

@test 2 + 2 == 4
@test_throws BoundsError [1,2,3][4]
@test π ≈ 3.14 atol=0.01
```

### Test Sets

Group related tests for structured reporting:

```julia
@testset "arithmetic" begin
    @test 1 + 1 == 2
    @test 2 - 1 == 1
end

@testset "arrays" begin
    @test length([1,2,3]) == 3
    @test [1,2] ⊆ [1,2,3]
end
```

### TestEnv for Interactive Testing

After environment preparation, use TestEnv to start an interactive test-driven
session:

```julia
using TestEnv
TestEnv.activate("MyPackage")
using Revise, MyPackage
include("test/unit/foo_tests.jl")
```

This is useful because `Pkg.test` runs tests in a temporary environment that
contains the package plus test-only dependencies from `[extras]`/`[targets]` or
`test/Project.toml`. The plain package environment often does not include those
dependencies. `TestEnv.activate()` lets you enter an equivalent environment
interactively for focused test writing and debugging. Target a selected test
file or group; do not default to the package's full `test/runtests.jl` runner.
It does not replace a final fresh `Pkg.test` run for a project-selected gate.

### Package Runtime and Test-Dependency Warming

For package `src/` and `test/` edits, every process uses
`--project=MyPackage`. Keep the no-Revise warm processes separate from the
Revise session: Revise can only track code loaded in the process after Revise
itself is loaded. The runtime warm loads the package without TestEnv; the test
warm activates TestEnv and warms test dependencies without loading the package,
tests, or editable source files.

```bash
# Runtime warm: no TestEnv and no Revise.
repld --fresh --session mypkg-runtime-warm julia --startup-file=no --project=MyPackage -e 'import Pkg; Pkg.instantiate(); using MyPackage'
repld close --session mypkg-runtime-warm

# Test-dependency warm: no Revise, MyPackage, tests, or source files.
repld --fresh --session mypkg-test-warm julia --startup-file=no --project=MyPackage -e 'import Pkg, TestEnv; TestEnv.activate(); Pkg.instantiate(); Pkg.precompile()'
repld close --session mypkg-test-warm

# Hot focused-test loop: TestEnv first, Revise second, package third.
repld --fresh --session mypkg-test-dev julia --startup-file=no --project=MyPackage -e 'import TestEnv; TestEnv.activate(); using Revise; using MyPackage'
repld --session mypkg-test-dev julia -e 'Revise.revise(); include("test/unit/foo_tests.jl")'
```

Run `Pkg.resolve()` only after an explicit project/dependency edit or manifest
conflict. Do no Pkg work after Revise is loaded in the hot session. Package
precompilation does not transfer every JIT specialization into the separate hot
process, so judge re-test speed from a second focused include in that same
session, not from its first execution.

Use focused includes for tight loops:

```bash
repld --session mypkg-test-dev julia -e 'Revise.revise(); include("test/unit/foo_tests.jl")'
```

For a package whose runner uses an environment selector, set it inside the
Julia evaluation. Do not use `GROUP=UNIT repld ...`: a persistent daemon session
does not inherit that per-client shell assignment.

```bash
repld --session mypkg-test-dev julia -e 'ENV["GROUP"] = "UNIT"; Revise.revise(); Base.include(Main, joinpath(pkgdir(MyPackage), "test", "runtests.jl"))'
```

Keep one aggregate selector per hot session unless the runner reads `ENV`
dynamically on every include. A runner that stores its selector in a `const`
when first included retains that first choice; start a fresh hot session before
selecting another group. For the tightest loop, prefer a documented standalone
leaf test over an aggregate runner, because an aggregate may repeatedly create
test sets and compile their setup.

### Selected Pkg.test Gate

`Pkg.test()` is not part of the TestEnv warm loop. It creates its own temporary
test environment and SHALL run from a separate fresh session with the package
project active. Do not load TestEnv or Revise in that session:

```bash
repld --fresh --session mypkg-pkgtest julia --startup-file=no --project=MyPackage -e 'import Pkg; Pkg.test(; coverage=false)'
```

For a selector-driven fresh gate, set the selector before `Pkg.test()` so its
isolated subprocess inherits it:

```bash
repld --fresh --session mypkg-unit-pkgtest julia --startup-file=no --project=MyPackage -e 'ENV["GROUP"] = "UNIT"; import Pkg; Pkg.test(; coverage=false)'
```

Keep `TestEnv` as a developer tool in a global/dev environment; do not add it
as a dependency of the package under test.

### Docs, Review Cases, and Temporary Environments

Documentation and human-facing review cases are non-package workflows, so they
never use TestEnv. A durable docs or case directory can use a separate
no-Revise precompile process and a fresh Revise session. Do not load editable
case definitions before the Revise session.

```bash
# Docs project: no TestEnv.
repld --fresh --session docs-warm julia --startup-file=no --project=docs -e 'import Pkg; Pkg.instantiate(); Pkg.precompile()'
repld close --session docs-warm
repld --fresh --session docs-dev julia --startup-file=no --project=docs -e 'using Revise; using MyPackage'
repld --session docs-dev julia -e 'Revise.revise(); include("docs/make.jl")'

# Durable human-review case: no TestEnv.
repld --fresh --session case1-warm julia --startup-file=no --project=examples/cases/case1 -e 'import Pkg; Pkg.instantiate(); Pkg.precompile()'
repld close --session case1-warm
repld --fresh --session case1-dev julia --startup-file=no --project=examples/cases/case1 -e 'using Revise; using MyPackage; includet("examples/cases/case1/definitions.jl")'
repld --session case1-dev julia -e 'Revise.revise(); include("examples/cases/case1/run.jl")'
```

`--project=@temp` is process-local. Start one named session, finish Pkg setup
without Revise or editable code, then load Revise in that same session. A fresh
`@temp` session requires recreating its environment setup.

```bash
repld --fresh --session scratch-dev julia --startup-file=no --project=@temp -e 'import Pkg; Pkg.develop(path="MyPackage"); Pkg.instantiate(); Pkg.precompile()'
repld --session scratch-dev julia -e 'using Revise; using MyPackage; includet("scratch/definitions.jl")'
repld --session scratch-dev julia -e 'Revise.revise(); include("scratch/run.jl")'
```

### Environment-selected aggregate suites

For a package whose `test/runtests.jl` selects a test group through an
environment variable, use TestEnv for the warm interactive loop and `Pkg.test`
for verification. The selector and its allowed values belong to the package;
`MECHANICS` is only an example. This is still a package-owned test path:
shared analyzers, formatters, documentation builds, coverage tooling, and
benchmarks stay outside `Pkg.test()` as described in `package-quality-gates.md`.

```bash
repld --session mypkg-test-dev julia -e '
    Revise.revise()
    previous_group = get(ENV, "GROUP", nothing)
    try
        ENV["GROUP"] = "MECHANICS"
        Base.include(Main, joinpath(pkgdir(MyPackage), "test", "runtests.jl"))
    finally
        isnothing(previous_group) ? delete!(ENV, "GROUP") : (ENV["GROUP"] = previous_group)
    end
'
```

This reuses the hot session above, where TestEnv is already active and Revise
loaded `MyPackage`. Restore the selector yourself, as above. Use `Base.include`
for the test entry point: it evaluates the runner at top level and avoids a
Julia world-age error caused by importing a newly available test-only dependency
and immediately calling it inside a callback.

Run a leaf file directly only when it documents its complete standalone harness;
TestEnv supplies dependencies, not runner imports, helpers, configuration, or
ordering.

For multi-command interactive debugging, use a TestEnv-plus-Revise session and
keep test dependencies available in later `repld` evaluations:

```bash
repld --fresh --session mypkg-debug julia --startup-file=no --project=/path/to/MyPackage -e 'import TestEnv; TestEnv.activate(); using Revise; using MyPackage'
repld --session mypkg-debug julia -e 'Revise.revise(); using TestDependencies; include("debug_case.jl")'
repld close --session mypkg-debug
```

Do not leave this persistent session active across unrelated work. Start a
fresh session after dependency, type-layout, or module-structure changes.

Confirm an interactive green group with Pkg's real isolated test subprocess:

```bash
repld --fresh --session mypkg-group julia --startup-file=no --project=/path/to/MyPackage -e '
    ENV["GROUP"] = "MECHANICS"
    import Pkg
    Pkg.test(; coverage=false)
'
repld close --session mypkg-group
```

TestEnv can re-resolve its temporary dependency graph and a warm process can
retain code or mutable state. It is therefore an interactive tool, not a
byte-for-byte replacement for fresh `Pkg.test` evidence.

### Tool boundaries and session lifecycle

- `TestEnv.activate`: warm interactive test execution with a package's declared
  test extras, targets, or `test/Project.toml` dependencies.
- Fresh `Pkg.test`: final evidence for a selected group or package test target;
  it runs the package manager's isolated test subprocess.
- JET: inference and compilation analysis for declared concrete calls. It is
  not a mechanics, numerical-accuracy, or runtime-behavior test.
- Runic and Aqua: formatting and package-hygiene checks. They are not mechanics
  verification.
- Allocation/time profiles: evidence for choosing an optimization target and
  comparing a declared change; never a machine-dependent correctness gate.

Use a named `repld` session for one active iteration phase and close it at that
phase's completion, after its evaluation has returned success or a completion
marker. A host shell integration can yield while a long `Pkg` evaluation still
runs; wait or poll before closing the named session. Start a fresh session after
environment or dependency changes, type-layout changes, module/include
reorganization, changed thread or Julia-version assumptions, or repeated
Revise/world-age symptoms.

---

## Pkg API (Programmatic)

Use Pkg APIs for dependency graph changes. Prefer `Pkg.add()` / `Pkg.rm()` / `Pkg.dev()` over hand-editing dependency entries or `Manifest.toml`; manual `Project.toml` edits are still appropriate for package metadata and explicit user-requested fields.

```julia
using Pkg

# Inspect current environment
Pkg.status()
Pkg.project().path           # Current Project.toml path

# Add/remove deps
Pkg.add("PackageName")
Pkg.add(url="https://github.com/user/Package.jl")
Pkg.rm("PackageName")

# Activate environments
Pkg.activate("/path/to/env")  # Specific env
Pkg.activate(; temp=true)      # Temporary env
```

---

## Revise.jl

Auto-reload code changes during development. Add to `~/.julia/config/startup.jl`:

```julia
using Revise
```

For package development:
```julia
pkg> dev MyPkg
julia> using MyPkg  # Changes auto-reload on file save
```

For scripts:
```julia
julia> includet("myscript.jl")  # Track and reload changes
```

In a warm `repld` session, load a definitions script with `includet` once, then
call `Revise.revise()` before the focused check that exercises it. Keep
side-effecting computations in a separate script and re-run that script with
ordinary `include` when needed. `includet` defaults to `:evalmeth`, so it does
not rerun arbitrary top-level statements; set `__revise_mode__ = :eval` only
when full top-level re-evaluation is intentional and safe.

`includet` is non-recursive. For a multi-file source tree, prefer a dev'd
package loaded with `using` or `import`; do not expect `includet` on one parent
file to track the files it includes.

### Revise Limits And Restart Rules

Revise is excellent for ordinary method-body edits, new methods, test tweaks,
and many package-source changes. Use a fresh Julia session after:

- changing struct fields or type hierarchy, unless Julia 1.12+ struct revision
  was deliberately enabled with Revise's `revise_structs` preference and no
  pre-existing values or stale state can affect the check;
- moving files/modules or changing `include` order;
- changing dependencies or environments;
- repeated world-age, stale-method, or precompile-cache symptoms.

On Julia versions older than 1.12, Revise does not support changes to `struct`
definitions. Newer versions improve this, but a fresh session remains the safe
default after type-layout changes in serious package work.

### Revise.jl with MCP Servers

Both MCP servers integrate with Revise for hot-reloading — see `mcp-servers.md` for server-specific behavior and the full Revise limits table (when auto-reload works vs when to restart).
