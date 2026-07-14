# Type System & Performance

Detailed reference for Julia's type system and performance optimization idioms.

---

## Type System

### Abstract vs Concrete Types

Use abstract types to organize hierarchies. Define concrete types for instantiation — they are final (no subtypes).

```julia
abstract type Number end
abstract type Real <: Number end

struct Point{T}
    x::T
    y::T
end
```

### Designing Type Hierarchies

Good Julia hierarchies are usually shallow and behavioral. Add an abstract type
when downstream code is expected to add new concrete implementations, or when
several concrete implementations share a documented interface. Put behavior in
generic functions, not inside the type.

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

Prefer typed selectors over `Symbol` branches when a public option changes
algorithmic behavior:

```julia
abstract type SolverMode end
struct DirectMode <: SolverMode end
struct IterativeMode <: SolverMode end

solve(problem, ::DirectMode) = direct_solve(problem)
solve(problem, ::IterativeMode) = iterative_solve(problem)
```

Keep `Symbol` or `String` options at the boundary only when they improve user
ergonomics or preserve compatibility:

```julia
function solve(problem; mode::Symbol=:direct)
    selected = mode === :direct ? DirectMode() :
               mode === :iterative ? IterativeMode() :
               throw(ArgumentError("unsupported mode: $mode"))
    return solve(problem, selected)
end
```

Use traits when the dispatch axis is a capability, not a natural type identity:

```julia
supports_streaming(::Type) = Val(false)
supports_streaming(::Type{<:Dataset{<:Any,OnDisk}}) = Val(true)

stream(data) = stream(data, supports_streaming(typeof(data)))
stream(data, ::Val{false}) = collect(load(data))
stream(data, ::Val{true}) = open_stream(data)
```

### When to Annotate Types

Let inference work first. Annotate for documentation, method dispatch, or performance when inference fails.

```julia
# GOOD: Generic — works for any numeric type
add_one(x) = x + oneunit(x)

# GOOD: Abstract type for dispatch
process(x::Number) = ...

# BAD: Overly specific — no performance gain
process(x::Float64) = ...
```

Prefer abstract interfaces in signatures and concrete or parametric storage in
fields:

```julia
# Too narrow as an API
normalize(x::Vector{Float64}) = x ./ sum(x)

# Better: accepts views, StaticArrays, custom vectors, and AD/unitful numbers
normalize(x::AbstractVector) = x ./ sum(x)

# Bad for performance-sensitive storage
struct BadContainer
    value::Number
end

# Better
struct Container{T<:Number}
    value::T
end
```

Remember that Julia dispatches on positional arguments, not keyword arguments.
If a behavioral choice needs dispatch, pass a typed selector positionally or
build the selector in a thin keyword wrapper.

### Constructors As The Object-Building API

Use constructors as the normal API for making instances. Prefer outer
constructors for alternate input forms, default values, conversions, and
keyword ergonomics.

```julia
struct Config{T<:Real}
    threshold::T
    name::String
end

Config(threshold::Real) = Config(threshold, "default")
Config(; threshold=1.0, name="default") = Config(threshold, name)
Config(pair::Pair{<:AbstractString,<:Real}) = Config(pair.second, pair.first)
```

Use inner constructors only when the type has invariants that every instance
must satisfy:

```julia
struct NonemptyName
    value::String

    function NonemptyName(value::AbstractString)
        isempty(value) && throw(ArgumentError("name cannot be empty"))
        return new(String(value))
    end
end
```

Avoid redundant constructor-like helpers:

```julia
# Usually unnecessary
make_config(x) = Config(x)
create_config(; threshold=1.0) = Config(; threshold)

# Prefer
Config(x)
Config(; threshold=1.0)
```

Use a named function instead of a constructor when the operation is not just
object creation: reading files, discovering environment state, opening
resources, mutating existing objects, caching, launching processes, or running a
multi-step workflow.

### Type Stability

Ensure functions return the same type regardless of input values. Type instability is a major performance killer.

```julia
# BAD: Returns Int or Float64 depending on value
function pos_or_zero(x)
    x > 0 ? x : 0
end

# GOOD: Always returns Float64
function pos_or_zero(x)::Float64
    x > 0 ? Float64(x) : 0.0
end
```

### Val Types for Compile-Time Dispatch

Use `Val` types for zero-overhead dispatch on compile-time values.

```julia
algorithm(x, ::Val{1}) = simple_version(x)
algorithm(x, ::Val{2}) = complex_version(x)

algorithm(data, Val(1))  # Dispatch resolved at compile time
```

Use `Val` sparingly. It is useful when the value is already known at compile
time or comes from a type/trait. Do not turn arbitrary runtime values into
`Val` just to look "dispatchy"; that can increase method count and compile time
without improving design.

---

## Performance Idioms

### Broadcasting Fusion

Use `@.` to fuse broadcast operations into a single loop, avoiding intermediate allocations.

```julia
# BAD: Three separate allocations
result = 3*x.^2 + 4*x + 7*x.^3

# GOOD: Fused into single loop
result = @. 3x^2 + 4x + 7x^3
```

### Avoid Globals

Non-const global variables break type inference. Pass data as arguments or use `const`.

```julia
# BAD: Global kills performance
x = rand(1000)
sum_global() = sum(x)

# GOOD: Pass as argument
sum_arg(x) = sum(x)

# If global needed, use const
const DEFAULT_SIZE = 100
```

### In-Place Operations

Functions modifying arguments end with `!`:

```julia
sort(arr)    # Returns new sorted array
sort!(arr)   # Sorts in-place
```

### Loop Optimization

Write clear loops with `eachindex`, avoid unnecessary allocations, and profile
before changing code for speed. In ordinary package code, keep the loop easy to
read unless profiling identifies a real bottleneck.

```julia
for i in eachindex(a, b, c)
    a[i] = b[i] + c[i]
end
```

### Macro Use

Macros are powerful but should stay purposeful. Prefer ordinary functions unless
the macro needs syntax access, source-location information, generated code, test
reporting, diagnostics, or benchmarking support.

Common macros:

- `@assert`: internal invariants and debugging checks. Do not rely on it as the
  only recoverable user-input validation; use `throw(ArgumentError(...))` or a
  domain-specific error when callers should handle the failure.
- `@views`: turn slices into views when avoiding copies is intended.
- `@.`: fuse broadcasted operations; avoid it when it hides which operations are
  scalar vs vectorized.
- `@time`, `@allocated`, `@code_warntype`: quick diagnostics. For serious
  benchmarks, prefer `BenchmarkTools.@btime`.
