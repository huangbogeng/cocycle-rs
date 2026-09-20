"""Diagnose verified scaling fixtures with private, instrumented release tests.

Never executes the old cubic boundary reference on large inputs. Instead, checks
the instrumented diagram against the same-source, independently validated public
benchmark result. Counters are diagnostic; timings come only from that benchmark.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

import benchmark_gudhi as bench
import benchmark_scaling as scaling


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--benchmark", type=Path, required=True)
    parser.add_argument("--cases", nargs="+", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=60.)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("output must be new")
    metadata = json.loads((args.benchmark / "environment.json").read_text())
    if metadata["source_sha256"] != bench.source_hash():
        parser.error("benchmark must match current source")
    records = {r["case"]: r for r in json.loads((args.benchmark / "results.json").read_text())}
    for name in args.cases:
        if name not in records or records[name]["validation"] != "passed" or records[name]["results"]["cocycle"]["status"] != "completed":
            parser.error(f"{name} has no independently validated Cocycle result")
    if metadata["affinity"] is not None:
        os.sched_setaffinity(0, set(metadata["affinity"]))
    build = subprocess.check_output([
        "cargo", "test", "--release", "--locked", "--offline", "--lib", "--no-run", "--message-format=json",
    ], cwd=bench.REPO, text=True)
    artifacts = [json.loads(line) for line in build.splitlines()]
    executable, = [a["executable"] for a in artifacts if a.get("reason") == "compiler-artifact" and a.get("executable")]
    output = {"source_sha256": bench.source_hash(), "benchmark": str(args.benchmark),
              "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "status": "running", "results": []}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    try:
        for name in args.cases:
            print(name, flush=True)
            fixture = args.benchmark / "fixtures" / f"{name}.bin"
            if hashlib.sha256(fixture.read_bytes()).hexdigest() != records[name]["fixture_sha256"]:
                raise ValueError("fixture hash mismatch")
            env = dict(os.environ, COCYCLE_ABLATION_FIXTURE=str(fixture.resolve()), **metadata["threads"])
            command = [sys.executable, str(Path(scaling.__file__).resolve()), "--limited-worker",
                       str(metadata["address_space_mib"]), executable,
                       "persistence::rips::cohomology::profiling::profile_workload", "--exact", "--ignored", "--nocapture"]
            result = subprocess.check_output(command, cwd=bench.REPO, env=env, text=True, timeout=args.timeout)
            row, = [json.loads(line.split("WORKLOAD ", 1)[1]) for line in result.splitlines() if "WORKLOAD " in line]
            bench.compare(row, records[name]["results"]["cocycle"], "distances")
            output["results"].append({"case": name, "fixture_sha256": records[name]["fixture_sha256"],
                                      "statistics": row["statistics"], "diagram_comparison": "passed"})
            bench.save_json(args.output, output)
        output["status"] = "passed"
    finally:
        if output["status"] == "running":
            output["status"] = "failed"
        bench.save_json(args.output, output)


if __name__ == "__main__":
    main()
