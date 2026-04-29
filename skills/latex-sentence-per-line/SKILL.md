---
name: latex-sentence-per-line
description: Use when writing, revising, or polishing prose in LaTeX `.tex` files and you want one sentence per line, especially for thesis sections, paper sections, manuscript prose, or any workflow where edited LaTeX prose should be normalized before completion.
---

# LaTeX Sentence-Per-Line

Keep prose-oriented LaTeX files in one-sentence-per-line format after writing edits.

## Overview

This skill makes sentence-per-line formatting a required finishing step for LaTeX writing work. It is for prose-heavy `.tex` files, not for bibliography files, style files, or TeX fragments dominated by equations, macros, or tables.

## When to Use

Use this skill when the user asks to:

- write or revise a LaTeX section,
- polish thesis or paper prose in `.tex` files,
- edit introduction, method, experiments, discussion, or conclusion text,
- keep LaTeX prose in one-sentence-per-line style,
- normalize sentence-per-line formatting after content edits.

Do not use this skill for:

- `.bib`, `.sty`, or `.cls` files,
- generated files,
- table-heavy, macro-heavy, or equation-heavy TeX fragments,
- tasks that are not editing prose.

## Default Workflow

1. Identify which touched files are prose-oriented `.tex` files.
2. Make the requested writing edits first.
3. Run the bundled formatter on the touched prose files:

```bash
python3 scripts/apply_sentence_per_line.py path/to/file.tex
```

For multiple files:

```bash
python3 scripts/apply_sentence_per_line.py path/to/intro.tex path/to/conclusion.tex
```

4. Review the resulting diff.
5. Repair edge cases manually if abbreviations, citations, math-adjacent prose, or command-heavy sentences were split awkwardly.
6. Only then report the task complete.

## Safety Rules

- Preserve LaTeX structure and commands.
- Do not run the formatter on non-prose files just because they end in `.tex`.
- Do not claim completion before the formatter step for qualifying prose edits.
- LaTeX comments beginning with `%` are formatter-excluded and should remain untouched.
- If the script creates suspicious breaks, fix those lines manually instead of skipping review.

## Quick Reference

| Situation | Action |
|---|---|
| Editing prose in `section.tex` | Edit first, then run formatter |
| Editing multiple prose sections | Run formatter on all touched prose files |
| Editing a macro-heavy file | Skip formatter unless the edited region is clearly prose-heavy |
| Suspicious output after formatting | Manually repair affected lines before finishing |

## Formatter Behavior

The bundled script inserts a newline after `.`, `!`, or `?` when the next token begins with an uppercase letter or a LaTeX command. This preserves the existing project behavior instead of broadening scope.

Formatter path:

```text
scripts/apply_sentence_per_line.py
```

## Common Mistakes

- Editing LaTeX prose and forgetting to run the formatter.
- Running the formatter on `.bib`, `.sty`, or `.cls` files.
- Assuming every `.tex` file is prose-oriented.
- Skipping the diff review after formatting.
- Leaving awkward splits around abbreviations or citations uncorrected.
