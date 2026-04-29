# techang skills

Reusable agent skills for Claude Code, OpenCode, Cursor, and other tools supported by the Vercel `skills` CLI.

## Install

Install from GitHub:

```bash
npx skills add jacobleft/techang
```

List available skills without installing:

```bash
npx skills add jacobleft/techang --list
```

Install one skill only:

```bash
npx skills add jacobleft/techang --skill latex-optimizer
```

## What is in this repo

This repository uses the standard `skills/` layout expected by the Vercel `skills` ecosystem. Each skill lives in its own directory and includes a `SKILL.md` entrypoint plus any bundled scripts, references, or examples.

```text
techang/
├── README.md
└── skills/
    ├── intimate-relationship-guide/
    ├── latex-optimizer/
    ├── latex-sentence-per-line/
    └── ocr-tiered/
```

## Skill catalog

| Skill | Purpose |
|---|---|
| `intimate-relationship-guide` | Relationship guidance grounded in scientific psychology, with references for attraction, communication, conflict, maintenance, and breakup recovery. |
| `latex-optimizer` | Low-noise LaTeX compilation workflow for AI agents, with tool detection and tiered fallback across `texfot`, `pplatex`, `latexmk`, and related commands. |
| `latex-sentence-per-line` | Finishing workflow for prose-heavy LaTeX files that normalizes edited text into one-sentence-per-line format. |
| `ocr-tiered` | Repeatable OCR workflow that compares PaddleOCR, Tesseract with `tessdata_best`, and default Tesseract for local images and PDFs. |

## Repository notes

- Skills are stored under `skills/`, which is one of the standard discovery paths used by `npx skills`.
- Each skill is self-contained and can include helper scripts, examples, and reference material.
- The GitHub install path works even if a skill is not yet surfaced in the `skills.sh` search index.

## Verification

The repository can be verified with:

```bash
npx skills add jacobleft/techang --list
```

Expected result: the CLI should discover these four skills:

- `intimate-relationship-guide`
- `latex-optimizer`
- `latex-sentence-per-line`
- `ocr-tiered`
