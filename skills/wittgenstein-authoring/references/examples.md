# Examples

## Case: forbidden frame / 此地无银

User says:

> Do not make this sound like perching. The old draft is too defensive.

Bad agent behavior:

> This is not a perching mechanism, but a compliant contact strategy.

Correct action:

1. Route to `references/subskills/contrast-revelation-audit.md`.
2. Treat `perching` as a ghost frame.
3. Remove direct mentions, denial traces, and contrast around the ghost frame.
4. Write the positive mechanism.

Good output:

> The mechanism enables compliant contact with slender structures through controllable stiffness and load-transfer paths.

Chinese bad:

> 本文并非模仿鸟类栖附行为，而是研究张拉整体结构的接触稳定机制。

Chinese good:

> 本文研究螺线张拉整体单元在复杂接触条件下的构型稳定、刚度调控与力传递机制。

## Case: term discipline

User says:

> The paragraph keeps saying framework, method, model, and system. Make the terms stable.

Bad prose:

> The framework constructs a model through a system-level method and then forms a pipeline for evaluation.

Correct action:

1. Route to `references/subskills/term-and-naming-discipline-audit.md`.
2. Build a minimal term graph.
3. Assign roles: method = procedure; model = representation/equations; system = physical/software object only if needed.
4. Delete decorative names.

Good output:

> The method first constructs the mechanical model and then evaluates its stability under the prescribed loading conditions.

## Case: structural skeleton

User says:

> Analyze the structure, not the content. The logic jumps.

Bad skeleton:

```text
1. announces broad contribution
2. introduces mechanism without definition
3. states results
4. returns to motivation
5. defines term used in paragraph 2
```

Correct action:

1. Route to `references/subskills/structural-skeleton-audit.md`.
2. Label each paragraph by function.
3. Identify missing bridges and premature abstractions.
4. Propose reordering without arguing about factual truth.

Good skeleton:

```text
1. motivation/problem pressure
2. technical gap
3. definition
4. mechanism
5. result/evidence
6. bounded contribution
```

## Case: exposition boundary

User says:

> This conclusion feels too large. Enforce silence.

Overclaim:

> This approach establishes a new paradigm for aerial manipulation.

Correct action:

1. Route to `references/subskills/exposition-boundary-audit.md`.
2. Build a claim boundary table.
3. Narrow claims to what the text can support.

Bounded output:

> This approach provides a lightweight structural route for contact and load transfer in low-altitude UAV manipulation scenarios.

Unsupported completeness:

> The system is robust in complex environments.

Bounded output:

> The prototype tests characterize contact stability under the tested load and geometry ranges.

## Case: source to reader-facing prose

User says:

> Turn these code comments and review notes into paper prose.

Bad agent behavior:

- follows source order
- imports class names and helper names
- mentions reviewer complaints as denials
- explains debugging history

Correct action:

1. Route to `references/workflows/translate-source-to-reader-facing-prose.md`.
2. Classify source terms as reader-facing concepts, internal machinery, evidence for judgment, or rejected frames.
3. Write around reader questions, not source order.
4. Keep source machinery silent unless reproducibility requires it.

Expected output:

Reader-facing prose with mechanisms, evidence boundaries, and stable terms; no source-internal vocabulary unless justified.

## Case: skill actability

User says:

> This skill is conceptually nice but agents don't act when it loads.

Concept-only skill:

```text
Use silence, saying/showing, term discipline, and ladder to improve writing.
```

Correct action:

1. Route to `references/workflows/improve-agent-skill-prose.md`.
2. Add first action, route map, subskills/gates, artifacts, output contracts, examples, and test cases.
3. Move theory to references.

Actable skill:

```text
First classify the task: revision, source-to-prose, diagnosis, or skill improvement.
Then run one of four audits: contrast-revelation, term discipline, structural skeleton, exposition boundary.
Return the required artifact: revised text, diagnosis, source ledger, term graph, skeleton map, or claim boundary table.
```
