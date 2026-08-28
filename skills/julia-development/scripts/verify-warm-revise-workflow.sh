#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fixture_dir="$script_dir/../fixtures/warm-revise-workflow"

for command in julia repld; do
    command -v "$command" >/dev/null || {
        printf 'missing required command: %s\n' "$command" >&2
        exit 1
    }
done

julia --startup-file=no -e 'using Revise, TestEnv; println("shared Revise and TestEnv available")'

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/warm-revise-workflow.XXXXXX")"
pkg_dir="$work_dir/WarmWorkflowFixture"
cp -R "$fixture_dir" "$pkg_dir"

session_prefix="warm-workflow-$$"
sessions=(runtime-warm test-warm test-dev pkgtest docs-warm docs-dev case-warm case-dev scratch-dev)

cleanup() {
    local session
    for session in "${sessions[@]}"; do
        repld close --session "$session_prefix-$session" >/dev/null 2>&1 || true
    done
    rm -rf "$work_dir"
}
trap cleanup EXIT INT TERM

rewrite_once() {
    local path="$1"
    local before="$2"
    local after="$3"

    julia --startup-file=no -e '
        path, before, after = ARGS
        content = read(path, String)
        count = length(split(content, before)) - 1
        count == 1 || error("expected one replacement in $path, found $count")
        write(path, replace(content, before => after; count=1))
    ' "$path" "$before" "$after"
}

echo '1/5 package runtime warm without TestEnv or Revise'
repld --fresh --session "$session_prefix-runtime-warm" julia --startup-file=no --project="$pkg_dir" -e '
    import Pkg
    Pkg.instantiate()
    using WarmWorkflowFixture
    @assert marker() == "baseline"
'
repld close --session "$session_prefix-runtime-warm"

echo '2/5 TestEnv warm, hot package/test loop, and isolated Pkg.test gate'
repld --fresh --session "$session_prefix-test-warm" julia --startup-file=no --project="$pkg_dir" -e '
    import Pkg, TestEnv
    TestEnv.activate()
    Pkg.instantiate()
    Pkg.precompile()
'
repld close --session "$session_prefix-test-warm"

repld --fresh --session "$session_prefix-test-dev" julia --startup-file=no --project="$pkg_dir" -e '
    import TestEnv
    TestEnv.activate()
    using Revise
    using WarmWorkflowFixture
    ENV["GROUP"] = "UNIT"
    Base.include(Main, joinpath(pkgdir(WarmWorkflowFixture), "test", "runtests.jl"))
'

rewrite_once "$pkg_dir/src/WarmWorkflowFixture.jl" '"baseline"' '"test-revised"'
repld --session "$session_prefix-test-dev" julia -e '
    sleep(0.25)
    Revise.revise()
    ENV["WARM_WORKFLOW_MARKER"] = "test-revised"
    @assert WarmWorkflowFixture.marker() == "test-revised"
    println("package source reload proof passed")
    Base.include(Main, joinpath(pkgdir(WarmWorkflowFixture), "test", "runtests.jl"))
'
repld close --session "$session_prefix-test-dev"

repld --fresh --session "$session_prefix-pkgtest" julia --startup-file=no --project="$pkg_dir" -e '
    import Pkg
    ENV["GROUP"] = "UNIT"
    ENV["WARM_WORKFLOW_MARKER"] = "test-revised"
    Pkg.test(; coverage=false)
'
repld close --session "$session_prefix-pkgtest"

echo '3/5 docs environment without TestEnv'
docs_dir="$pkg_dir/docs"
repld --fresh --session "$session_prefix-docs-warm" julia --startup-file=no --project="$docs_dir" -e '
    import Pkg
    Pkg.instantiate()
    Pkg.precompile()
'
repld close --session "$session_prefix-docs-warm"

repld --fresh --session "$session_prefix-docs-dev" julia --startup-file=no --project="$docs_dir" -e '
    using Revise
    using WarmWorkflowFixture
    docs_dir = joinpath(pkgdir(WarmWorkflowFixture), "docs")
    ENV["WARM_WORKFLOW_MARKER"] = "test-revised"
    include(joinpath(docs_dir, "make.jl"))
'
rewrite_once "$pkg_dir/src/WarmWorkflowFixture.jl" '"test-revised"' '"docs-revised"'
repld --session "$session_prefix-docs-dev" julia -e '
    sleep(0.25)
    Revise.revise()
    docs_dir = joinpath(pkgdir(WarmWorkflowFixture), "docs")
    ENV["WARM_WORKFLOW_MARKER"] = "docs-revised"
    @assert WarmWorkflowFixture.marker() == "docs-revised"
    println("docs source reload proof passed")
    include(joinpath(docs_dir, "make.jl"))
'
repld close --session "$session_prefix-docs-dev"

echo '4/5 human-facing case environment without TestEnv'
case_dir="$pkg_dir/examples/cases/case1"
repld --fresh --session "$session_prefix-case-warm" julia --startup-file=no --project="$case_dir" -e '
    import Pkg
    Pkg.instantiate()
    Pkg.precompile()
'
repld close --session "$session_prefix-case-warm"

repld --fresh --session "$session_prefix-case-dev" julia --startup-file=no --project="$case_dir" -e '
    using Revise
    using WarmWorkflowFixture
    case_dir = joinpath(pkgdir(WarmWorkflowFixture), "examples", "cases", "case1")
    includet(joinpath(case_dir, "definitions.jl"))
    ENV["WARM_WORKFLOW_MARKER"] = "docs-revised"
    ENV["WARM_WORKFLOW_CASE"] = "case-v1"
    include(joinpath(case_dir, "run.jl"))
'
rewrite_once "$case_dir/definitions.jl" '"case-v1"' '"case-v2"'
repld --session "$session_prefix-case-dev" julia -e '
    sleep(0.25)
    Revise.revise()
    case_dir = joinpath(pkgdir(WarmWorkflowFixture), "examples", "cases", "case1")
    ENV["WARM_WORKFLOW_CASE"] = "case-v2"
    @assert case_label() == "case-v2"
    println("case includet reload proof passed")
    include(joinpath(case_dir, "run.jl"))
'
repld close --session "$session_prefix-case-dev"

echo '5/5 process-local @temp environment without TestEnv'
scratch_dir="$pkg_dir/scratch"
pkg_dir_literal="$(julia --startup-file=no -e 'print(repr(ARGS[1]))' "$pkg_dir")"
repld --fresh --session "$session_prefix-scratch-dev" julia --startup-file=no --project=@temp -e "
    import Pkg
    Pkg.develop(path=$pkg_dir_literal)
    Pkg.instantiate()
    Pkg.precompile()
"
repld --session "$session_prefix-scratch-dev" julia -e '
    using Revise
    using WarmWorkflowFixture
    scratch_dir = joinpath(pkgdir(WarmWorkflowFixture), "scratch")
    includet(joinpath(scratch_dir, "definitions.jl"))
    ENV["WARM_WORKFLOW_MARKER"] = "docs-revised"
    ENV["WARM_WORKFLOW_SCRATCH"] = "scratch-v1"
    include(joinpath(scratch_dir, "run.jl"))
'
rewrite_once "$scratch_dir/definitions.jl" '"scratch-v1"' '"scratch-v2"'
repld --session "$session_prefix-scratch-dev" julia -e '
    sleep(0.25)
    Revise.revise()
    scratch_dir = joinpath(pkgdir(WarmWorkflowFixture), "scratch")
    ENV["WARM_WORKFLOW_SCRATCH"] = "scratch-v2"
    @assert scratch_label() == "scratch-v2"
    println("scratch includet reload proof passed")
    include(joinpath(scratch_dir, "run.jl"))
'
repld close --session "$session_prefix-scratch-dev"

echo 'warm Revise/TestEnv workflow fixture passed'
