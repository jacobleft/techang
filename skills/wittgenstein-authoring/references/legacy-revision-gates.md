# Revision Gates

Use these gates after drafting and before presenting edits.

## Gate 1: Reader-Facing Vocabulary

Ask of every noun imported from source material:

- Does the reader need this term to understand the method, argument, or reproducibility boundary?
- Is it part of the public contribution or only local implementation machinery?
- Can the mechanism be stated without the source term?

Bad:

```text
Each defect is instantiated through an AssetFactory subclass.
```

Better:

```text
Each defect follows a two-stage generation pattern: a tagged bounding box is first supplied for placement, and the placed instance is then replaced by the wall-attached defect plane, shader material, and any explicit geometry.
```

## Gate 2: Defensive Negation

Flag defensive negation, not only the literal phrases listed here.
Examples include "rather than", "not merely", "not just", "instead of", "unlike", "in contrast to", "compared with", "not X but Y", and claims that end by reassuring the reader that something is not "alone".
The exact phrase list is only a tripwire; the real question is whether the sentence is arguing with an imagined criticism.

Keep the contrast only if:

1. The reader needs the contrast to interpret the method.
2. Both sides are defined in the document.
3. The positive claim is evidenced or immediately shown by structure.

Bad:

```text
The contribution is defect-specific parameterization rather than the use of Blender rendering alone.
```

Better:

```text
The following subsections specify the controllable morphology, support, material response, bump or displacement behavior, and appearance variation for each defect category.
```

## Gate 3: Claim Load

For each claim, mark what supports it:

- Definition in the same paragraph.
- Equation or procedure.
- Figure, table, or example.
- Citation or source evidence.
- Section structure that shows the relation.

If support is absent, narrow the claim or remove it.

## Gate 4: Local Term Stability

For the current document, identify local near-synonym clusters before revising.
Clusters should contain terms at the same conceptual level, not parent-child terms or words that already have distinct roles.
Examples: "method/system/framework", "module/component/unit", "pipeline/workflow/procedure", "model/estimator/network".
Do not switch among terms in the same cluster unless each has a distinct role.
Use one term for one role.
Change terms only when the concept changes.

## Autonomous Use

If a specific task needs deterministic extraction, create a temporary ad hoc check for that task only. Treat any automated findings as prompts for revision, not as a substitute for judgment.
