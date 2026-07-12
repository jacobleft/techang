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

```julia
# @inbounds — skip bounds checking when safe
@inbounds for i in 1:length(a)
    a[i] = b[i] + c[i]
end

# @simd — hint for SIMD vectorization
@inbounds @simd for i in eachindex(a)
    s += a[i]
end
```
