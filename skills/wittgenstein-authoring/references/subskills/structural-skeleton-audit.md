# Structural Skeleton Audit

## Use when

Use when prose has content but weak architecture: the reader cannot see why paragraphs appear in this order, what each unit does, or how the logic progresses.

Typical triggers:

- “analyze structure, not content”
- “reader cannot follow”
- “logic jumps”
- “too abrupt”
- “needs better progression”
- “scaffolding / ladder”
- “section feels disorganized”
- “outline the logic”

## Failure mode

The agent edits local sentences but misses the reader-facing structure. Claims arrive before definitions, abstractions appear before examples, bridge sentences are missing, or paragraphs combine multiple incompatible functions.

## Operation

Create a skeleton map:

```text
Unit | Function | Main move | Depends on | Reader question answered | Structural issue | Repair
```

Possible functions:

```text
context
problem pressure
gap
definition
mechanism
method
evidence
result
limitation
transition
implication
summary
```

Then audit:

1. Does every paragraph or section have one dominant function?
2. Are functions ordered according to reader need?
3. Is any definition used before it is available?
4. Does the text jump abstraction levels?
5. Is a bridge sentence missing?
6. Is scaffolding necessary, obstructive, or absent?
7. Does the section title match what the section actually does?
8. Can structure show a relation better than explicit meta-commentary?

## Ladder/progression check

Use this as a sub-mode of structural skeleton, not a separate subskill.

Ask:

```text
What must the reader already understand to accept this sentence?
Has the text supplied that step?
If not, should we add a bridge, definition, example, or reorder the section?
```

## Repair operations

- Reorder units.
- Split overloaded paragraphs.
- Merge functionless fragments.
- Move definitions earlier.
- Insert a bridge sentence.
- Delete decorative preamble.
- Rename section headings around actual function.
- Preserve necessary scaffolding; remove only scaffolding that no longer helps reader ascent.

## Reader-facing criterion

The reader should always know:

```text
Where am I?
Why am I reading this sentence now?
What question does this paragraph answer?
What does this prepare me to understand next?
```

## Tractatus grounding

Some relations are shown by form. The ladder is scaffolding for reader ascent: keep support while it helps the reader reach the concept; remove it when it obstructs clarity.
