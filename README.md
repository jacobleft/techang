# techang （特长）

**Specialty agent skills** for Claude Code, OpenCode, Cursor — anything the Vercel `skills` CLI supports.

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
└── skills/
    ├── intimate-relationship-guide/
    ├── latex-optimizer/
    ├── latex-sentence-per-line/
    └── ocr-tiered/
```
