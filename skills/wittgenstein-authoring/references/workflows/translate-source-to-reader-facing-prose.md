# Workflow: Translate Source to Reader-Facing Prose

## Use when

Use when input is code, logs, comments, review notes, meeting notes, source excerpts, old drafts, chat context, or mixed material that must become prose for a reader.

## Steps

1. **Identify target reader and genre.**
   Decide what the reader needs to understand, decide, or do.

2. **Build a source ledger.**

```text
Source item | Source role | Reader-facing concept | Suppress/Translate/Say | Risk
```

Classify each source item:

- reader-facing concept
- implementation machinery
- evidence for author judgment
- rejected or misleading frame
- citation/evidence boundary

3. **Apply Term/Naming Discipline.**
   Decide which names may enter the prose. Translate source/internal terms into mechanisms or omit them.

4. **Apply Contrast-Revelation.**
   Prevent old, rejected, reviewer, or prompt-history frames from reappearing as denial or disclaimer.

5. **Apply Structural Skeleton.**
   Arrange prose around reader questions, not source order.

6. **Apply Exposition Boundary.**
   Do not expose implementation detail, debugging history, or speculative claims unless the reader needs them.

7. **Write prose.**
   Produce the reader-facing version. Keep the source ledger private unless the user asked for diagnosis or traceability.

8. **Use a reviewer subagent when useful.**
   If the source set is large, mixed, or constraint-heavy, ask a reviewer subagent to audit whether source/internal terms leaked into reader-facing prose, whether rejected frames reappeared, and whether claims stayed inside the evidence boundary. Use `references/subagent-review-protocol.md` and provide the source ledger or a concise source summary.

## If stuck

- If source terms are unclear, keep them out of prose and report uncertainty briefly.
- If source order conflicts with reader order, follow reader order.
- If implementation detail may be reproducibility-critical, include it only at the boundary where readers need it.
- If reviewer criticism or old-draft wording appears, translate the valid concern instead of carrying the defensive frame.

## Output contract

Return reader-facing prose first. Optionally list suppressed/translated source terms and remaining risks. Do not dump the source ledger unless requested.
