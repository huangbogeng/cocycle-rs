"""Run focused contributor checks from a source checkout, without native tools."""

import argparse
import json
from pathlib import Path
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def run(command, **kwargs):
    print("+ " + shlex.join(map(str, command)), flush=True)
    return subprocess.run(command, cwd=ROOT, check=True, text=True, **kwargs)


def library_artifact():
    # Ask Cargo for the actual path so CARGO_TARGET_DIR and platform-specific
    # paths do not require a second copy of Cargo's configuration logic.
    result = run(["cargo", "build", "--locked", "--lib", "--message-format=json-render-diagnostics"],
                 stdout=subprocess.PIPE)
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if (message.get("reason") == "compiler-artifact"
                and message.get("target", {}).get("name") == "cocycle"):
            for filename in message.get("filenames", []):
                if filename.endswith(".rlib"):
                    return Path(filename)
    raise ValueError("Cargo did not report the cocycle library artifact")


def check_diagram_analysis():
    run([sys.executable, "tools/check_source.py"])
    run([sys.executable, "tools/check_docs.py"])
    sources = [*sorted((ROOT / "src/descriptors").glob("*.rs")),
               *sorted((ROOT / "src/diagram").glob("*.rs")),
               ROOT / "tests/descriptors.rs", ROOT / "tests/contracts.rs",
               ROOT / "examples/diagram_analysis.rs"]
    run(["rustfmt", "--edition", "2024", "--check",
         *(str(path.relative_to(ROOT)) for path in sources)])
    run(["cargo", "clippy", "--locked", "--all-features", "--lib",
         "--test", "descriptors", "--test", "contracts", "--example", "diagram_analysis",
         "--", "-D", "warnings"])
    run(["cargo", "test", "--locked", "--all-features",
         "--test", "descriptors", "--test", "contracts"])
    run(["cargo", "run", "--locked", "--example", "diagram_analysis"])
    library = library_artifact()
    run(["rustdoc", "--edition", "2024", "-D", "warnings", "--test",
         "docs/development/diagram-analysis.md", "--extern", f"cocycle={library}",
         "-L", f"dependency={library.parent / 'deps'}"])


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("domain", choices=["diagram-analysis"])
    parser.parse_args(argv)
    try:
        check_diagram_analysis()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Algorithm checks failed: {error}", file=sys.stderr)
        return 1
    print("Diagram-analysis checks passed. Full maintainer/CI checks remain separate.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
