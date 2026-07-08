# Patterns

## Silence Pass
**When to use**: Before finalizing claims, conclusions, executive summaries, CV bullets, README promises, proposal outcomes, or social posts.

**How**:
1. Underline each load-bearing claim.
2. Ask whether its terms, evidence, scope, and medium support it.
3. For unsupported claims, choose one: clarify, qualify, move to open question, show indirectly, or omit.
4. Preserve silence where adding prose would create false clarity.

**Trade-offs**: Reduces drama and apparent completeness; increases trust and precision.

## Say-or-Show Split
**When to use**: When drafting explanations, examples, docs, agent skills, or prompts.

**How**:
1. List what the reader must act on: say these explicitly.
2. List what the reader must recognize: show these through structure, examples, order, or behavior.
3. Remove meta-claims the document can prove by its own form.
4. Add explicit constraints where structure would be ambiguous.

**Trade-offs**: Produces cleaner prose but requires stronger examples and structure.

## Source-to-Document Translation Gate
**When to use**: After reading code, notes, logs, reviewer comments, handwritten annotations, or any source material that is not itself the target document.

**How**:
1. List the nouns and claims learned from the source material.
2. Classify each item as conceptual method term, implementation machinery, or evidence for authoring judgment.
3. Say conceptual method terms only when the reader needs them.
4. Translate implementation machinery into reader-facing mechanism, or omit it.
5. Keep judgment evidence silent unless reproducibility, citation, or auditability requires it.

**Trade-offs**: Prevents source leakage but requires the author to separate what they know from what the document should say.

## Defensive Negation Audit
**When to use**: Whenever a sentence uses contrastive framing, including but not limited to "not merely", "rather than", "not just", "instead of", "unlike", "not X but Y", or "X alone".

**How**:
1. Ask what imagined criticism the sentence is answering.
2. Keep the contrast only if it is necessary for the reader's interpretation.
3. Verify that both sides of the contrast are defined and evidenced.
4. If the contrast is defensive, delete it and state the positive mechanism directly.
5. Let examples, subsections, equations, or procedure show the distinction where possible.

**Trade-offs**: Removes self-protective prose; may require stronger structure to carry the intended emphasis.

## Term Ledger
**When to use**: In any document with overloaded terms: model, agent, skill, system, context, value, method, impact, rigor, theory.

**How**:
1. Make a table of load-bearing terms.
2. Define each term by use, not by vibe.
3. Mark forbidden meanings or competing meanings.
4. Replace synonyms that do not carry distinct roles.
5. Recheck conclusions for semantic drift.

**Trade-offs**: May feel repetitive; repetition is often required for conceptual stability.

## Sharp-Boundary Revision
**When to use**: When a paragraph sounds impressive but unclear.

**How**:
1. Rewrite the paragraph as “This is how things stand: …”.
2. Name actor, relation, condition, evidence, and limit.
3. State what the claim does not include.
4. Cut sentences that do not sharpen the boundary.

**Trade-offs**: Removes rhetorical flourish; exposes weak thinking quickly.

## Scaffold Audit
**When to use**: When revising introductions, docs, skills, proposals, tutorials, or explanations of difficult concepts.

**How**:
1. Mark each example, definition, analogy, and preface.
2. Classify it: mandatory, helpful, redundant, obstructive.
3. Keep mandatory support even if it lengthens the document.
4. Remove or compress redundant support.
5. Add warnings where an analogy stops applying.

**Trade-offs**: Prevents expert compression but can produce over-explanation if reader level is not specified.

## Picture Fit Test
**When to use**: Before using an example, analogy, diagram, or model.

**How**:
1. Identify the target relation the picture must preserve.
2. Map elements in the picture to elements in the target.
3. Mark mismatches.
4. State what the picture captures and what it excludes.

**Trade-offs**: Slows drafting but avoids misleading examples.

## Exceptional Proposition Tree
**When to use**: Only for unusually hard logical arguments, formal prompts, conceptual designs, or high-stakes claims where hidden dependency is the problem.

**How**:
1. Ask the user whether they want Tractatus-level sharpness.
2. Create 3–7 cardinal propositions.
3. Attach subordinate propositions only under the parent they clarify.
4. Keep terms stable across the whole tree.
5. Convert to normal prose unless the tree remains necessary scaffolding.

**Trade-offs**: Gives maximum inspectability; usually too rigid for daily or normal professional writing.
