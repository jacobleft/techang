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

### Revise.jl with MCP Servers

Both MCP servers integrate with Revise for hot-reloading — see `mcp-servers.md` for server-specific behavior and the full Revise limits table (when auto-reload works vs when to restart).
