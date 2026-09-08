# idiomatic-julia-check

`idiomatic-julia-check` validates a deliberately small Julia-shaped design notation and can statically compare its typed function signatures with package source. It is a compiled Rust executable backed by Fatou's parser. It parses source but never evaluates it or starts Julia.

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

After implementing the design, report static API compatibility:

```sh
idiomatic-julia-check --api-report . docs/design/IdiomaticJulia.jl
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

## Static API report

The API report extracts typed call declarations and long-form function signatures from the design note, then indexes method definitions in the package's `src/**/*.jl` files. It compares callable ownership, positional arity and types, varargs, keyword names, keyword types and default presence, explicit return annotations, and subtype coverage.

The report uses four results:

- `exact`: the source declares the same signature.
- `covered`: a broader declared method accepts the required typed arguments.
- `missing`: no compatible declared method exists, or the qualified module is not the package or a direct dependency.
- `unknown`: macros, `where` constraints, generated constructors, unresolved types, missing return annotations, or unavailable pinned source prevent a static conclusion.

`exact` and `covered` succeed. `missing` and `unknown` make the command exit with status `1` so an unproven signature cannot pass as compatible.

Dependency lookup is environment-bound. A qualified dependency must be listed in the package's `[deps]` and have a matching entry in `Manifest.toml`. Path dependencies use the recorded path. Registry and Git dependencies use the Manifest UUID and `git-tree-sha1` to locate Julia's exact version-slug directory under `JULIA_DEPOT_PATH`; the checker does not substitute another installed version. Only dependencies referenced by the design note are indexed.

This is a static source report. It does not claim compatibility for methods created by macros, `eval`, package extensions, or other runtime generation.

Exit status is `0` when every input is valid and every requested API signature is confirmed, `1` for invalid, unreadable, missing, or unknown results, and `2` for invalid command usage. Diagnostics use `path:line:column: message`.
