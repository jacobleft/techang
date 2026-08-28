using Test
using WarmWorkflowFixture

let group = get(ENV, "GROUP", "ALL")
    group in ("ALL", "UNIT") || error("unsupported fixture test group: $group")

    @test marker() == get(ENV, "WARM_WORKFLOW_MARKER", "baseline")
    @test 2 + 2 == 4
end
