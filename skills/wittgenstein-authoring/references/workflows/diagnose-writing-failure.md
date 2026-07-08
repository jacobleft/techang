# Workflow: Diagnose Writing Failure

## Use when

Use when the user asks why text feels wrong, vague, suspicious, defensive, AI-like, structurally weak, or terminologically confused.

## Steps

1. **Do not rewrite immediately.**
   Diagnosis comes before prose generation unless the user explicitly asks for direct rewrite.

2. **Classify the failure.**

Use one or more labels:

- contrast-revelation failure
- term/naming discipline failure
- structural skeleton failure
- exposition boundary failure
- mixed failure

3. **Quote evidence.**
   Use short excerpts only. Do not overquote.

4. **Name the reader effect.**
   Explain what the reader is likely to experience: suspicion, confusion, missing bridge, unsupported claim, source-vocabulary leak, etc.

5. **Give repair rule.**
   Provide one operational rule, not a long essay.

6. **Provide a rewritten example.**
   Rewrite one representative sentence or paragraph.

7. **Use a reviewer subagent when useful.**
   For long, high-stakes, or mixed-failure diagnoses, ask a reviewer subagent to check whether the diagnosis cites visible evidence, names the reader effect, gives an operational repair rule, and avoids drifting into full rewrite or philosophy commentary. Use `references/subagent-review-protocol.md`.

## If stuck

- If multiple failures are present, name the dominant one first and mention secondary failures briefly.
- If the text is too long, sample the most representative paragraph and say the diagnosis is local.
- If the user asks why it feels wrong, do not jump straight to rewriting.
- If evidence is missing, state that the diagnosis is based on visible prose only.

## Output contract

Return:

1. Problem label.
2. Evidence from the text.
3. Repair rule.
4. Rewritten example.

Do not produce a full rewrite unless asked.
