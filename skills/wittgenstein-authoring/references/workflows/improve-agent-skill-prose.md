# Workflow: Improve Agent Skill Prose

## Use when

Use when revising `SKILL.md`, prompts, AGENTS.md, workflow docs, or instructions intended for future agents.

## Failure mode

The document may be conceptually rich but not actable. Agents load it, understand it, and still fail to act because there is no route, task type, workflow, artifact, output contract, or case.

## Steps

1. **Identify intended agent action.**
   What should the agent do differently after loading this text?

2. **Build a route map.**
   Map user complaints/triggers to workflows or subskills.

3. **Apply Structural Skeleton.**
   Ensure the document has:
   - purpose
   - first action
   - routing map
   - workflows
   - diagnostic subskills/gates
   - artifacts
   - output contracts
   - examples/cases

4. **Apply Term/Naming Discipline.**
   Stabilize names for routes, subskills, gates, artifacts, and modes. Merge overlapping subskills.

5. **Apply Exposition Boundary.**
   Move theory, background, and long explanations into references. Keep main instructions operational.

6. **Apply Contrast-Revelation.**
   Replace negative instructions that may provoke denial traces with positive action rules.

7. **Produce actable prose.**
   The future agent should know what to do, what artifact to create, and when to stop.

8. **Use a reviewer subagent when useful.**
   For skill, prompt, AGENTS.md, or workflow revisions, use a reviewer subagent whenever available unless the edit is trivial. Ask it to audit frontmatter triggering, first action, routing, subskills/gates, artifacts, output contracts, examples/tests, and whether the main file stays operational rather than theoretical. Use `references/subagent-review-protocol.md`.

## If stuck

- If the skill is conceptually rich but not actable, preserve theory as references and rewrite the main file around routes and artifacts.
- If two subskills overlap, merge them by scale or mode rather than creating siblings.
- If a deterministic check seems useful, do not add a built-in script by default; describe the check or create an ad hoc one only for the current task.
- If the target agent is the reader, optimize for action rather than explanation.

## Output contract

Return a revised structure or patched prose. Prefer artifacts over advice: route map, subskill list, workflow steps, output contracts, examples.
