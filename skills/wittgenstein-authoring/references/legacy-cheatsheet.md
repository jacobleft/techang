# Cheatsheet

## Default Priority

1. **Silence** — do not say what cannot be said clearly here.
2. **Saying vs showing** — say action-critical claims; show structure, competence, and relations.
3. **Source translation** — do not leak source-material terms into the target document without a role.
4. **Defensive negation** — avoid contrastive reassurance unless it is necessary and evidenced.
5. **Term discipline** — stabilize load-bearing words.
6. **Clarification** — sharpen boundaries before polishing style.
7. **Scaffolding** — keep necessary reader support.
8. **Proposition tree** — use only after asking, for exceptional logical rigor.

## Say / Show / Silence

| Need | Action |
|---|---|
| Reader must act, decide, or comply | Say it explicitly |
| Reader must recognize a relation or pattern | Show it through structure/example |
| Claim lacks evidence, terms, or scope | Stay silent, mark unknown, or narrow it |
| Concept is too hard to say directly | Scaffold, then say/show progressively |
| Dependency is too complex for prose | Ask before using proposition tree |

## Source Translation Check

- Did I read source material that is not the target document?
- Which imported nouns are conceptual terms, and which are implementation machinery?
- Does the reader need this source term, or only the mechanism it points to?
- Am I using source facts to guide wording while keeping irrelevant evidence silent?
- Have I checked that class names, helper APIs, private paths, and local labels did not leak into paper prose?

## Defensive Negation Check

- Does the sentence use contrastive framing such as "not merely", "rather than", "not just", "instead of", "unlike", "not X but Y", or "X alone"?
- Is the contrast necessary for interpretation, not just reassurance?
- Are both sides defined and evidenced?
- Can the positive mechanism be stated directly instead?
- Can the following structure show the distinction better than a defensive sentence?

## Term Discipline Check

- Same word, same concept?
- Different words, real distinction?
- Any overloaded term marked?
- Any key term drifting between intro and conclusion?
- Any useless signs/headings/labels?
- Did I define local near-synonym clusters at the same conceptual level and check role drift within them?

## Scaffolding Check

| Scaffold type | Keep if | Remove if |
|---|---|---|
| Definition | Reader needs it to use later terms | It repeats common knowledge for this audience |
| Example | It preserves the target structure | It is decorative or mismatched |
| Analogy | It enables ascent and has limits marked | It becomes mistaken for the concept |
| Background | It defines the problem space | It delays the point without changing understanding |
| Proposition tree | Dependencies must be inspectable | Normal prose is enough |

## Ask Before High-Rigor Mode

“Do you want Tractatus-level logical sharpness here? It will make dependencies explicit but may be too rigid for ordinary writing.”

Use only if the user accepts.
Even for formal or high-stakes work, ask first before imposing proposition-tree structure.

## Common Applications

- **Paper**: silence overclaims and code-internal names; say methods/claims; show rigor through design and evidence.
- **README/docs**: say install/use/failure conditions; show workflow through examples.
- **AI skill/prompt**: say triggers and constraints; show priorities through ordering and examples.
- **Proposal**: say scope and evidence; scaffold the problem; silence unsupported promises.
- **Email/DM**: say the ask; show respect by brevity; silence unnecessary argument.
- **Post/blog**: say the thesis; show insight with one strong picture; silence totalizing claims.
