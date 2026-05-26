# techang opencode plugin index

Installable opencode plugins published from this repository.

## Plugins

| Directory | Package | Purpose |
|---|---|---|
| `opencode-zotero-citation-guard/` | `opencode-zotero-citation-guard` | Adds citation-integrity reminders and warns on unverified citekeys, missing literature notes, or bibliography output without retraction checks. |
| `opencode-latex-sentence-per-line-harness/` | `opencode-latex-sentence-per-line-harness` | Automatically rewrites touched `.tex` files to one sentence per line after edit/write/patch tools. |

## Publish

From the repository root:

```bash
npm install
npm run check
npm run build
npm publish --workspace opencode-zotero-citation-guard --access public
npm publish --workspace opencode-latex-sentence-per-line-harness --access public
```
