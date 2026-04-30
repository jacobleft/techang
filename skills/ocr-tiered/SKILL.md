---
name: ocr-tiered
description: This skill should be used when the user asks to OCR a local image or PDF, extract text from scanned pages, or needs OCR for Chinese or mixed Chinese-English documents. Uses a tiered approach — picks the best available engine based on what's installed and what the user prioritizes (accuracy, balanced, or speed).
---

# Tiered OCR

Pick the right OCR engine based on **what's available** and **what the user prioritizes**.

## The three tiers

| Tier | Engine | When to use | Trade-offs |
|:-----|:-------|:------------|:-----------|
| **1 — Accuracy** | PaddleOCR | Maximum accuracy, especially for Chinese/mixed CJK | Requires Python venv with `paddlepaddle` + `paddleocr` (~1 GB) |
| **2 — Balanced** | Tesseract + `tessdata_best` | Good accuracy without heavy Python deps | Downloads best `.traineddata` on first use; needs `tesseract` on PATH |
| **3 — Fast** | Default Tesseract | Lightweight, no extra downloads, fast | Lower accuracy; uses whatever model ships with the system |

### Decision guide

- **User wants best quality → Tier 1** (if PaddleOCR venv exists or user is okay installing it)
- **User wants good quality but no Python deps → Tier 2**
- **User wants it fast / already has tesseract → Tier 3**
- **User is unsure → `compare` mode** on one representative page, then pick the winner for the rest

## Use this when

- User wants OCR on a scanned image or PDF
- User wants Chinese or mixed Chinese-English OCR
- User has strong accuracy/speed preferences
- User doesn't know what OCR tools are available (cascade finds the best one that works)

## Boundaries

- Outputs **text files + JSON summaries**, not searchable PDFs
- Does **not** overwrite source files
- Tiers 2/3 require `tesseract` on PATH; PDF input also needs `pdftoppm`
- Tier 1 requires a dedicated Python venv (setup below)

## Setup (Tier 1 only)

```bash
bash scripts/setup_paddleocr_env.sh
```

Creates `.venv-paddleocr/` in this skill directory with `paddlepaddle` + `paddleocr`.

## Commands

### Single tier — when you know which one to use

```bash
# Tier 1: best accuracy
.venv-paddleocr/bin/python scripts/ocr_tiered.py tier1 /path/to/page.png

# Tier 2: balanced (no Python venv needed, uses system tesseract)
python3 scripts/ocr_tiered.py tier2 /path/to/page.png

# Tier 3: fast and lightweight
python3 scripts/ocr_tiered.py tier3 /path/to/page.png
```

### Compare — test all tiers on one page, pick the best

```bash
.venv-paddleocr/bin/python scripts/ocr_tiered.py compare /path/to/page.png
```

### Cascade — try tier 1 → 2 → 3, stop on first good result

```bash
.venv-paddleocr/bin/python scripts/ocr_tiered.py cascade /path/to/file.pdf --pages 100
```

### PDF support

All modes accept PDFs. Use `--pages` to target specific pages:

```bash
.venv-paddleocr/bin/python scripts/ocr_tiered.py tier1 /path/to/file.pdf --pages 1,3-5
```

## Outputs

Written to a sibling directory (`<stem>.ocr-tiered/`) by default:

- `tier1_paddleocr.txt`
- `tier2_tesseract_best.txt`
- `tier3_tesseract_default.txt`
- `summary.json`

## Tier 1 config details

PaddleOCR runs with quality-oriented defaults:

- `lang="ch"`, `use_doc_orientation_classify=True`, `use_doc_unwarping=True`
- `use_textline_orientation=True`, `text_det_limit_type="max"`, `text_det_limit_side_len=1216`
- `text_det_thresh=0.3`, `text_det_box_thresh=0.5`, `text_det_unclip_ratio=1.5`

Tier 2 caches downloaded `.traineddata` in `~/.cache/ocr-tiered/tessdata_best/`.

Tier 3 uses whatever `tesseract` ships with on the system — no extra downloads.
