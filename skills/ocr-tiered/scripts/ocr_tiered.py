#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.request
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable, Sequence


IMAGE_EXTS = {".png", ".jpg", ".jpeg", ".tif", ".tiff", ".bmp", ".webp"}
BEST_TESSDATA_BASE = "https://raw.githubusercontent.com/tesseract-ocr/tessdata_best/main"
VERTICAL_TESS_LANGS = {
    "chi_sim": "chi_sim_vert",
    "chi_tra": "chi_tra_vert",
    "jpn": "jpn_vert",
    "kor": "kor_vert",
}


class OCRTieredError(RuntimeError):
    pass


@dataclass
class TierResult:
    tier: str
    engine: str
    success: bool
    text_path: str | None
    nonspace_chars: int
    cjk_chars: int
    line_count: int
    stderr: str | None = None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Tiered OCR comparison for local images or PDFs."
    )
    parser.add_argument(
        "mode",
        choices=["compare", "cascade", "tier1", "tier2", "tier3"],
        help="compare all tiers, stop on first good tier, or run one tier only",
    )
    parser.add_argument("input_path", help="local image or PDF path")
    parser.add_argument(
        "--pages",
        help="PDF pages to OCR, e.g. 1,3-5. Omit to use all pages.",
    )
    parser.add_argument(
        "--output-dir",
        help="directory for OCR outputs (default: sibling '<stem>.ocr-tiered')",
    )
    parser.add_argument(
        "--langs",
        default="chi_sim+eng",
        help="Tesseract language string for tiers 2/3 (default: chi_sim+eng)",
    )
    parser.add_argument(
        "--paddle-lang",
        default="ch",
        help="PaddleOCR language selector for tier 1 (default: ch)",
    )
    parser.add_argument(
        "--psm",
        type=int,
        default=6,
        help="Tesseract page segmentation mode for tiers 2/3 (default: 6)",
    )
    parser.add_argument(
        "--cascade-min-chars",
        type=int,
        default=80,
        help="minimum non-space chars needed for cascade mode to accept a tier",
    )
    parser.add_argument(
        "--keep-rendered-images",
        action="store_true",
        help="keep temporary PNG renders for PDF input",
    )
    return parser.parse_args()


def parse_page_spec(spec: str) -> list[int]:
    pages: set[int] = set()
    for part in spec.split(","):
        part = part.strip()
        if not part:
            continue
        if "-" in part:
            start_s, end_s = part.split("-", 1)
            start = int(start_s)
            end = int(end_s)
            if start < 1 or end < start:
                raise OCRTieredError(f"Invalid page range: {part}")
            pages.update(range(start, end + 1))
        else:
            page = int(part)
            if page < 1:
                raise OCRTieredError(f"Invalid page number: {part}")
            pages.add(page)
    if not pages:
        raise OCRTieredError("Page specification resolved to an empty set")
    return sorted(pages)


def slugify(name: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]+", "_", name).strip("_") or "ocr_input"


def default_output_dir(input_path: Path) -> Path:
    return input_path.with_name(f"{input_path.stem}.ocr-tiered")


def ensure_tool(name: str) -> str:
    path = shutil.which(name)
    if not path:
        raise OCRTieredError(f"Required tool not found on PATH: {name}")
    return path


def prepare_images(input_path: Path, pages: Sequence[int] | None, keep: bool) -> tuple[list[tuple[str, Path]], Path | None]:
    suffix = input_path.suffix.lower()
    if suffix in IMAGE_EXTS:
        return [(input_path.name, input_path)], None

    if suffix != ".pdf":
        raise OCRTieredError(f"Unsupported input type: {input_path}")

    ensure_tool("pdftoppm")
    temp_dir = Path(tempfile.mkdtemp(prefix="ocr-tiered-"))
    images: list[tuple[str, Path]] = []

    def render_page(page_num: int) -> Path:
        prefix = temp_dir / f"page-{page_num:04d}"
        command = [
            "pdftoppm",
            "-f",
            str(page_num),
            "-l",
            str(page_num),
            "-singlefile",
            "-png",
            str(input_path),
            str(prefix),
        ]
        subprocess.run(command, check=True, capture_output=True, text=True)
        return prefix.with_suffix(".png")

    if pages:
        for page_num in pages:
            image_path = render_page(page_num)
            images.append((f"page-{page_num}", image_path))
    else:
        prefix = temp_dir / "page"
        command = ["pdftoppm", "-png", str(input_path), str(prefix)]
        subprocess.run(command, check=True, capture_output=True, text=True)
        for image_path in sorted(temp_dir.glob("page-*.png")):
            images.append((image_path.stem, image_path))

    if not images:
        raise OCRTieredError(f"No images rendered from PDF: {input_path}")
    return images, temp_dir if keep else temp_dir


def ensure_best_traineddata(langs: str, cache_root: Path) -> Path:
    cache_dir = cache_root / "tessdata"
    cache_dir.mkdir(parents=True, exist_ok=True)
    requested_langs = langs.split("+")
    extra_langs = [VERTICAL_TESS_LANGS[lang] for lang in requested_langs if lang in VERTICAL_TESS_LANGS]
    for lang in requested_langs + extra_langs:
        filename = f"{lang}.traineddata"
        target = cache_dir / filename
        if target.exists() and target.stat().st_size > 0:
            continue
        url = f"{BEST_TESSDATA_BASE}/{filename}"
        urllib.request.urlretrieve(url, target)
    return cache_dir


def join_page_texts(page_outputs: Iterable[tuple[str, str]]) -> str:
    chunks: list[str] = []
    for label, text in page_outputs:
        cleaned = text.strip()
        if cleaned:
            chunks.append(f"=== {label} ===\n{cleaned}")
        else:
            chunks.append(f"=== {label} ===")
    return "\n\n".join(chunks) + "\n"


def text_stats(text: str) -> tuple[int, int, int]:
    nonspace = len(re.sub(r"\s+", "", text))
    cjk = sum(1 for char in text if "\u4e00" <= char <= "\u9fff")
    lines = sum(1 for line in text.splitlines() if line.strip())
    return nonspace, cjk, lines


def write_text_output(output_dir: Path, filename: str, text: str) -> Path:
    path = output_dir / filename
    path.write_text(text, encoding="utf-8")
    return path


def run_paddleocr(images: Sequence[tuple[str, Path]], output_dir: Path, paddle_lang: str) -> TierResult:
    try:
        from paddleocr import PaddleOCR
    except ImportError as exc:  # pragma: no cover - depends on runtime env
        return TierResult(
            tier="tier1",
            engine="paddleocr",
            success=False,
            text_path=None,
            nonspace_chars=0,
            cjk_chars=0,
            line_count=0,
            stderr=f"PaddleOCR import failed: {exc}",
        )

    try:
        ocr = PaddleOCR(
            lang=paddle_lang,
            use_doc_orientation_classify=True,
            use_doc_unwarping=True,
            use_textline_orientation=True,
            text_det_limit_type="max",
            text_det_limit_side_len=1216,
            text_det_thresh=0.3,
            text_det_box_thresh=0.5,
            text_det_unclip_ratio=1.5,
        )
        pages: list[tuple[str, str]] = []
        for label, image_path in images:
            prediction = ocr.predict(str(image_path))
            if not prediction:
                pages.append((label, ""))
                continue
            payload = prediction[0].json
            if isinstance(payload, str):
                try:
                    payload = json.loads(payload)
                except json.JSONDecodeError:
                    pages.append((label, payload))
                    continue
            res = payload.get("res", payload)
            lines = res.get("rec_texts", []) if isinstance(res, dict) else []
            pages.append((label, "\n".join(lines)))
        text = join_page_texts(pages)
        nonspace, cjk, lines = text_stats(text)
        path = write_text_output(output_dir, "tier1_paddleocr.txt", text)
        return TierResult(
            tier="tier1",
            engine="paddleocr",
            success=nonspace > 0,
            text_path=str(path),
            nonspace_chars=nonspace,
            cjk_chars=cjk,
            line_count=lines,
        )
    except Exception as exc:  # pragma: no cover - engine dependent
        return TierResult(
            tier="tier1",
            engine="paddleocr",
            success=False,
            text_path=None,
            nonspace_chars=0,
            cjk_chars=0,
            line_count=0,
            stderr=str(exc),
        )


def run_tesseract(
    tier: str,
    engine_name: str,
    images: Sequence[tuple[str, Path]],
    output_dir: Path,
    langs: str,
    psm: int,
    tessdata_dir: Path | None,
) -> TierResult:
    ensure_tool("tesseract")
    pages: list[tuple[str, str]] = []
    stderr_chunks: list[str] = []

    for label, image_path in images:
        command = ["tesseract", str(image_path), "stdout", "--oem", "1", "-l", langs, "--psm", str(psm)]
        if tessdata_dir is not None:
            command.extend(["--tessdata-dir", str(tessdata_dir)])
        proc = subprocess.run(command, capture_output=True, text=True)
        stderr = proc.stderr.strip()
        if stderr:
            stderr_chunks.append(f"[{label}] {stderr}")
        if proc.returncode != 0:
            return TierResult(
                tier=tier,
                engine=engine_name,
                success=False,
                text_path=None,
                nonspace_chars=0,
                cjk_chars=0,
                line_count=0,
                stderr="\n".join(stderr_chunks) or proc.stderr,
            )
        pages.append((label, proc.stdout))

    text = join_page_texts(pages)
    nonspace, cjk, lines = text_stats(text)
    filename = "tier2_tesseract_best.txt" if tier == "tier2" else "tier3_tesseract_default.txt"
    path = write_text_output(output_dir, filename, text)
    return TierResult(
        tier=tier,
        engine=engine_name,
        success=nonspace > 0,
        text_path=str(path),
        nonspace_chars=nonspace,
        cjk_chars=cjk,
        line_count=lines,
        stderr="\n".join(stderr_chunks) or None,
    )


def run_requested_tiers(args: argparse.Namespace, images: Sequence[tuple[str, Path]], output_dir: Path) -> tuple[list[TierResult], str | None]:
    mode = args.mode
    results: list[TierResult] = []
    selected: str | None = None

    def tier1() -> TierResult:
        return run_paddleocr(images, output_dir, args.paddle_lang)

    def tier2() -> TierResult:
        cache_dir = ensure_best_traineddata(args.langs, Path.home() / ".cache" / "ocr-tiered" / "tessdata_best")
        return run_tesseract("tier2", "tesseract+tessdata_best", images, output_dir, args.langs, args.psm, cache_dir)

    def tier3() -> TierResult:
        return run_tesseract("tier3", "tesseract_default", images, output_dir, args.langs, args.psm, None)

    tier_map = {"tier1": tier1, "tier2": tier2, "tier3": tier3}

    if mode in tier_map:
        result = tier_map[mode]()
        results.append(result)
        selected = result.tier if result.success else None
        return results, selected

    ordered = [tier1, tier2, tier3]
    for runner in ordered:
        result = runner()
        results.append(result)
        if mode == "cascade" and result.success and result.nonspace_chars >= args.cascade_min_chars:
            selected = result.tier
            break

    if mode == "compare":
        best = max(results, key=lambda item: (item.nonspace_chars, item.cjk_chars, item.line_count), default=None)
        selected = best.tier if best and best.success else None
    return results, selected


def main() -> int:
    args = parse_args()
    input_path = Path(args.input_path).expanduser().resolve()
    if not input_path.exists():
        raise OCRTieredError(f"Input not found: {input_path}")

    pages = parse_page_spec(args.pages) if args.pages else None
    output_dir = Path(args.output_dir).expanduser().resolve() if args.output_dir else default_output_dir(input_path)
    output_dir.mkdir(parents=True, exist_ok=True)

    images, temp_dir = prepare_images(input_path, pages, args.keep_rendered_images)
    try:
        results, selected = run_requested_tiers(args, images, output_dir)
        summary = {
            "input": str(input_path),
            "mode": args.mode,
            "pages": pages,
            "selected_tier": selected,
            "rendered_images": [str(image) for _, image in images] if args.keep_rendered_images else [],
            "tiers": [asdict(result) for result in results],
        }
        summary_path = output_dir / "summary.json"
        summary_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
        print(f"Summary: {summary_path}")
        for result in results:
            status = "OK" if result.success else "FAIL"
            print(
                f"{result.tier}\t{status}\tnonspace={result.nonspace_chars}\tcjk={result.cjk_chars}\tlines={result.line_count}\t{result.text_path or '-'}"
            )
            if result.stderr:
                print(result.stderr, file=sys.stderr)
        return 0 if any(result.success for result in results) else 1
    finally:
        if temp_dir and not args.keep_rendered_images:
            shutil.rmtree(temp_dir, ignore_errors=True)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except OCRTieredError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        raise SystemExit(2)
