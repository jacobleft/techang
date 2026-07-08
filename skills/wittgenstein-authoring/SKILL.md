---
name: wittgenstein-authoring
description: Audits and revises reader-facing prose for agentic writing failures: defensive negation/此地无银, source-term leakage, unstable naming, weak structure, logic jumps, and overclaiming. Use when the user asks to revise prose, translate notes/code/reviews into prose, diagnose AI-like or suspicious writing, stabilize terminology, analyze structure rather than content, or enforce silence at exposition boundaries.
---

# Wittgenstein Authoring

## Purpose

This skill produces **thinking-heavy reader-facing prose**. Treat notes, source files, old drafts, reviewer comments, chat context, and user instructions as source material, not final vocabulary. The final prose should expose only what the reader needs: stable terms, visible structure, supported claims, and no ghost traces of rejected frames.

The skill is grounded in Tractatus-derived discipline, but the agent should act through operational audits and workflows, not philosophical commentary.

## Do not use for

- General Wittgenstein explanation, biography, or philosophy commentary unless the user explicitly asks for authoring application.
- Pure content fact-checking without a prose, structure, naming, or boundary problem.
- Tiny typo fixes, spelling cleanup, punctuation-only changes, or ordinary grammar checks. Do the minimal edit directly without invoking this workflow.
- Casual grammar-only edits unless an agentic writing failure mode appears.

If typos or grammar problems appear while auditing logic, structure, terminology, or scope, use best judgment and fix them as part of the revision. Explicitly tell the user you also corrected minor typo/grammar issues discovered during the logic/prose audit.

## First action: decide whether this skill applies

Before choosing a route, check whether the request is about thinking-heavy prose: logic, structure, terminology, scope, argument, source-to-prose translation, or agent-skill actability.

If the request is only a tiny typo fix, spelling cleanup, punctuation-only edit, or ordinary grammar check, do the minimal edit directly and do not invoke this skill's workflow, subskills, or subagents.

If the request is a logic/prose audit and you notice typo or grammar issues along the way, fix them by best judgment and tell the user briefly that incidental typo/grammar cleanup was included.

Then choose one route:

1. **Revise reader-facing prose** — the user provides draft text and wants improvement.
2. **Translate source to reader-facing prose** — the input is code, notes, comments, reviews, logs, prior drafts, or mixed source material.
3. **Diagnose writing failure** — the user asks why the writing feels vague, defensive, AI-like, suspicious, structurally broken, or conceptually confused.
4. **Improve agent-skill prose** — the target is `SKILL.md`, prompts, AGENTS.md, workflow docs, or instructions for future agents.

Use `references/routing-map.md` when uncertain.

## Use subagents when useful

If the platform provides subagents/delegation and the task is substantial, use an independent reviewer subagent to audit the draft or patch before finalizing. This is especially appropriate for long or high-stakes prose, constraint-heavy revisions, skill/prompt edits, grant/proposal/paper sections, and any task where the user explicitly asks for subagents.

Default subagent pattern:

1. Main agent chooses the route, reads the needed workflow/subskill files, and drafts the revision or patch.
2. Reviewer subagent audits the draft against the chosen route, diagnostic subskills, mandatory reader-facing rules, and output contract.
3. Main agent applies valid findings, rejects findings that violate user constraints or evidence boundaries, and performs the final local check.

Do not spawn subagents for tiny grammar edits, short direct rewrites, cases where the subagent would need to ask the user clarifying questions, or platforms without a delegation tool. If no delegation tool exists, perform the reviewer checklist locally. Use `references/subagent-review-protocol.md` for the reviewer prompt, role split, and final check.

## Core diagnostic subskills

Use the smallest set needed:

1. `references/subskills/contrast-revelation-audit.md` — leakage control: negation, contrast, disclaimers, 此地无银, rejected-frame traces.
2. `references/subskills/term-and-naming-discipline-audit.md` — naming control: term graph, stable roles, source/internal vocabulary, synonym drift.
3. `references/subskills/structural-skeleton-audit.md` — structure control: paragraph/section function, logical progression, ladder/scaffolding, reader path.
4. `references/subskills/exposition-boundary-audit.md` — scope control: what can be responsibly said, overclaiming, evidence boundary, silence.

## Mandatory reader-facing rules

### 1. No defensive denial of forbidden concepts

If the user says “do not mention X,” “avoid X,” “remove X,” or “this sounds like X,” do not write:

- “this is not X”
- “we do not claim X”
- “rather than X”
- “not X but Y”
- “并非 X”
- “不是 X，而是 Y”
- “这里并不是说 X”

unless the contrast is explicitly required for reader interpretation.

Default repair:

1. Remove X silently.
2. Remove denials, contrasts, caveats, and disclaimers around X.
3. State the positive mechanism, claim, boundary, or structure the reader needs.

### 2. Source material is not final vocabulary

When reading code, comments, old drafts, logs, reviews, or notes, classify terms before writing:

- reader-facing concept
- source/internal machinery
- evidence for author judgment
- rejected or misleading frame

Only reader-facing concepts normally appear in the final prose.

### 3. Structure should show relations

Do not compensate for weak structure with meta-commentary. If the relation can be shown by paragraph order, section function, example sequence, or definitions, fix the structure instead of explaining that the structure exists.

### 4. Enforce silence at the boundary

If the document lacks evidence, scope, terms, or reader need for a claim, narrow it, move it to future work/open issue, or omit it. Do not fill the gap with grand rhetoric or defensive caveats.

## Workflows

- `references/workflows/revise-reader-facing-prose.md` — default route for rewriting or polishing existing prose.
- `references/workflows/translate-source-to-reader-facing-prose.md` — route for turning source material into prose without leaking source vocabulary or ghost frames.
- `references/workflows/diagnose-writing-failure.md` — route for explaining why writing fails before rewriting.
- `references/workflows/improve-agent-skill-prose.md` — route for making skills/prompts actable by agents.

## Output contracts

If a reviewer subagent was used, do not dump the review report unless requested. Incorporate valid fixes into the final artifact and mention independent review only when useful.

### Revision

Return:

1. Revised text.
2. Optional short change note only if useful.
3. Conflict statement if constraints cannot all be satisfied.

Do not return long skill explanations, Wittgenstein meta-commentary, apologies, or defensive denials.

### Diagnosis

Return:

1. Problem label.
2. Evidence from the text.
3. Repair rule.
4. Rewritten example.

### Source-to-prose

Return:

1. Reader-facing prose.
2. Suppressed/translated source terms only if useful.
3. Remaining risks or missing evidence.

## Testing and examples

- `references/examples.md` — user/action/result cases and bad/good prose examples.
- `references/test-cases.md` — manual trigger and functional tests; no built-in scripts required.
- `references/subagent-review-protocol.md` — when available, how to use reviewer/lens subagents without replacing the main agent's responsibility.

## Theory references

Use theory only to support decisions, not as the user-facing output unless requested:

- `references/theory/tractatus-grounding.md` — compressed theoretical grounding.
- `references/theory/chapters/` — legacy book-derived expansion notes.
- `references/glossary.md`, `references/legacy-patterns.md`, `references/legacy-cheatsheet.md`, `references/legacy-revision-gates.md` — legacy support material.

## Scripts policy

Do **not** include or rely on built-in scripts for this skill. If a particular task needs deterministic extraction or checking, create a temporary ad hoc script during that task, use it, and discard or report it. The skill’s durable unit is the subskill/workflow/case structure, not a bundled linter.
