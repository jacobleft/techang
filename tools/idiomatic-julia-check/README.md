# idiomatic-julia-check

`idiomatic-julia-check` validates a deliberately small Julia-shaped design notation. It is a compiled Rust executable backed by Fatou's parser. It parses the file but never evaluates it.

The tool requires and pins Rust 1.98.1.

Build it once from the repository root:

```sh
cargo +1.98.1 build --release --manifest-path tools/idiomatic-julia-check/Cargo.toml
```

Or install the command on `PATH`:

```sh
cargo +1.98.1 install --locked --path tools/idiomatic-julia-check
```

From a Julia package root, check its concept-only quoted Julia file:

```sh
idiomatic-julia-check docs/design/IdiomaticJulia.jl
```

The standardized path is `<package-root>/docs/design/IdiomaticJulia.jl`, where `<package-root>` is a placeholder for the package's actual root directory.

A valid note contains one or more quoted blocks. A block may be bare or labeled with any const name, and line or block comments may appear anywhere:

```julia
# Noun relationships
const NOUNS = quote
    SpecificNoun <: GeneralNoun
end

# Verb declarations
const VERBS = quote
    result::ResultNoun = verb(a::NounA, b::NounB)
    mutate!(a, b)
end

# Representative composition
quote
    result = verb(a, b)
    mutate!(a, b)
end
```

Every non-comment top-level item must be either `quote ... end` or `const NAME = quote ... end`. Each quoted body is checked independently against the same restricted notation.

Supported control flow is `if`/`elseif`/`else`, `for`, `while`, `break`, and `continue`. Conditions and iteration sources are noun names, booleans, or verb calls.

Arbitrary expressions, nested calls, property and index access, qualified callees, macros, definitions, keyword arguments, and assignment from `verb!` are rejected.

Exit status is `0` when every input is valid, `1` for an invalid or unreadable input, and `2` when no path is supplied. Diagnostics use `path:line:column: message`.
