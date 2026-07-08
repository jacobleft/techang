# Subagent Review Protocol

Use this protocol when the platform provides subagents/delegation and the task is large enough that an independent audit improves the result. Do not spawn subagents for tiny grammar edits, short single-paragraph rewrites, or urgent direct answers.

## When to use subagents

Use a subagent when at least one condition holds:

- The draft is long, high-stakes, or constraint-heavy.
- The user explicitly asks for subagents or independent review.
- Multiple diagnostic lenses are needed and may conflict.
- The task is revising a skill, prompt, grant section, paper section, proposal, README, or public-facing document.
- The agent has produced a substantial revision and needs a second pass against the skill requirements.

Do not use subagents when:

- The user asked for a minimal direct edit.
- The platform has no subagent/delegation tool available.
- The subagent would need to ask the user questions.
- The added latency would be larger than the revision itself.

## Roles

### Reviewer subagent (default)

Purpose: audit the main agent's plan or draft against this skill's requirements.

Give the reviewer all constraints explicitly. Include:

1. User request and target genre.
2. Original text or target file excerpt.
3. Current draft or planned patch.
4. Applicable route and subskills.
5. Mandatory reader-facing rules:
   - remove forbidden ghost frames silently;
   - source material is not final vocabulary;
   - structure should show relations rather than describe them;
   - unsupported claims should be narrowed, moved, or omitted.
6. Required output: findings only, grouped as `must fix`, `should fix`, and `passes`; include minimal replacement wording only when needed to make a finding actionable.

For skill/prompt/workflow revisions, also audit against agent-skill-development requirements:

- frontmatter has clear trigger/use conditions;
- the main body has concrete workflow steps;
- route map covers common user situations;
- output contracts define final artifacts;
- manual tests exist when no bundled script is appropriate;
- support files are linked from `SKILL.md`;
- no built-in scripts are added unless deterministic reuse clearly justifies them.

### Lens subagents (optional)

For very large or high-stakes documents, split independent audits by lens:

- Contrast reviewer: forbidden frames, denial traces, defensive caveats.
- Term reviewer: unstable naming, leaked source vocabulary, synonym drift.
- Structure reviewer: paragraph/section function, reader path, missing bridges.
- Boundary reviewer: overclaiming, unsupported scope, necessary silence.

Use lens subagents only when parallel review is clearly useful. Otherwise use one reviewer subagent with all requirements.

## Main-agent workflow with reviewer

1. Main agent chooses the route and drafts the revision or patch.
2. Main agent dispatches a reviewer subagent when available and useful.
3. Reviewer audits against the route, subskills, mandatory rules, and output contract.
4. Main agent decides which reviewer findings to apply. Do not blindly accept review; preserve user constraints and source evidence.
5. Main agent applies fixes and performs the final local check.
6. Final response includes the revised text or patch summary first, plus a short note that independent review was used if useful.

If the reviewer finds `must fix` issues, the main agent must revise and, for substantial changes, rerun the reviewer or perform an equivalent local recheck against the same checklist. Do not proceed with unresolved `must fix` findings.

## Reviewer prompt template

```text
You are a reviewer subagent for the wittgenstein-authoring skill.

Task: Audit the draft/patch below against the skill requirements. Do not rewrite unless a minimal replacement is needed to show a fix.

User request:
<request>

Target genre/reader:
<genre>

Applicable route and subskills:
<route + subskills>

Hard requirements:
- No defensive denial of forbidden concepts or ghost frames.
- Source/internal terms must not become reader-facing vocabulary unless justified.
- Structure must show relations through order/function, not meta-commentary.
- Claims must stay inside the evidence/scope boundary; silence unsupported claims.
- Output contract for the chosen route must be satisfied.

Original/source material:
<source>

Draft or planned patch:
<draft>

Return:
1. Must fix
2. Should fix
3. Passes
4. Minimal suggested replacements only where needed to make a finding actionable
```

## Final local check

Before finalizing, the main agent must still check:

- Did the revision obey the user's explicit constraints?
- Did any reviewer suggestion reintroduce a forbidden frame or source term?
- Is the output contract still satisfied?
- Is the final prose reader-facing rather than reviewer-facing?

## Do not

- Do not ask the reviewer to rewrite the full prose by default.
- Do not expose reviewer scaffolding in the final reader-facing answer.
- Do not accept a reviewer suggestion that reintroduces forbidden terminology, defensive denial, source vocabulary, or unsupported claims.
- Do not use subagents as a substitute for reading the relevant workflow/subskill files.
