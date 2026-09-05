# Idiomatic Julia Check

This is a small design notation for Julia code shaped around multiple dispatch. It records the important verbs, noun type relationships, and a few representative compositions.

## Agent rule

> State the design as `verb(a, b, c...)`. Verbs say what the program does. Nouns are the values they act on. Use noun types where meaningful differences should select behavior. Use `result = verb(...)` for a returned value and `verb!(...)` for mutation. Keep the note small.

Method bodies remain ordinary Julia.

## Note format

Use a `.jl` file containing one quoted expression:

```julia
const DESIGN = quote
    SpecificNoun <: GeneralNoun

    result::ResultNoun = verb(a::NounA, b::NounB)
    mutate!(a::NounA, b::NounB)

    result = verb(a, b)
    mutate!(a, b)

    if condition(a)
        result = verb(a, b)
    else
        fallback!(a)
    end

    for item in items(a)
        mutate!(item, b)
    end
end
```

The quote keeps the note valid Julia syntax without resolving types or executing calls. Fatou can format and lint the `.jl` file.

The checker permits:

```julia
SpecificNoun <: GeneralNoun
verb(a::NounA, b::NounB)
result::ResultNoun = verb(a::NounA, b::NounB)
result = verb(a, b)
verb!(a, b)
```

It also permits `if`/`elseif`/`else`, `for`, `while`, `break`, and `continue`. Conditions and iteration sources are noun names, booleans, or verb calls. Other Julia constructs are rejected.

## Checker

Build and install the Rust executable from the `techang` repository:

```sh
cargo install --path tools/idiomatic-julia-check
```

Then check and format the note:

```sh
idiomatic-julia-check design/IdiomaticJulia.jl
fatou format design/IdiomaticJulia.jl
fatou lint design/IdiomaticJulia.jl
```

The checker uses Fatou's parser and does not start Julia or maintain a second Julia grammar.

## Scope

Create or update the note when a change introduces or renames important verbs or noun types, or changes how they compose. Do not enumerate every function, method, field, or helper. After validation, inspect the corresponding generics, types, dependencies, `public` declarations, exports, and concrete restrictions in the implementation.
