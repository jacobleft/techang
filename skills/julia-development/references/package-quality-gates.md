# Package Quality Gates

Use this reference when the main Julia skill asks for package-level
verification. Keep the main skill short; put detailed gate selection here.

## Core Rules

- `PkgTemplates` is for creating new Julia packages or app scaffolds. It is not
  a QA tool for existing packages.
- `Aqua` and `SafeTestsets` are package-owned test infrastructure. When the
  package uses them, declare them as test-only `[extras]` and in the `test`
  target, never in runtime `[deps]`.
- Do not add `ExplicitImports`, `JET`, `JETLS`, `JuliaFormatter`,
  `Runic`, `BenchmarkTools`, `Documenter`, or coverage tooling to the
  package just because this reference names them.
- Install analyzer/formatter tools once in Julia's default shared environment
  for the active version, such as `@v1.12`, or use a temporary environment.
  They remain outside the package under development.
- Run existing project-owned gates when they are already configured, but do not
  modify the target package's dependency graph to satisfy an agent-side check.
- Tests decide behavior. Static analysis and quality checks are review signals
  to investigate.
- Do not claim CI, coverage, docs, or benchmark success without seeing the
  result.

## Gate Matrix

| Change surface | Gate | Command shape | Notes |
|----------------|------|---------------|-------|
| Ordinary package change | Test suite | Package-selected test group or file | Do not default to a full suite. Use `Pkg.test()` only for the package-owned functional/test gate selected by project policy, from a fresh session with the package project active and never from TestEnv. |
| Package maintenance | Aqua | `Aqua.test_all(MyPkg)` from the package test target | Aqua is a test-only dependency. It may run through a scoped `Pkg.test()` or `TestEnv.activate` test path when the package owns that gate. |
| Test structure | SafeTestsets | `@safetestset` in package tests | SafeTestsets is a test-only dependency in the package test target, not shared analysis tooling. |
| Import or namespace cleanup | ExplicitImports | `ExplicitImports.print_explicit_imports(MyPkg)` or project test integration | Run from the default shared env or temp env unless project-owned tests already cover it. Use for explicit import hygiene, unused imports, and qualified-access cleanup. |
| Static type analysis | JET or JETLS | `JET.test_package(MyPkg; target_modules=(MyPkg,))` or JETLS diagnostics | Run separately from the default shared env or a temp env. Never invoke JET through `Pkg.test()`. Treat findings as signals; reduce or triage false positives instead of blindly rewriting code. |
| Formatting | Runic or JuliaFormatter | Project formatter command/config | Run separately from shared tooling. Never invoke formatting through `Pkg.test()`, and do not add formatter deps to the target package for agent-side formatting. |
| Documentation | Documenter draft build | `DOCUMENTER_DRAFT=true julia --project=docs docs/make.jl` | Use the project's docs env when present. Do not add Documenter to the target package just to build docs. |
| Coverage and CI | Project CI equivalent | Existing `make`, `just`, `julia --project`, or hosted CI commands | Use when touching tests, CI, docs, package metadata, coverage setup, or public APIs. Report unavailable or unrun CI plainly. |
| Performance-sensitive work | BenchmarkTools/Profile/project benchmarks | `BenchmarkTools.@btime`, `@benchmark`, `Profile`, or package benchmark suite | Use project benchmark env when present, otherwise default shared env or temp env. Only benchmark real performance work. `@time` is a smell test, not proof. |

## Analyzer Environments

For ad hoc ExplicitImports, JET, formatter, or benchmark checks, first use the
default shared environment for the active Julia version. This is the normal
`@v#.#` environment on `LOAD_PATH`, for example
`~/.julia/environments/v1.12` on Julia 1.12. Aqua instead belongs in the
package test target with SafeTestsets when the package owns an Aqua gate.

Run an Aqua-only test file through `TestEnv.activate`, or use `Pkg.test()` only
when the project provides a selected test-only gate that contains functional
tests, Aqua, and SafeTestsets but no shared analyzer or formatter. Do not treat
a generic `QUALITY` group as safe without inspecting its contents.

Do not create a separate shared analyzer environment just to keep tools out of
the target package. If the default shared env lacks an analyzer or formatter,
prefer a temp env. Use another shared env only when the user or project already
provides one.

```julia
# Temp env: installs analyzer tooling into a throwaway active project.
using Pkg
target = dirname(Base.active_project())  # run Julia with --project=/path/to/pkg
Pkg.activate(; temp=true)                # active project becomes a temp Project.toml
Pkg.develop(path=target)                 # make the target package available by path
Pkg.add(["ExplicitImports", "JET", "Runic"])
using MyPkg, ExplicitImports, JET
ExplicitImports.print_explicit_imports(MyPkg)
JET.test_package(MyPkg; target_modules=(MyPkg,))
```

In the temp-env path, `Pkg.add` mutates the temporary Project/Manifest, not the
target package's Project/Manifest. It may still populate the user's depot cache.
Never add analyzers or formatters while the target package project is active
solely to satisfy agent QA.

## Suggested Staged Gate

1. Focused package test groups/files through `repld` plus `TestEnv.activate`
   while iterating.
2. Run a project-selected `Pkg.test()` gate only from a fresh session with the
   package project active, never from TestEnv or a Revise warm loop. It may
   contain functional tests and package-owned test-only dependencies such as
   Aqua/SafeTestsets, but it must not invoke JET/JETLS, ExplicitImports,
   Runic/JuliaFormatter, Documenter, benchmarks, or other shared tooling.
3. Run ExplicitImports, JET/JETLS, formatting, and docs draft checks separately
   from shared or temporary tooling environments, never through `Pkg.test()`.
4. Run CI or coverage checks when the change touches CI, tests, docs, package
   metadata, or public APIs.
5. Run BenchmarkTools/Profile only for performance claims or performance-risky
   changes, with stable inputs and reported units.

## Scaffolding, Not QA

Use `PkgTemplates` only when the user asks to create or modernize a package
scaffold. It can set up CI, Documenter, tests, coverage, licenses, and standard
files for a new package, but those generated files are not themselves proof of
quality. After scaffolding, still run the relevant gates above.

Do not retrofit an existing repository with `PkgTemplates` unless the user
explicitly wants a scaffolding migration and accepts the file churn.

## References

- [JET.jl](https://aviatesk.github.io/JET.jl/dev/jetanalysis/)
- [Tim Holy's Claude config](https://github.com/timholy/claude_config)
