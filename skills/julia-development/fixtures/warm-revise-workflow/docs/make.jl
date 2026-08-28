using WarmWorkflowFixture

expected = get(ENV, "WARM_WORKFLOW_MARKER", "baseline")
@assert marker() == expected
println("docs workflow passed: ", expected)
