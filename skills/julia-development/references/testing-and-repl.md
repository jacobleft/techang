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
    include("test/runtests.jl")
end
```

This is useful because `Pkg.test` runs tests in a temporary environment that
contains the package plus test-only dependencies from `[extras]`/`[targets]` or
`test/Project.toml`. The plain package environment often does not include those
dependencies. `TestEnv.activate()` lets you enter an equivalent environment
interactively for focused test writing and debugging.

Preferred warm-session pattern with `repld`:

```bash
repld --fresh --session mypkg julia --project=MyPackage -e 'using Revise; using MyPackage; println("ready")'
repld --session mypkg julia -e 'using Revise; Revise.revise(); import TestEnv; TestEnv.activate("MyPackage") do; include("test/runtests.jl"); end'
```

Use focused includes for tight loops:

```bash
repld --session mypkg julia -e 'using Revise; Revise.revise(); import TestEnv; TestEnv.activate("MyPackage") do; include("test/unit/foo_tests.jl"); end'
```

Keep `TestEnv` as a developer tool in a global/dev environment; do not add it
as a dependency of the package under test.

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
