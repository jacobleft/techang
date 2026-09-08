# Idiomatic Julia Check

This is a small design notation for Julia code shaped around multiple dispatch. It records the important verbs, noun type relationships, and a few representative compositions.

## Agent rule

> State the design as `verb(a, b, c...)`. Verbs say what the program does. Nouns are the values they act on. Use noun types where meaningful differences should select behavior. Use `result = verb(...)` for a returned value and `verb!(...)` for mutation. Keep the note small.

Method bodies remain ordinary Julia.

## Note format

Every Julia package uses one canonical path for concept-only quoted Julia. Here, `<package-root>` is a placeholder for the package's actual root directory:

```text
<package-root>/docs/design/IdiomaticJulia.jl
```

Do not place these concept notes under `src/`, `test/`, the package root, or another documentation directory. The file contains one or more quoted blocks. Comments may appear between or inside them:

```julia
# Noun relationships
const NOUNS = quote
    SpecificNoun <: GeneralNoun
end

# Verb declarations
const VERBS = quote
    verb(a::NounA, b::NounB)::ResultNoun
    mutate!(a::NounA, b::NounB)
end

# Representative composition
quote
    result::ResultNoun = verb(a::NounA, b::NounB)
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

Each top-level item must be either `quote ... end` or `const NAME = quote ... end`; a file may mix both forms. Line comments and `#= ... =#` block comments are allowed anywhere. The quotes keep the note valid Julia syntax without resolving types or executing calls. Fatou can format and lint the `.jl` file.

The checker permits:

```julia
SpecificNoun <: GeneralNoun
verb(a::NounA, b::NounB)
result::ResultNoun = verb(a::NounA, b::NounB)
result = verb(a, b)
verb!(a, b)
```

It also permits `if`/`elseif`/`else`, `for`, `while`, `break`, and `continue`. Conditions and iteration sources are noun values, including dot access, booleans, or verb calls. Other note-level constructs are rejected unless they occur inside an algorithm block described below.

Dot access is part of the checked notation for noun values, callables, and type references. This covers object fields and chains such as `body.cache.d`; module qualification is one use of the same syntax:

```julia
PackageName.AbstractNoun
PackageName.ConcreteNoun(items)
PackageName.verb!(object.state, context.cache.value)
result::PackageName.ResultNoun = PackageName.verb(input::PackageName.InputNoun)
```

The notation also accepts return annotations, mixed typed and value arguments, keyword arguments, owned-field annotations, tuple loop bindings, and mutation assignments:

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

A long-form function definition is an algorithm block:

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

Fatou checks the syntax of the whole algorithm block. Its body is ordinary Julia and is not restricted to the note-level grammar. The restricted grammar continues to apply outside function definitions.

## Checker

Build and install the Rust executable from the `techang` repository:

```sh
cargo +1.98.1 install --locked --path tools/idiomatic-julia-check
```

Then check the note's notation and semantics, and format it:

```sh
# Run from the actual package root.
idiomatic-julia-check --semantic . docs/design/IdiomaticJulia.jl
fatou format docs/design/IdiomaticJulia.jl
fatou lint docs/design/IdiomaticJulia.jl
```

The checker uses Fatou's parser and does not start Julia or maintain a second Julia grammar.

Run this gate during design, before implementation. Typed standalone calls and long-form signatures declare planned methods. Assignments whose right side is a call are signature requirements. Fully value-only calls require a planned callable with the same argument shape. Subtype lines provide the hierarchy used for coverage checks.

Package-owned declarations build a virtual dispatch surface from the note itself. The package's `src` directory is never consulted. A qualified dependency declaration is a planned extension and must name an existing callable or type binding. A qualified dependency requirement must match or be covered by a statically declared method.

Dependency references are accepted only when the dependency is direct in `Project.toml` and pinned in `Manifest.toml`. Path dependencies resolve from their recorded path. Registry and Git dependencies resolve through the Manifest UUID and `git-tree-sha1`, which identify the exact Julia depot version-slug directory. The checker never substitutes a different installed version.

The gate reports `planned`, `exact`, `covered`, `missing`, or `unknown`. `planned`, `exact`, and `covered` pass. `missing` and `unknown` produce exit status `1`. Macro-generated methods, `eval`, package extensions, complex `where` constraints, generated constructors, and unavailable dependency source remain `unknown`. This is a semantic check of the design note and its dependency assumptions, not a post-implementation API conformance report.

## Scope

During package design, create or update `docs/design/IdiomaticJulia.jl` relative to the actual package root when a change introduces or renames important verbs or noun types, or changes how they compose. Run the semantic gate before implementation. Do not enumerate every function, method, field, or helper. After validation, implement the corresponding generics, types, dependencies, `public` declarations, exports, and concrete restrictions.
