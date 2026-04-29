#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_DIR="${1:-$SKILL_DIR/.venv-paddleocr}"

python3 -m venv "$ENV_DIR"
"$ENV_DIR/bin/python" -m pip install -U pip
"$ENV_DIR/bin/python" -m pip install paddlepaddle paddleocr

printf 'PaddleOCR environment ready: %s\n' "$ENV_DIR"
printf 'Use it with:\n  %s/bin/python %s/ocr_tiered.py compare /absolute/path/to/input\n' "$ENV_DIR" "$SCRIPT_DIR"
