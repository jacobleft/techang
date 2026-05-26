# techang （特长）

**Specialty agent skills and opencode plugins** for Claude Code, OpenCode, Cursor, and related AI coding tools.

[中文版](README.cn.md)

---

## Install

```bash
# All skills
npx skills add jacobleft/techang

# Preview before installing
npx skills add jacobleft/techang --list

# One skill only
npx skills add jacobleft/techang --skill latex-optimizer
```

---

## Skills

| Skill | What it does |
|:------|:-------------|
| **intimate-relationship-guide** | Relationship advice backed by scientific psychology — attraction, communication, conflict, maintenance, breakup recovery |
| **latex-optimizer** | Noise-free LaTeX compilation for AI agents; auto-detects `texfot`, `pplatex`, `latexmk`, `tectonic` |
| **latex-sentence-per-line** | Normalizes LaTeX prose to one sentence per line |
| **ocr-tiered** | Tiered OCR: PaddleOCR (highest accuracy, Python deps) → Tesseract + `tessdata_best` (balanced) → default Tesseract (fast, lightweight) |

```
techang/
├── README.md
├── plugins/
│   ├── opencode-latex-sentence-per-line-harness/
│   └── opencode-zotero-citation-guard/
└── skills/
    ├── intimate-relationship-guide/
    ├── latex-optimizer/
    ├── latex-sentence-per-line/
    └── ocr-tiered/
```

---

## OpenCode plugins

| Package | What it does |
|:--------|:-------------|
| **opencode-zotero-citation-guard** | Guards Zotero citations, citekeys, literature-note checks, and bibliography retraction-check reminders |
| **opencode-latex-sentence-per-line-harness** | Automatically rewrites touched `.tex` files to one sentence per line after edit/write/patch tools |

Install after publishing:

```json
{
  "plugin": [
    "opencode-zotero-citation-guard",
    "opencode-latex-sentence-per-line-harness"
  ]
}
```

Restart opencode after changing `opencode.json`.
