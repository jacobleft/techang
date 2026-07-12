# OOP to Julia Migration Guide

Common mistakes when transitioning from OOP languages to Julia, with fixes.

---

## 1. Methods "Inside" Types

Julia does not support methods inside `struct`. Define external functions with type annotations instead.

```julia
# WRONG: Trying to make OOP-style methods
struct MyType
    x::Int
    # Can't define methods here
end

# RIGHT: External functions with dispatch
my_method(data::MyType) = data.x * 2
```

---

## 2. Overly Specific Types

Don't restrict function signatures to concrete types without reason. Julia specializes at call time for any argument types.

```julia
# WRONG: No flexibility gain
add(x::Int64, y::Int64) = x + y

# RIGHT: Let Julia specialize
add(x, y) = x + y
```

---

## 3. Type Piracy

Avoid type piracy: do not extend functions you don't own on types you don't own. Extending Base or package functions for your own types is normal Julia style. Piracy is not literally undefined behavior, but it can cause surprising method conflicts and load-order-dependent behavior.

```julia
# WRONG: Extending Base.+ for types you don't own
import Base: +
+(a::String, b::String) = a * b  # Don't do this!

# RIGHT: Define your own function
concat(a::String, b::String) = a * b

# ALSO RIGHT: Extending Base.show for your own type
struct Widget
    name::String
end
Base.show(io::IO, w::Widget) = print(io, "Widget(", w.name, ")")
```

---

## 4. Mutable When Immutable Works

Default to immutable `struct`. Use `mutable struct` only when mutation is required. Immutable types are faster and thread-safe.

```julia
# WRONG: Unnecessary mutability
mutable struct Point
    x::Float64
    y::Float64
end

# RIGHT: Default to immutable
struct Point
    x::Float64
    y::Float64
end
```

---

## 5. Inheritance Instead of Composition

Julia concrete types are final — no subtyping. Use composition and abstract types for hierarchy.

```julia
# WRONG: Trying to inherit from concrete type
struct Dog <: Animal  # Only works if Animal is abstract
    name::String
end

# RIGHT: Use abstract types for hierarchy, composition for data
abstract type Animal end
struct Dog <: Animal
    name::String
    traits::TraitSet  # Composition
end
```
