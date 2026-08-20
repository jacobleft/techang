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

Use TestEnv to activate a package's test environment interactively:

```julia
using TestEnv
TestEnv.activate("MyPackage") do
    using Test, TestDependencies
    include("test/unit/foo_tests.jl")
end
```

This is useful because `Pkg.test` runs tests in a temporary environment that
contains the package plus test-only dependencies from `[extras]`/`[targets]` or
`test/Project.toml`. The plain package environment often does not include those
dependencies. `TestEnv.activate()` lets you enter an equivalent environment
interactively for focused test writing and debugging. Target a selected test
file or group; do not default to the package's full `test/runtests.jl` runner.
It does not replace a final fresh `Pkg.test` run for a project-selected gate.

Preferred warm-session pattern with `repld`:

```bash
repld --fresh --session mypkg julia --project=MyPackage -e 'using Revise; using MyPackage; println("ready")'
repld --session mypkg julia -e 'using Revise; Revise.revise(); import TestEnv; TestEnv.activate("MyPackage") do; include("test/unit/foo_tests.jl"); end'
```

Use focused includes for tight loops:

```bash
repld --session mypkg julia -e 'using Revise, TestEnv; Revise.revise(); TestEnv.activate("MyPackage") do; Base.include(Main, joinpath(pkgdir(MyPackage), "test", "unit", "foo_tests.jl")); end'
```

Keep `TestEnv` as a developer tool in a global/dev environment; do not add it
as a dependency of the package under test.

### Environment-selected aggregate suites

For a package whose `test/runtests.jl` selects a test group through an
environment variable, use TestEnv for the warm interactive loop and `Pkg.test`
for verification. The selector and its allowed values belong to the package;
`MECHANICS` is only an example. This is still a package-owned test path:
shared analyzers, formatters, documentation builds, coverage tooling, and
benchmarks stay outside `Pkg.test()` as described in `package-quality-gates.md`.

```bash
repld --session mypkg julia -e '
    using Revise, TestEnv
    Revise.revise()
    previous_group = get(ENV, "GROUP", nothing)
    try
        ENV["GROUP"] = "MECHANICS"
        TestEnv.activate("MyPackage") do
            Base.include(Main, joinpath(pkgdir(MyPackage), "test", "runtests.jl"))
        end
    finally
        isnothing(previous_group) ? delete!(ENV, "GROUP") : (ENV["GROUP"] = previous_group)
    end
'
```

The `do` form restores the prior active project but does not undo module loads,
mutable state, or environment variables. Restore the selector yourself, as
above. Use `Base.include` for the test entry point: it evaluates the runner at
top level and avoids a Julia world-age error caused by importing a newly
available test-only dependency and immediately calling it inside the callback.

Run a leaf file directly only when it documents its complete standalone harness;
TestEnv supplies dependencies, not runner imports, helpers, configuration, or
ordering.

For multi-command interactive debugging, activate TestEnv without a callback,
use test dependencies in later `repld` evaluations, and restore the prior
project explicitly:

```bash
repld --fresh --session mypkg-debug julia --project=/path/to/MyPackage -e '
    using TestEnv
    global TESTENV_RETURN_PROJECT = Base.active_project()
    TestEnv.activate("MyPackage")
'
repld --session mypkg-debug julia -e 'using TestDependencies; include("debug_case.jl")'
repld --session mypkg-debug julia -e 'import Pkg; Pkg.activate(TESTENV_RETURN_PROJECT)'
repld close --session mypkg-debug
```

This persistent form is less isolated; do not leave it active across unrelated
work. Start a fresh session after dependency, type-layout, or module-structure
changes.

Confirm an interactive green group with Pkg's real isolated test subprocess:

```bash
repld --fresh --session mypkg-group julia --project=/path/to/MyPackage -e '
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
phase's completion. Start a fresh session after environment or dependency
changes, type-layout changes, module/include reorganization, changed thread or
Julia-version assumptions, or repeated Revise/world-age symptoms.

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

### Revise Limits And Restart Rules

Revise is excellent for ordinary method-body edits, new methods, test tweaks,
and many package-source changes. Use a fresh Julia session after:

- changing struct fields or type hierarchy;
- moving files/modules or changing `include` order;
- changing dependencies or environments;
- repeated world-age, stale-method, or precompile-cache symptoms.

On Julia versions older than 1.12, Revise does not support changes to `struct`
definitions. Newer versions improve this, but a fresh session remains the safe
default after type-layout changes in serious package work.

### Revise.jl with MCP Servers

Both MCP servers integrate with Revise for hot-reloading — see `mcp-servers.md` for server-specific behavior and the full Revise limits table (when auto-reload works vs when to restart).
