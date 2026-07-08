# Term and Naming Discipline Audit

## Use when

Use when prose introduces too many terms, rotates synonyms, leaks source/internal vocabulary, overloads names, or confuses the reader about what each term does.

Typical triggers:

- “terms are confusing”
- “too many new terms”
- “term discipline”
- “source vocabulary leaked”
- “don’t invent concepts”
- “make terminology stable”
- “this sounds like code/review language”

## Failure mode

The prose contains signs without stable reader-facing roles. A word may carry multiple roles, multiple words may name the same role, or source-material names may enter prose even though they are not reader concepts.

## Operation

Build a term graph / term dictionary:

```text
Term | Reader-facing role | First use | Defined? | Synonyms/near terms | Source/internal? | Necessary? | Action
```

For short paragraphs, use minimal mode instead of a full table:

```text
Unstable term | Problem | Dominant term | Replacement rule
```

For each load-bearing term, ask:

1. Does the reader need this term?
2. Is the term defined before it is used heavily?
3. Does it name a distinct concept?
4. Is it a synonym of another term used for stylistic variation?
5. Did it come from code, comments, reviews, old drafts, or implementation machinery?
6. Does it appear once and vanish?
7. Does its meaning broaden or shift later?

## Actions

- **Keep and define** — term is necessary and stable.
- **Merge** — synonym or near-synonym should collapse into the dominant term.
- **Replace** — source/internal term should become reader-facing mechanism language.
- **Delete** — term is decorative, single-use, or creates false conceptual weight.
- **Move to glossary** — term is necessary but too heavy for inline explanation.
- **Mark boundary** — term is legitimate only in a limited scope.

## Reader-facing criterion

A reader should not have to infer whether “method,” “framework,” “model,” “system,” “mechanism,” and “strategy” refer to the same thing or different things.

## Repair principles

```text
One role, one term.
One term, one role.
New names only when they reduce reader confusion.
Source names appear only when they are reader-facing.
```

## Example

Problem terms:

```text
framework, method, model, system, pipeline
```

Audit:

```text
framework | broad container | undefined | method/system | no | delete/replace
method    | actual procedure | partially defined | framework | yes | keep
model     | representation/equations | distinct | no | yes | keep and define
system    | ambiguous: physical object or software | undefined | framework | no | replace by physical system/software system as needed
```

Repair:

Use “method” for procedure, “model” for representation/equations, and avoid “framework/system” unless their roles are explicitly defined.

## Tractatus grounding

A sign becomes a symbol only in use. Naming discipline makes each visible term correspond to a stable role in the document’s logical syntax.
