using WarmWorkflowFixture

@assert marker() == get(ENV, "WARM_WORKFLOW_MARKER", "baseline")
@assert scratch_label() == get(ENV, "WARM_WORKFLOW_SCRATCH", "scratch-v1")
println("temporary workflow passed: ", scratch_label())
