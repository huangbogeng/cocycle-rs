"""Check maintained text sources without rewriting files or importing tools.

This is a whitespace/encoding and Python syntax check, not a language formatter.
Raw benchmark evidence, downloaded sources and build output are excluded.
"""

import ast
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
DIRECTORIES = ("src", "tests", "examples", "docs", "tools", "assets", ".github", "benches")
SUFFIXES = {".rs", ".py", ".cpp", ".hpp", ".md", ".toml", ".json", ".yml", ".yaml", ".svg"}
ROOT_FILES = {".editorconfig", ".gitattributes", ".gitignore", "Cargo.lock", "LICENSE"}


def sources():
    paths = {path for path in ROOT.iterdir()
             if path.is_file() and (path.suffix in SUFFIXES or path.name in ROOT_FILES)}
    for directory in DIRECTORIES:
        for path in (ROOT / directory).rglob("*"):
            relative = path.relative_to(ROOT)
            if relative.parts[:2] == ("benches", "results"):
                continue
            if path.is_file() and path.suffix in SUFFIXES:
                paths.add(path)
    return sorted(paths)


def check(path):
    relative = path.relative_to(ROOT)
    try:
        text = path.read_bytes().decode("utf-8")
    except UnicodeDecodeError:
        return [f"{relative}: expected UTF-8"]
    errors = []
    if text.startswith("\ufeff"):
        errors.append(f"{relative}: unexpected UTF-8 BOM")
    if "\r" in text:
        errors.append(f"{relative}: expected LF line endings")
    if text and not text.endswith("\n"):
        errors.append(f"{relative}: missing final newline")
    for number, line in enumerate(text.splitlines(), 1):
        if line.rstrip(" \t") != line:
            errors.append(f"{relative}:{number}: trailing whitespace")
        indentation = line[:len(line) - len(line.lstrip(" \t"))]
        if "\t" in indentation:
            errors.append(f"{relative}:{number}: use spaces for indentation")
    if path.suffix == ".py":
        try:
            ast.parse(text, filename=str(relative))
        except SyntaxError as error:
            errors.append(f"{relative}:{error.lineno}: Python syntax: {error.msg}")
    return errors


def main():
    paths = sources()
    errors = [error for path in paths for error in check(path)]
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"Source checks passed: {len(paths)} files; encoding, whitespace and Python syntax.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
