"""Reject generated artifacts in the Git index without historical exemptions.

Run after staging changes. CI checks the checked-out index too; .gitignore alone
cannot prevent forced additions. Local untracked outputs are not source changes.
"""

from pathlib import Path, PurePosixPath
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
GENERATED_PARTS = {"target", "artifacts", "__pycache__", ".venv", "node_modules"}
GENERATED_SUFFIXES = (".log", ".gz", ".zip", ".tar", ".xz", ".zst", ".pyc",
                      ".o", ".a", ".so", ".dll", ".exe", ".rlib", ".rmeta", ".crate")


def git(root, *args):
    return subprocess.check_output(["git", *args], cwd=root, stderr=subprocess.PIPE)


def generated(path):
    return (path.startswith("benches/results/")
            or bool(GENERATED_PARTS.intersection(PurePosixPath(path).parts))
            or path.lower().endswith(GENERATED_SUFFIXES))


def check(root=ROOT):
    errors = []
    for entry in git(root, "ls-files", "--stage", "-z").split(b"\0"):
        if not entry:
            continue
        metadata, path = entry.decode().split("\t", 1)
        _mode, _oid, stage = metadata.split()
        if stage != "0":
            errors.append(f"{path}: unresolved index entry")
        elif generated(path):
            errors.append(f"{path}: generated output belongs in target/ or external artifact storage")
    return errors


def main():
    try:
        errors = check()
    except (OSError, subprocess.CalledProcessError) as error:
        print(f"Artifact check could not read the Git index: {error}. "
              "Run in a Git checkout and stage changes before checking.", file=sys.stderr)
        return 1
    if errors:
        print("\n".join(errors[:10]), file=sys.stderr)
        print(f"Artifact check failed: {len(errors)} index entries.", file=sys.stderr)
        return 1
    print("Artifact check passed: no generated output in the Git index.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
