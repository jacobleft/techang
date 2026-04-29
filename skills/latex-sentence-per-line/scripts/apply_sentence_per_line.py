#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path


PATTERN = re.compile(r"([.!?])\s+(?=[A-Z\\])")


def split_code_and_comment(line: str) -> tuple[str, str]:
    escaped = False
    for index, char in enumerate(line):
        if char == "\\" and not escaped:
            escaped = True
            continue
        if char == "%" and not escaped:
            return line[:index], line[index:]
        escaped = False
    return line, ""


def split_sentences(text: str) -> str:
    output: list[str] = []
    for line in text.splitlines(keepends=True):
        line_body = line[:-1] if line.endswith("\n") else line
        newline = "\n" if line.endswith("\n") else ""
        code, comment = split_code_and_comment(line_body)
        output.append(PATTERN.sub(lambda match: match.group(1) + "\n", code) + comment + newline)
    return "".join(output)


def process_file(path: Path) -> None:
    if not path.exists():
        raise FileNotFoundError(f"File not found: {path}")
    if not path.is_file():
        raise IsADirectoryError(f"Not a file: {path}")

    original = path.read_text(encoding="utf-8")
    updated = split_sentences(original)
    path.write_text(updated, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Rewrite LaTeX prose files so each sentence is placed on its own line."
    )
    parser.add_argument("files", nargs="+", type=Path, help="One or more files to rewrite")
    args = parser.parse_args()

    try:
        for path in args.files:
            process_file(path)
    except (FileNotFoundError, IsADirectoryError) as exc:
        print(str(exc), file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
