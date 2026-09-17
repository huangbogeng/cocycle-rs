"""Check repository Markdown for stale local links and non-English source prose.

Uses only the standard library. External URLs are not fetched; dated raw artifacts
and generated output are not documentation sources. Run from any directory.
"""

from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
PATTERNS = ("*.md", "docs/*.md", "benches/*.md", "tools/*.md", "assets/*.md", ".github/**/*.md")
LINK = re.compile(r"!?\[[^\]\n]*\]\(([^\s)]+)(?:\s+\"[^\"]*\")?\)")
CJK = re.compile(r"[\u3400-\u4dbf\u4e00-\u9fff]")


def prose(text):
    """Remove fenced code before interpreting inline Markdown links."""
    return re.sub(r"^(`{3,}|~{3,})[^\n]*\n.*?^\1[^\n]*$", "", text,
                  flags=re.MULTILINE | re.DOTALL)


def anchors(text):
    used = {}
    result = set(re.findall(r'<a\s+(?:id|name)=["\']([^"\']+)', text))
    for heading in re.findall(r"^#{1,6}\s+(.+?)\s*#*\s*$", prose(text), re.MULTILINE):
        slug = re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-")
        count = used.get(slug, 0)
        used[slug] = count + 1
        result.add(f"{slug}-{count}" if count else slug)
    return result


def check(path):
    text = path.read_text(encoding="utf-8")
    errors = []
    for line, content in enumerate(text.splitlines(), 1):
        if CJK.search(content):
            errors.append(f"{path.relative_to(ROOT)}:{line}: project documentation must be English")
    for match in LINK.finditer(prose(text)):
        link = urlsplit(match[1].strip("<>"))
        if link.scheme or link.netloc:
            continue
        target = (path.parent / unquote(link.path)).resolve() if link.path else path
        if not target.exists():
            errors.append(f"{path.relative_to(ROOT)}: missing local target {match[1]}")
        elif link.fragment and target.suffix == ".md":
            if unquote(link.fragment) not in anchors(target.read_text(encoding="utf-8")):
                errors.append(f"{path.relative_to(ROOT)}: missing heading {match[1]}")
    return errors


def main():
    paths = sorted({p for pattern in PATTERNS for p in ROOT.glob(pattern) if p.is_file()})
    errors = [error for path in paths for error in check(path)]
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"Documentation checks passed: {len(paths)} Markdown files; local links and English prose.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
