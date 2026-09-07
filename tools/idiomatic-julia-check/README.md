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
    verb(a::NounA, b; option::OptionNoun = default)::ResultNoun
    mutate!(a, b)
end

# Representative composition
quote
    result = verb(a, b)
    mutate!(a, b)
end
```

Every non-comment top-level item must be either `quote ... end` or `const NAME = quote ... end`. Each quoted body is checked independently.

Dot access is accepted for noun values, callables, and type references. Module qualification is one use of the same syntax:

```julia
PackageName.AbstractNoun
PackageName.ConcreteNoun(items)
PackageName.verb!(object.state, context.cache.value)
result::PackageName.ResultNoun = PackageName.verb(input::PackageName.InputNoun)
```

Return annotations, mixed typed and value arguments, keywords, owned fields, tuple loop bindings, and mutation assignments are accepted:

```julia
verb(a::NounA, b; option::OptionNoun = default)::ResultNoun
result = verb(a::NounA, b; option = settings.option, mode = :fast)::ResultNoun
owner.field::FieldNoun

for (key, value) in pairs(source)
    update!(destination, key, value; mode = settings.mode)
end

destination .= source.values
owner.field += increment
```

Long-form function definitions are accepted as algorithm blocks:

```julia
function transform!(destination, workspace, item::ConcreteNoun{Variant})
    combine!(
        destination,
        workspace.intermediate,
        workspace.parameters,
        item.data,
    )
end
```

Fatou checks the syntax of an algorithm block, while `idiomatic-julia-check` leaves its body as ordinary Julia. The restricted notation still applies to statements outside function definitions.

Supported control flow is `if`/`elseif`/`else`, `for`, `while`, `break`, and `continue`. Conditions and iteration sources are noun values, including dot access, booleans, or verb calls.

Outside function definitions, arbitrary expressions, nested calls, index arguments, macros, type definitions, unsupported assignment operators, and binding a result from `verb!` with plain `=` are rejected.

Exit status is `0` when every input is valid, `1` for an invalid or unreadable input, and `2` when no path is supplied. Diagnostics use `path:line:column: message`.
