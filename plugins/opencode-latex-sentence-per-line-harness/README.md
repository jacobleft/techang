# opencode-latex-sentence-per-line-harness

opencode plugin that automatically rewrites touched `.tex` files to one sentence per line after edit/write/patch tools.

## What it does

- Hooks `tool.execute.after`.
- Detects touched `.tex` files from common edit/write/patch arguments.
- Preserves LaTeX comments beginning with `%`.
- Inserts a newline after `.`, `!`, or `?` when the next token starts with an uppercase letter or LaTeX command.
- Restricts absolute paths to the current opencode directory/worktree.
- Skips vendor/build/generated-ish path segments.

## Install

```json
{
  "plugin": [
    "opencode-latex-sentence-per-line-harness"
  ]
}
```

Restart opencode after changing `opencode.json`.

## Caveat

The splitter is intentionally simple. Review diffs after prose-heavy `.tex` edits and manually repair awkward splits around abbreviations, citations, math-adjacent prose, commands, or tables.
