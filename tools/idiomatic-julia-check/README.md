# idiomatic-julia-check

`idiomatic-julia-check` is a semantic gate for a deliberately small Julia-shaped design note. It validates the note's verb-and-noun notation, builds its planned dispatch surface, and checks requirements against that surface and Manifest-bound dependencies. It is a compiled Rust executable backed by Fatou's parser. It parses source but never evaluates it or starts Julia.

The tool requires and pins Rust 1.98.1.

Build it once from the repository root:

```sh
cargo +1.98.1 build --release --manifest-path tools/idiomatic-julia-check/Cargo.toml
```

Or install the command on `PATH`:

```sh
cargo +1.98.1 install --locked --path tools/idiomatic-julia-check
```

From a Julia package root, check its concept-only quoted Julia file during design:

```sh
idiomatic-julia-check --semantic . docs/design/IdiomaticJulia.jl
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
    mutate!(a::NounA, b::NounB)
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

## Design-time semantic gate

The semantic gate reads declarations and requirements differently:

- A typed standalone call or long-form function signature declares a planned method.
- An assignment whose right side is a call requires that call signature.
- A value-only call requires a planned callable with the same argument shape.
- A subtype line adds a relationship used when checking whether a broader method covers a requirement.

Package-owned declarations form a virtual dispatch surface directly from the note. The checker does not read the package's `src` directory, so a new package can pass before any implementation exists. A qualified declaration for a dependency represents a planned extension and requires the callable or type binding to exist in that dependency.

Qualified dependency requirements are compared with statically declared methods in dependency source. The dependency must be direct in `Project.toml` and have a matching `Manifest.toml` record. Path dependencies use the recorded path. Registry and Git dependencies use the Manifest UUID and `git-tree-sha1` to locate the exact Julia depot version-slug directory; another installed version is never substituted. Only dependencies referenced by the design note are indexed.

The gate reports:

- `planned`: the note declares a valid package method or dependency extension.
- `exact`: a requirement has the same signature as a planned or dependency method.
- `covered`: a broader method covers the required types, or a value-only call has a matching callable and argument shape.
- `missing`: the planned surface or pinned dependency has no compatible signature or binding.
- `unknown`: static source cannot decide because of unavailable pinned source, macros, `eval`, package extensions, complex `where` constraints, generated constructors, or unresolved types.

`planned`, `exact`, and `covered` succeed. `missing` and `unknown` make the command exit with status `1`. This result validates the internal semantics of the design note and its assumptions about pinned dependencies; it does not report whether a later implementation conforms to the note.

Exit status is `0` when every input is valid and every semantic requirement is confirmed, `1` for invalid, unreadable, missing, or unknown results, and `2` for invalid command usage. Diagnostics use `path:line:column: message`.
