# Workflow: Revise Reader-Facing Prose

## Use when

Use when the user gives existing prose and asks to revise, polish, rewrite, shorten, clarify, make less AI-like, make more rigorous, or remove a framing.

Do not use for typo-only, spelling-only, punctuation-only, or ordinary grammar-check requests. Make those minimal edits directly. If minor typo/grammar issues are discovered while performing this logic/prose workflow, fix them by judgment and tell the user briefly.

## Steps

1. **Identify reader and genre.**
   - paper, proposal, README/docs, skill/prompt, note, email/DM, post/blog, CV/profile.
   - State the reader-facing purpose in one line if needed.

2. **Identify revision intent.**
   - clarity
   - rigor
   - concision
   - tone
   - remove forbidden framing
   - reduce AI-like writing
   - stabilize terms
   - improve structure

3. **Preserve what must remain.**
   - factual claims
   - citations
   - technical terms with stable roles
   - required section function
   - user-imposed constraints

4. **Run only the needed audits.**
   - Contrast-Revelation if negation, contrast, forbidden frames, or suspicion appear.
   - Term/Naming Discipline if terms drift, multiply, or leak from source material.
   - Structural Skeleton if paragraphs or logic do not guide the reader.
   - Exposition Boundary if claims are too broad, vague, or unsupported.

5. **Rewrite affected parts.**
   - Replace defensive contrast with positive formulation.
   - Merge or define unstable terms.
   - Reorder or bridge structural jumps.
   - Narrow or silence unsupported claims.

6. **Use a reviewer subagent when useful.**
   If the revision is long, high-stakes, constraint-heavy, or explicitly requests subagents, send the original, user constraints, chosen subskills, and draft to a reviewer subagent using `references/subagent-review-protocol.md`. Apply only findings that preserve the user's constraints and evidence boundary.

7. **Final check.**
   - No ghost frame from forbidden concepts.
   - Reader-facing terms are stable.
   - Structure carries the intended relation.
   - Claims fit support and scope.

## If stuck

- If reader/genre is unknown, infer from the document type; ask only if ambiguity changes the revision.
- If the user asked for direct rewrite, do not lecture or over-diagnose.
- If the user asked for diagnosis, do not produce a full rewrite unless asked.
- If constraints conflict, preserve reader-facing clarity and report the conflict.

## Output contract

Return revised text first. Add a short change note only if useful. If constraints conflict, state the conflict. Do not explain the skill or mention Wittgenstein unless asked.
