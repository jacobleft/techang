---
name: ocr-tiered
description: This skill should be used when the user asks to OCR a local image or PDF, compare PaddleOCR against Tesseract variants, extract text from scanned pages, or choose the best OCR tier for noisy Chinese or mixed Chinese-English documents.
---

# Tiered OCR

## Goal

Run a repeatable OCR workflow with three tiers:

1. **PaddleOCR** with the best tested quality-oriented settings
2. **Tesseract + `tessdata_best`** downloaded on demand from `tesseract-ocr/tessdata_best`
3. **Default Tesseract** using the local system models

The bundled script is intended for **text extraction and engine comparison** on local images or PDFs.

## Use this when

- the user wants OCR on a scanned image or PDF
- the user wants to compare OCR engines on the same page
- the user wants a fallback chain instead of trusting one OCR engine
- the user wants Chinese or mixed Chinese-English OCR and needs better quality than default Tesseract

## Boundaries

- This skill outputs **text files + JSON summaries**, not searchable PDFs
- It does **not** overwrite source PDFs by itself
- It assumes `tesseract` and `pdftoppm` are available locally for tiers 2/3 and PDF rendering
- Tier 1 requires a Python environment with `paddleocr` and `paddlepaddle`

## Setup

Create a dedicated PaddleOCR environment once:

```bash
bash scripts/setup_paddleocr_env.sh
```

That creates `.venv-paddleocr/` in this skill directory and installs `paddlepaddle` + `paddleocr`.

## Default workflow

1. Use the PaddleOCR venv for script execution.
2. Run the OCR script in **compare** mode on one representative page first.
3. Inspect the generated tier outputs and summary JSON.
4. If one tier is clearly best, rerun in **cascade** or single-tier mode for the full input set.

## Commands

Compare all three tiers on an image:

```bash
.venv-paddleocr/bin/python scripts/ocr_tiered.py compare /absolute/path/to/page.png
```

Compare all three tiers on a PDF, page 100 only:

```bash
.venv-paddleocr/bin/python scripts/ocr_tiered.py compare /absolute/path/to/file.pdf --pages 100
```

Run the fallback order and stop on first good result:

```bash
.venv-paddleocr/bin/python scripts/ocr_tiered.py cascade /absolute/path/to/file.pdf --pages 100
```

Run a single tier only:

```bash
.venv-paddleocr/bin/python scripts/ocr_tiered.py tier1 /absolute/path/to/page.png
.venv-paddleocr/bin/python scripts/ocr_tiered.py tier2 /absolute/path/to/page.png
.venv-paddleocr/bin/python scripts/ocr_tiered.py tier3 /absolute/path/to/page.png
```

## Outputs

The script writes an output directory next to the input by default:

- `tier1_paddleocr.txt`
- `tier2_tesseract_best.txt`
- `tier3_tesseract_default.txt`
- `summary.json`

For PDF input it renders requested pages to temporary PNGs first.

## Notes

- Tier 1 uses the strongest tested local config so far:
  - `lang="ch"`
  - `use_doc_orientation_classify=True`
  - `use_doc_unwarping=True`
  - `use_textline_orientation=True`
  - `text_det_limit_type="max"`
  - `text_det_limit_side_len=1216`
  - `text_det_thresh=0.3`
  - `text_det_box_thresh=0.5`
  - `text_det_unclip_ratio=1.5`
- Tier 2 downloads only the requested `.traineddata` files into `~/.cache/ocr-tiered/tessdata_best/`
- Tier 3 uses whatever `tesseract` sees by default on the machine
