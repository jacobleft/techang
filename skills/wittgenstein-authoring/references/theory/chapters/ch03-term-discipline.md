# Chapter 3: Term Discipline

## Core Idea

Most conceptual confusion in writing comes from unstable terms: the same word used in different roles, or different words used as if they were identical.

## Frameworks Introduced

- **Sign / Symbol / Use discipline**
  - When to use: whenever a document contains load-bearing terms.
  - How: distinguish the visible word from the role it plays in context; define or rename when the role changes.
- **Logical syntax for documents**
  - When to use: in papers, specs, prompts, README/AGENTS.md, proposals, and docs where misuse of terms creates operational errors.
  - How: establish allowed terms, their roles, substitution rules, and forbidden overloads.
- **Occam's maxim for signs**
  - When to use: during revision.
  - How: remove words, categories, headings, and labels that do no work.

## Key Concepts

- **Sign**: the visible word, phrase, variable, label, or string.
- **Symbol**: a sign used with a specific sense and role.
- **Mode of signification**: how the sign functions in the sentence or system.
- **Logical syntax**: rules governing how terms may be combined independent of their psychological associations.
- **Overload**: one sign secretly serving multiple symbols.
- **Pseudo-concept**: a term that looks like a concept but cannot be used as one without confusion.

## Mental Models

- A glossary is not decoration; it is a syntax table for the document.
- Rename concepts when their use diverges.
- Mark overloads explicitly: “model” as estimator vs conceptual picture vs LLM.
- If two terms are interchangeable, choose one or state the distinction.
- If a term has no operational consequence, cut it.

## Anti-patterns

- **Same sign, different symbol**: using “agent,” “model,” “system,” “context,” or “skill” in incompatible senses.
- **Different signs, same symbol**: rotating through synonyms for style when precision matters.
- **Unlicensed abstraction**: using words such as “intelligence,” “quality,” “alignment,” “impact,” or “rigor” without specifying their role.
- **Semantic drift**: allowing a term introduced narrowly to become broad in the conclusion.
- **Aesthetic synonymy**: varying key terms to avoid repetition even though repetition is needed for stability.

## Key Takeaways

1. In ordinary prose, style can vary; in load-bearing prose, terms must remain stable.
2. Do not use one word for multiple roles without naming the overload.
3. Do not use multiple words for one concept unless the distinction matters.
4. Define by use: show how the term behaves in claims, examples, and decisions.
5. Every load-bearing term should survive a substitution test: what can replace it without changing the claim?
6. Useless signs are meaningless for the document's work.

## Connects To

- **Ch 1**: undefined terms often mark what should be left unsaid.
- **Ch 2**: stable terms decide what can be said and what can be shown.
- **Ch 4**: clarification depends on term boundaries.
- **Ch 7**: proposition hierarchy fails if terms are unstable.
