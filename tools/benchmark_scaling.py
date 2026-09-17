"""Bounded H1 scaling experiments; reuse the established cross-library workers.

Linux only: every worker has a wall-time limit and an RLIMIT_AS address-space
limit. Limits, failures and planned omissions are retained, never ranked as wins.
Python libraries remain development-only dependencies.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import importlib.metadata
import json
import math
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess
import sys
import tempfile

import benchmark_gudhi as bench

FAMILIES = ("uniform", "circle", "sphere8", "nonmetric", "grid", "bipartite")


def lcg(count):
    state = 1729
    for _ in range(count):
        state = (state * 6364136223846793005 + 1) & ((1 << 64) - 1)
        yield (state >> 11) / (1 << 53)


def make_case(family, n, cutoff=None):
    """Deterministic, explicit input families, quantized once after construction."""
    if n < 2:
        raise ValueError("scaling fixtures need at least two vertices")
    d = 2
    if family == "uniform":
        values = bench.distances(bench.uniform(n))
    elif family == "circle":
        points = [[math.cos(2 * math.pi * i / n), math.sin(2 * math.pi * i / n)] for i in range(n)]
        values = bench.distances(points)
    elif family == "sphere8":
        # Normalized cube samples on S^7 in R^8, NOT uniform sphere samples.
        d = 8
        rng = iter(lcg(d * n))
        points = []
        for _ in range(n):
            row = [2 * next(rng) - 1 for _ in range(d)]
            norm = math.hypot(*row)
            points.append([x / norm for x in row])
        values = bench.distances(points)
    elif family == "nonmetric":
        values = [(int(x * 64) + 1) / 64 for x in lcg(n * (n - 1) // 2)]
    elif family == "grid":
        side = math.isqrt(n)
        if side * side != n:
            raise ValueError("grid needs a square vertex count")
        values = bench.distances([[i / (side - 1), j / (side - 1)]
                                 for i in range(side) for j in range(side)])
    elif family == "bipartite":
        # K_(a,b) at t=1; all remaining edges and the full simplex at t=2.
        half = n // 2
        values = [1. if (i < half) != (j < half) else 2. for i in range(n) for j in range(i)]
    else:
        raise ValueError("unknown family")
    suffix = "_cutoff" if cutoff is not None else ""
    return bench.float32_case(bench.Case(f"{family}_h1_{n}{suffix}", n, values, cutoff=cutoff, d=d))


def case_specs(quick=False):
    for family in FAMILIES:
        if family in ("uniform", "circle"):
            sizes = [16, 32] if quick else [128, 256, 512, 1024]
        elif family == "grid":
            sizes = [16, 36] if quick else [64, 144, 256]
        elif family == "bipartite":
            sizes = [16, 32] if quick else [64, 128, 256]
        else:
            sizes = [16, 32] if quick else [128, 256, 512]
        for n in sizes:
            for cutoff in ([None, 1.] if family == "bipartite" else [None]):
                yield family, n, cutoff


def bipartite_diagram(n, cutoff):
    """Independent analytic H0/H1 multiset; connected graph has E-V+1 cycles."""
    a, b = n // 2, n - n // 2
    if cutoff not in (None, 1.):
        raise ValueError("analytic fixture supports full range or cutoff=1")
    full = cutoff is None or n == 2
    intervals = [[0, 0., "F", 1.] for _ in range(n - 1)]
    intervals.append([0, 0., "E" if full else "C", 0. if full else 1.])
    intervals.extend([[1, 1., "F" if full else "C", 2. if full else 1.]
                     for _ in range((a - 1) * (b - 1))])
    return {"coverage": ["complete", None] if full else ["through", 1.], "intervals": intervals}


def worker(backend, driver, fixture, samples, timeout, address_space_mib, env):
    command = ([str(driver), str(fixture), str(samples), "1"] if backend == "cocycle" else
               [sys.executable, str(Path(bench.__file__).resolve()), "--worker", backend,
                str(fixture), str(samples), "1"])
    # exec replaces the wrapper; subprocess timeout kills the actual worker.
    command = [sys.executable, str(Path(__file__).resolve()), "--limited-worker",
               str(address_space_mib), *command]
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=timeout, env=env)
    except subprocess.TimeoutExpired:
        return {"status": "timeout", "timeout_seconds": timeout}
    if result.returncode:
        return {"status": "process_error", "returncode": result.returncode,
                "stderr": result.stderr[-8000:], "stdout": result.stdout[-2000:]}
    try:
        data = json.loads(result.stdout)
        times = data["samples_ms"]
        if len(times) != samples or not all(math.isfinite(t) and t > 0 for t in times):
            raise ValueError("invalid samples")
        if "intervals" not in data or "coverage" not in data:
            raise ValueError("missing diagram")
        return dict(data, status="completed")
    except (ValueError, KeyError, TypeError) as exc:
        return {"status": "protocol_error", "error": repr(exc), "stdout": result.stdout[-2000:]}


def validate_record(record, analytic=None):
    """Every completed diagram is checked, including when Cocycle timed out."""
    completed = [(k, r) for k, r in record["results"].items() if r["status"] == "completed"]
    record["comparisons"] = []
    record["validation"] = "unverified"
    if not completed:
        return
    anchor_name, anchor = completed[0]
    if analytic is not None:
        bench.compare(anchor, analytic, "distances")
        record["comparisons"].append([anchor_name, "analytic"])
    for name, result in completed[1:]:
        bench.compare(anchor, result, "distances")
        record["comparisons"].append([anchor_name, name])
    if record["comparisons"]:
        record["validation"] = "passed"


def summarize(path, records):
    import csv
    rows = []
    for record in records:
        native = record["results"].get("cocycle", {})
        for backend, result in record["results"].items():
            measured = result["status"] == "completed"
            verified = measured and record["validation"] == "passed"
            before, peak = result.get("hwm_before_kib"), result.get("peak_rss_kib")
            rows.append({"case": record["case"], "n": record["n"], "backend": backend,
                         "status": result["status"], "validation": record["validation"],
                         "median_ms": statistics.median(result["samples_ms"]) if verified else None,
                         "min_ms": min(result["samples_ms"]) if verified else None,
                         "max_ms": max(result["samples_ms"]) if verified else None,
                         "cocycle_over_backend": (statistics.median(native["samples_ms"]) /
                                                  statistics.median(result["samples_ms"]))
                         if verified and native.get("status") == "completed" else None,
                         "rss_before_kib": result.get("rss_before_kib"), "peak_rss_kib": peak,
                         "hwm_growth_kib": max(0, peak - before) if peak is not None and before is not None else None,
                         "intervals": len(result["intervals"]) if measured else None,
                         "simplices": result.get("simplices"), "reason": result.get("reason")})
    if rows:
        with path.open("w", newline="") as stream:
            writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
            writer.writeheader()
            writer.writerows(rows)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--quick", action="store_true")
    parser.add_argument("--families", nargs="+", choices=FAMILIES, default=list(FAMILIES))
    parser.add_argument("--samples", type=int, default=3)
    parser.add_argument("--timeout", type=float, default=60.)
    parser.add_argument("--address-space-mib", type=int, default=2048)
    parser.add_argument("--gudhi-max-n", type=int, default=256)
    parser.add_argument("--cpu", type=int)
    args = parser.parse_args()
    if sys.platform != "linux":
        parser.error("this bounded experiment requires Linux RLIMIT_AS")
    if args.samples < 1 or not math.isfinite(args.timeout) or args.timeout <= 0 or args.address_space_mib < 1 or args.gudhi_max_n < 0:
        parser.error("invalid sample count, timeout or resource limit")
    if args.cpu is not None:
        if args.cpu not in os.sched_getaffinity(0):
            parser.error("CPU is not available")
        os.sched_setaffinity(0, {args.cpu})
    args.output.mkdir(parents=True, exist_ok=False)
    fixtures = args.output / "fixtures"
    fixtures.mkdir()
    (args.output / "manifest-at-measurement.toml").write_bytes((bench.REPO / "Cargo.toml").read_bytes())
    threads = {k: "1" for k in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS", "NUMEXPR_NUM_THREADS")}
    env = dict(os.environ, **threads)
    cpu = next((x.split(":", 1)[1].strip() for x in Path("/proc/cpuinfo").read_text().splitlines()
                if x.startswith("model name")), "unknown")
    metadata = {"created_utc": datetime.now(timezone.utc).isoformat(), "source_sha256": bench.source_hash(),
                "scaling_script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "rustc": subprocess.check_output(["rustc", "-Vv"], text=True), "rustflags": env.get("RUSTFLAGS"),
                "python": sys.version, "platform": platform.platform(), "cpu": cpu,
                "affinity": sorted(os.sched_getaffinity(0)), "threads": threads,
                "packages": {p: importlib.metadata.version(p) for p in ("gudhi", "numpy", "ripser", "scipy", "scikit-learn")},
                "build": "release library, rustc -O driver; no target-cpu override",
                "precision": "float32-exact distances and cutoff; f64 Cocycle/GUDHI kernels",
                "samples": args.samples, "warmups": 1, "iterations": 1,
                "timeout_seconds": args.timeout, "address_space_mib": args.address_space_mib,
                "limit_scope": "whole worker including runtime/imports/output; time limit includes all samples and startup",
                "gudhi_max_n": args.gudhi_max_n, "quick": args.quick, "families": args.families,
                "seed": 1729, "order_seed": 20260917, "status": "running"}
    bench.save_json(args.output / "environment.json", metadata)
    records = []
    order_rng = random.Random(metadata["order_seed"])
    failed = False
    try:
        with tempfile.TemporaryDirectory(prefix="cocycle-scaling-") as temp:
            driver = bench.build_driver(Path(temp))
            for family, n, cutoff in case_specs(args.quick):
                if family not in args.families:
                    continue
                case = make_case(family, n, cutoff)
                fixture = fixtures / f"{case.name}.bin"
                case.write(fixture)
                order = [*bench.BACKENDS, bench.RIPSER_BACKEND]
                order_rng.shuffle(order)
                record = {"case": case.name, "family": family, "n": n, "cutoff": cutoff,
                          "fixture_sha256": hashlib.sha256(fixture.read_bytes()).hexdigest(),
                          "order": order, "results": {}, "validation": "unverified"}
                records.append(record)
                for backend in order:
                    if backend.startswith("gudhi_") and n > args.gudhi_max_n:
                        record["results"][backend] = {"status": "not_scheduled",
                            "reason": f"predeclared GUDHI limit n <= {args.gudhi_max_n}"}
                    else:
                        print(f"{case.name}: {backend}", flush=True)
                        record["results"][backend] = worker(backend, driver, fixture, args.samples,
                                                            args.timeout, args.address_space_mib, env)
                    bench.save_json(args.output / "results.json", records)
                try:
                    validate_record(record, bipartite_diagram(n, cutoff) if family == "bipartite" else None)
                except AssertionError as exc:
                    record["validation"] = "mismatch"
                    record["error"] = repr(exc)
                    failed = True
                if any(r["status"] in ("process_error", "protocol_error") for r in record["results"].values()):
                    failed = True
                bench.save_json(args.output / "results.json", records)
                summarize(args.output / "summary.csv", records)
        constrained = any(r["status"] == "timeout" for c in records for r in c["results"].values())
        unverified = any(c["validation"] == "unverified" for c in records)
        metadata["status"] = "failed" if failed else "completed_with_limits" if constrained or unverified else "passed"
        metadata["case_count"] = len(records)
        metadata["backend_status_counts"] = {status: sum(r["status"] == status for c in records for r in c["results"].values())
                                              for status in ("completed", "timeout", "process_error", "protocol_error", "not_scheduled")}
        metadata["comparisons_passed"] = sum(len(c.get("comparisons", [])) for c in records if c["validation"] == "passed")
    finally:
        if metadata["status"] == "running":
            metadata["status"] = "interrupted_or_failed"
        bench.save_json(args.output / "environment.json", metadata)
        summarize(args.output / "summary.csv", records)
    print(f"{metadata['status']}: {len(records)} cases; {args.output}")
    return int(failed)


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--limited-worker":
        import resource
        limit = int(sys.argv[2]) * 1024 * 1024
        resource.setrlimit(resource.RLIMIT_AS, (limit, limit))
        os.execvpe(sys.argv[3], sys.argv[3:], os.environ)
    else:
        raise SystemExit(main())
