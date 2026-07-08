# Routing Map

Use this map before editing. Route by the user's complaint, not by keywords alone.

## Direct routes

```text
“revise/polish/rewrite this paragraph”
→ references/workflows/revise-reader-facing-prose.md

“turn these notes/code/comments/reviews into prose”
→ references/workflows/translate-source-to-reader-facing-prose.md

“why does this feel wrong / AI-like / suspicious?”
→ references/workflows/diagnose-writing-failure.md

“make this skill/prompt/AGENTS.md actable”
→ references/workflows/improve-agent-skill-prose.md

“use subagents” / “have a reviewer audit this” / “independent review against the skill requirements”
→ choose the normal prose route first, then apply references/subagent-review-protocol.md
```

## Diagnostic routes

```text
“don’t mention X” / “avoid X” / “remove this frame”
→ references/subskills/contrast-revelation-audit.md

“此地无银” / “sounds defensive” / “protesting too much”
→ references/subskills/contrast-revelation-audit.md

“terms are confusing” / “too many terms” / “source vocabulary leaked”
→ references/subskills/term-and-naming-discipline-audit.md

“only analyze structure, not content” / “reader cannot follow” / “logic jumps”
→ references/subskills/structural-skeleton-audit.md

“too broad” / “unsupported” / “overclaiming” / “what should be silent”
→ references/subskills/exposition-boundary-audit.md
```

## Combined routes

### Source material to paper/proposal prose

Use:

1. Term and Naming Discipline
2. Contrast-Revelation
3. Structural Skeleton
4. Exposition Boundary

Do not follow the source order. Arrange around reader questions.

### AI-like prose

Usually mixed:

1. Contrast-Revelation — defensive/disclaimer traces
2. Structural Skeleton — formulaic progression or missing reader path
3. Term Discipline — inflated abstractions and synonym drift
4. Boundary — unsupported significance language

### Skill loaded but agent does not act

Use Improve Agent Skill Prose. Usual fixes:

- add first action
- add route map
- merge overlapping subskills
- define artifacts
- define output contracts
- move theory to references
- add cases

### Substantial or high-stakes revision with subagents available

Use:

1. Normal route selection: revision, source-to-prose, diagnosis, or skill improvement.
2. One reviewer subagent with all applicable mandatory rules and output contract.
3. Optional lens subagents only for large documents where contrast, terms, structure, and boundary can be reviewed independently.
4. Main-agent final local check before returning the artifact.

Do not let reviewer output become reader-facing prose.

## Do not route to this skill for

- general historical/philosophical commentary on Wittgenstein
- generic biography or philosophy explanation
- pure content fact-checking without a prose/structure problem
- tiny typo fixes, spelling cleanup, punctuation-only edits, or ordinary grammar checks

Unless the user explicitly asks for authoring application.

If typo or grammar issues are discovered during a logic, structure, terminology, or scope audit, fix them by judgment and mention briefly that they were corrected as incidental cleanup.
