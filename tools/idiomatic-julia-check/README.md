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

Check one or more notes:

```sh
tools/idiomatic-julia-check/target/release/idiomatic-julia-check design/IdiomaticJulia.jl
```

A valid note has one wrapper:

```julia
const DESIGN = quote
    SpecificNoun <: GeneralNoun
    result::ResultNoun = verb(a::NounA, b::NounB)
    result = verb(a, b)
    mutate!(a, b)
end
```

Supported control flow is `if`/`elseif`/`else`, `for`, `while`, `break`, and `continue`. Conditions and iteration sources are noun names, booleans, or verb calls.

Arbitrary expressions, nested calls, property and index access, qualified callees, macros, definitions, keyword arguments, and assignment from `verb!` are rejected.

Exit status is `0` when every input is valid, `1` for an invalid or unreadable input, and `2` when no path is supplied. Diagnostics use `path:line:column: message`.
