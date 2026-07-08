# Test Cases

Manual tests for Anthropic-style skill validation. These are not scripts. Use them to check triggering, non-triggering, and functional behavior after installing or revising the skill.

## Trigger tests: should use this skill

### 1. Defensive negation / 此地无银

User says:

> This sounds 此地无银. Remove the defensive wording and don't mention perching.

Expected route:

- `references/workflows/revise-reader-facing-prose.md`
- `references/subskills/contrast-revelation-audit.md`

Expected behavior:

- Treat `perching` as a forbidden ghost frame.
- Remove direct mentions and denial traces.
- Rewrite as positive mechanism.

Functional pass condition:

- Output contains no `perching`, no `not perching`, no `并非栖附`, no `不是……而是` around the forbidden frame.

### 2. Term instability

User says:

> These terms framework/model/system/method are confusing. Stabilize the terminology.

Expected route:

- `references/subskills/term-and-naming-discipline-audit.md`

Expected behavior:

- Build a minimal term graph.
- Assign reader-facing roles.
- Merge synonyms and delete decorative terms.

Functional pass condition:

- Output identifies dominant terms and gives replacement rules.

### 3. Structure only

User says:

> Analyze the structure, not the content. The reader cannot follow the progression.

Expected route:

- `references/subskills/structural-skeleton-audit.md`

Expected behavior:

- Produce a skeleton map.
- Identify missing bridges, premature abstractions, or section-function mismatches.
- Avoid factual/content critique unless structure depends on it.

Functional pass condition:

- Output uses paragraph/section function labels and proposes structural repairs.

### 4. Boundary / silence

User says:

> This conclusion feels too big. Enforce silence where the text goes beyond evidence.

Expected route:

- `references/subskills/exposition-boundary-audit.md`

Expected behavior:

- Build a claim boundary table.
- Narrow, support, move, or omit claims.

Functional pass condition:

- Output distinguishes supported, too-broad, outside-scope, and should-be-silent claims.

### 5. Source to reader prose

User says:

> Turn these code comments and review notes into paper prose.

Expected route:

- `references/workflows/translate-source-to-reader-facing-prose.md`

Expected behavior:

- Build a source ledger internally or briefly if useful.
- Translate internal names into reader-facing mechanisms.
- Prevent reviewer/prompt history from leaking into final prose.

Functional pass condition:

- Final prose is organized around reader questions, not source order.

### 6. Agent skill actability

User says:

> This SKILL.md is conceptually nice but agents don't act when it loads. Make it actable.

Expected route:

- `references/workflows/improve-agent-skill-prose.md`

Expected behavior:

- Add or improve first action, routing, workflows, artifacts, output contracts, and cases.
- Move theory to references.

Functional pass condition:

- Revised skill tells the agent what to do, what artifact to produce, and when to stop.

### 7. Subagent-reviewed high-stakes revision

User says:

> Revise this proposal section and use subagents if possible. Have a reviewer audit it against the skill requirements.

Expected route:

- Main workflow appropriate to the source, usually `references/workflows/revise-reader-facing-prose.md` or `references/workflows/translate-source-to-reader-facing-prose.md`
- `references/subagent-review-protocol.md`

Expected behavior:

- Main agent drafts or patches first.
- Reviewer subagent audits against chosen route, subskills, mandatory reader-facing rules, and output contract.
- Main agent applies valid findings rather than forwarding the review unfiltered.

Functional pass condition:

- Final output satisfies the route's output contract and does not reintroduce forbidden frames, source vocabulary, unsupported claims, or reviewer-facing meta-commentary.

### 8. Skill prose revision with reviewer

User says:

> Make this SKILL.md more actable and have a reviewer check it against agent-skill-development.

Expected route:

- `references/workflows/improve-agent-skill-prose.md`
- `references/subagent-review-protocol.md`

Expected behavior:

- Check frontmatter trigger quality.
- Add or improve route map, workflow steps, output contracts, support-file links, and manual tests.
- Preserve no-built-in-scripts policy unless a reusable deterministic check is justified.
- Reviewer returns findings grouped as must fix / should fix / passes.

Functional pass condition:

- Patched skill has concrete first action, routing, review gate, output contracts, and tests; no orphan support file remains unlinked from `SKILL.md`.

## Non-trigger tests: should not use this skill by default

### 1. Historical philosophy

User says:

> Explain Wittgenstein's picture theory historically.

Expected behavior:

- Do not route to this skill unless the user asks for authoring application.

### 2. Pure fact-checking

User says:

> Is this citation correct?

Expected behavior:

- Use research/Zotero/web tools as appropriate, not this skill alone.

### 3. Grammar-only edit

User says:

> Fix typos only.

Expected behavior:

- Do a light edit directly; do not invoke this skill's route/workflow unless a logical prose failure mode is present.

### 4. Grammar discovered during logical prose audit

User says:

> Audit this paragraph's logic and structure; if you see small grammar problems, fix them too.

Expected behavior:

- Use the appropriate prose/logic route.
- Fix incidental typo or grammar issues by judgment.
- Explicitly mention that minor typo/grammar cleanup was included because it was discovered during the logic/prose audit.

## Performance expectations

Compared with no skill, this skill should reduce:

- meta-commentary about philosophy
- defensive denials after negative instructions
- synonym drift
- local sentence polishing that ignores structure
- unsupported significance claims

It should increase:

- reader-facing output first
- explicit route choice when diagnosing
- stable terms
- visible structure
- bounded claims
