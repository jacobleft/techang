# Exposition Boundary Audit

## Use when

Use when prose says more than the document can responsibly carry, or when the user asks what should be omitted, narrowed, supported, or left silent.

Typical triggers:

- “overclaiming”
- “too grand”
- “unsupported”
- “scope too broad”
- “make it rigorous”
- “what should be silent”
- “boundary of exposition”
- “this conclusion is too big”

## Failure mode

The draft fills evidence or scope gaps with rhetoric, vague significance, fake completeness, or speculative implications. It asks the reader to accept more than the document has prepared.

## Operation

Build a claim boundary table:

```text
Claim | Reader need | Evidence/support in text | Scope | Status | Action
```

Statuses:

```text
supported
needs support
too broad
outside scope
reader-does-not-need
future-work
should-be-silent
```

For each claim, ask:

1. Does the reader need this claim for the document’s purpose?
2. Is the claim supported by evidence, reasoning, citation, or already established terms?
3. Is the scope local, general, speculative, or universal?
4. Does the document have the authority to say this now?
5. Should this be a claim, limitation, open question, future work, or silence?

## Repair operations

- Delete unsupported claims.
- Narrow broad claims to local scope.
- Add evidence or citation if the claim is necessary.
- Convert speculation to future work.
- Convert vague significance into concrete contribution.
- Move peripheral ideas out of the main line.
- Omit what exceeds the document boundary.

Genre-specific repairs:

- **Paper**: narrow the claim, cite evidence, or move speculation to limitations/future work.
- **Proposal**: convert broad impact claims into bounded research objectives and evaluation conditions.
- **README/docs**: replace unsupported promises with tested behavior, failure modes, and recovery steps.
- **CV/profile**: remove inflated significance unless externally evidenced; state role, result, and scope.

## Reader-facing criterion

The reader should not be asked to accept claims that the document has not prepared, supported, or scoped.

## Repair principle

```text
Say what the document can carry.
Mark or omit what exceeds its boundary.
Do not fill boundary gaps with rhetoric.
```

## Examples

Overclaim:

> This approach establishes a new paradigm for aerial manipulation.

Boundary repair:

> This approach provides a lightweight structural route for contact and load transfer in low-altitude UAV manipulation scenarios.

Unsupported completeness:

> The system is robust in complex environments.

Boundary repair:

> The prototype tests characterize contact stability under the tested load and geometry ranges.

## Tractatus grounding

Where the document lacks the terms, evidence, or scope to say something clearly, pass over it in disciplined silence or mark the boundary explicitly.
