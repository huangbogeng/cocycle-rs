"""Serial Cocycle/GUDHI benchmark, optionally including Ripser.py.

Each backend/case runs in a fresh process. Worker clocks exclude startup and I/O.
Only development dependencies are needed; the Rust crate stays dependency-free.
"""

import argparse
from dataclasses import dataclass
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
import struct
import subprocess
import sys
import tempfile
import time

BACKENDS = ("cocycle", "gudhi_simplex_tree", "gudhi_edge_collapse")
RIPSER_BACKEND = "ripser_py"
HEADER = struct.Struct("<8sQQQQd")
REPO = Path(__file__).resolve().parents[1]


@dataclass
class Case:
    name: str
    n: int
    values: list
    q: int = 1
    cutoff: float | None = None
    mode: str = "distances"
    d: int = 2

    def write(self, path):
        import numpy as np

        path.write_bytes(
            HEADER.pack(b"COCYCLE1", int(self.mode == "points"), self.n, self.d,
                        self.q, math.nan if self.cutoff is None else self.cutoff)
            + np.asarray(self.values, dtype="<f8").tobytes()
        )


def uniform(n):
    # Identical generator to benches/rips.rs; no library RNG-version dependency.
    state = 1729
    values = []
    for _ in range(2 * n):
        state = (state * 6364136223846793005 + 1) & ((1 << 64) - 1)
        values.append((state >> 11) / (1 << 53))
    return [values[i:i + 2] for i in range(0, len(values), 2)]


def distances(points):
    return [math.dist(points[i], points[j]) for i in range(len(points)) for j in range(i)]


def float32_case(case):
    """Quantize the filtration once for all backends, never only Ripser's input."""
    if case.mode != "distances":
        raise ValueError("float32 parity suite requires precomputed distances")

    def quantize(value):
        rounded = struct.unpack("<f", struct.pack("<f", value))[0]
        if not math.isfinite(rounded):
            raise ValueError("float32 quantization must remain finite")
        return rounded

    return Case(case.name, case.n, [quantize(v) for v in case.values], case.q,
                None if case.cutoff is None else quantize(case.cutoff), case.mode, case.d)


def semantic_cases():
    geometries = [
        ("empty", [], None), ("singleton", [[0., 0.]], None),
        ("duplicates", [[0., 0.], [0., 0.]], 0.),
        ("pair", [[0., 0.], [2., 0.]], None),
    ]
    square = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]]
    for suffix, cutoff in [("full", None), ("isolated", .5), ("cycle", 1.), ("filled", math.sqrt(2))]:
        geometries.append((f"square_{suffix}", square, cutoff))
    cases = [Case(f"check_{name}_h{q}", len(points), distances(points), q, cutoff)
             for name, points, cutoff in geometries for q in (0, 1)]
    cases += [Case(f"check_tetrahedron_h{q}", 4, [1.] * 6, q) for q in (0, 1)]
    return cases


def performance_cases(quick):
    sizes = [16, 32] if quick else [32, 64, 128]
    n = sizes[-1]
    cases = [Case(f"uniform_h1_{size}", size, distances(uniform(size))) for size in sizes]
    circle = [[math.cos(2 * math.pi * i / n), math.sin(2 * math.pi * i / n)] for i in range(n)]
    clusters = [[x / 10 + (i % 4), y / 10] for i, (x, y) in enumerate(uniform(n))]
    duplicates = (uniform(n // 2) * 2)[:n]
    for family, points in [("circle", circle), ("clusters", clusters), ("duplicates", duplicates)]:
        cases.append(Case(f"{family}_h1_{n}", n, distances(points)))
    cases.append(Case(f"equal_h1_{n}", n, [1.] * (n * (n - 1) // 2)))
    m = sizes[-2]
    rng = random.Random(1729)
    cases.append(Case(f"nonmetric_h1_{m}", m, [rng.randrange(1, 17) / 16 for _ in range(m * (m - 1) // 2)]))
    cases.append(Case(f"uniform_h1_{n}_cutoff", n, distances(uniform(n)), cutoff=.2))
    cases.append(Case(f"uniform_h1_{n}_points", n, [v for p in uniform(n) for v in p], mode="points"))
    for size in ([64] if quick else [128, 512, 1024]):
        cases.append(Case(f"uniform_h0_{size}", size, distances(uniform(size)), q=0))
    return cases


def proc_memory():
    """Linux KiB, or null on other systems; never mislabel a Python heap peak."""
    try:
        fields = dict(line.split(":", 1) for line in Path("/proc/self/status").read_text().splitlines())
        return {key: int(fields[key].split()[0]) for key in ("VmRSS", "VmHWM")}
    except (OSError, KeyError):
        return {"VmRSS": None, "VmHWM": None}


def python_worker(backend, fixture, samples, iterations):
    import numpy as np
    if backend == RIPSER_BACKEND:
        from ripser import ripser
    else:
        import gudhi

    with open(fixture, "rb") as stream:
        magic, mode, n, d, q, cutoff = HEADER.unpack(stream.read(HEADER.size))
        values = np.fromfile(stream, dtype="<f8")
    if magic != b"COCYCLE1" or mode not in (0, 1):
        raise ValueError("invalid benchmark fixture")
    threshold = math.inf if math.isnan(cutoff) else cutoff
    if mode:
        data = values.reshape(n, d)
        diameter = max(distances(data), default=0.)
        kwargs = {"points": data}
    else:
        if len(values) != n * (n - 1) // 2:
            raise ValueError("invalid condensed input size")
        diameter = float(values.max()) if len(values) else 0.
        # Prepare GUDHI's contiguous float64 input outside the clock, avoiding
        # repeated Python ragged-list conversion in its C++ binding.
        data = np.zeros((n, n), dtype=np.float64)
        for i in range(n):
            data[i, :i] = values[i * (i - 1) // 2:i * (i + 1) // 2]
            data[:i, i] = data[i, :i]
        del values
        kwargs = {"distance_matrix": data}
    complete = threshold >= diameter

    def compute():
        if backend == RIPSER_BACKEND:
            if mode:
                raise ValueError("Ripser parity requires precomputed, float32-exact distances")
            result = ripser(data, distance_matrix=True, maxdim=q, thresh=threshold,
                            coeff=2, do_cocycles=False, n_perm=None)
            return result["dgms"], None
        rips = gudhi.RipsComplex(**kwargs, max_edge_length=threshold)
        if backend == "gudhi_edge_collapse":
            tree = rips.create_simplex_tree(max_dimension=1)
            tree.collapse_edges(nb_iterations=1)
            tree.expansion(q + 1)
        else:
            tree = rips.create_simplex_tree(max_dimension=q + 1)
        # With no triangles (or just isolated vertices), False would omit the
        # top-dimensional H1 (or H0) that we explicitly requested.
        tree.compute_persistence(homology_coeff_field=2, min_persistence=0.,
                                 persistence_dim_max=q >= tree.dimension())
        return [tree.persistence_intervals_in_dimension(k) for k in range(q + 1)], tree.num_simplices()

    before = proc_memory()
    bars, simplex_count = compute()  # One warmup, also used for validation.
    if backend == RIPSER_BACKEND and n == 0:
        # The dense API infers n=1 from an empty condensed buffer in 0.6.14.
        # Expose the actual output and exclusion; do not fabricate an empty diagram.
        return {"status": "unsupported", "reason": "Ripser.py dense API does not distinguish zero vertices from one",
                "observed_intervals": [[k, float(b), "E" if math.isinf(e) else "F",
                                         0. if math.isinf(e) else float(e)]
                                        for k, pairs in enumerate(bars) for b, e in pairs]}
    elapsed = []
    for _ in range(samples):
        start = time.perf_counter_ns()
        for _ in range(iterations):
            result = compute()
            del result  # Include output destruction, like the Rust worker.
        elapsed.append((time.perf_counter_ns() - start) / 1e6 / iterations)
    peak = proc_memory()["VmHWM"]
    intervals = []
    for k, pairs in enumerate(bars):
        for birth, death in pairs:
            if death <= birth:
                continue
            kind, end = ("F", float(death)) if math.isfinite(death) else (("E", 0.) if complete else ("C", cutoff))
            intervals.append([k, float(birth), kind, end])
    return {"samples_ms": elapsed, "rss_before_kib": before["VmRSS"],
            "hwm_before_kib": before["VmHWM"], "peak_rss_kib": peak,
            "coverage": ["complete", None] if complete else ["through", cutoff],
            "intervals": sorted(intervals), "simplices": simplex_count}


def compare(left, right, mode):
    """Exact f64 edge values for matrix input; a small tolerance for independent norms."""
    if left["coverage"] != right["coverage"]:
        raise AssertionError("coverage differs")
    a, b = sorted(left["intervals"]), sorted(right["intervals"])
    if len(a) != len(b):
        raise AssertionError(f"interval count differs: {len(a)} != {len(b)}")
    for x, y in zip(a, b):
        if x[0] != y[0] or x[2] != y[2]:
            raise AssertionError(f"dimension/end kind differs: {x} != {y}")
        for i in (1, 3):
            equal = x[i] == y[i] if mode == "distances" else math.isclose(x[i], y[i], rel_tol=1e-12, abs_tol=1e-14)
            if not equal:
                raise AssertionError(f"endpoint differs: {x} != {y}")


def build_driver(temp):
    target = temp / "target"
    executable = temp / ("benchmark_driver.exe" if os.name == "nt" else "benchmark_driver")
    subprocess.run(["cargo", "build", "--release", "--locked", "--offline", "--lib",
                    "--target-dir", str(target)], cwd=REPO, check=True)
    subprocess.run(["rustc", "--edition=2024", "-C", "opt-level=3", "-D", "warnings",
                    str(REPO / "tools/benchmark_driver.rs"), "--extern",
                    f"cocycle={target / 'release/libcocycle.rlib'}", "-L",
                    f"dependency={target / 'release/deps'}", "-o", str(executable)], check=True)
    return executable


def source_hash():
    digest = hashlib.sha256()
    paths = [REPO / "Cargo.toml", REPO / "Cargo.lock", *sorted((REPO / "src").rglob("*.rs")),
             Path(__file__).resolve(), REPO / "tools/benchmark_driver.rs", REPO / "tools/requirements-gudhi.txt",
             REPO / "tools/requirements-reference.txt", REPO / "tools/requirements-benchmark.txt"]
    for path in paths:
        digest.update(str(path.relative_to(REPO)).encode() + b"\0" + path.read_bytes())
    return digest.hexdigest()


def run_worker(backend, driver, fixture, samples, iterations, timeout, env):
    if backend == "cocycle":
        command = [str(driver), str(fixture), str(samples), str(iterations)]
    else:
        command = [sys.executable, str(Path(__file__).resolve()), "--worker", backend,
                   str(fixture), str(samples), str(iterations)]
    result = subprocess.run(command, check=True, capture_output=True, text=True,
                            timeout=timeout, env=env)
    return json.loads(result.stdout)


def save_json(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def summarize(output, records):
    import csv

    rows = []
    for record in records:
        if record["kind"] != "benchmark" or record["status"] != "passed":
            continue
        native = statistics.median(record["results"]["cocycle"]["samples_ms"])
        for backend, result in record["results"].items():
            if result.get("status") == "unsupported":
                continue
            times = result["samples_ms"]
            before, peak = result["hwm_before_kib"], result["peak_rss_kib"]
            rows.append({"case": record["case"], "backend": backend,
                         "median_ms": statistics.median(times), "min_ms": min(times), "max_ms": max(times),
                         "cocycle_over_backend": native / statistics.median(times),
                         "rss_before_kib": result["rss_before_kib"], "peak_rss_kib": peak,
                         "hwm_growth_kib": None if peak is None or before is None else max(0, peak - before),
                         "simplices": result.get("simplices"), "intervals": len(result["intervals"])})
    if rows:
        with (output / "summary.csv").open("w", newline="") as stream:
            writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
            writer.writeheader()
            writer.writerows(rows)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True, help="new directory; existing results are never overwritten")
    parser.add_argument("--quick", action="store_true", help="smaller fixtures for validation, not the recorded baseline")
    parser.add_argument("--include-ripser", action="store_true",
                        help="add Ripser.py on a separate float32-quantized distance suite; omit point-cloud input")
    parser.add_argument("--samples", type=int, default=7)
    parser.add_argument("--iterations", type=int, default=1)
    parser.add_argument("--timeout", type=float, default=120., help="seconds per backend/case, including process startup")
    parser.add_argument("--cpu", type=int, help="pin controller and workers to one available Linux CPU")
    args = parser.parse_args()
    if args.samples < 1 or args.iterations < 1 or args.timeout <= 0:
        parser.error("samples, iterations and timeout must be positive")
    if args.cpu is not None:
        if not hasattr(os, "sched_getaffinity") or args.cpu not in os.sched_getaffinity(0):
            parser.error("--cpu must name an available Linux CPU")
        os.sched_setaffinity(0, {args.cpu})
    args.output.mkdir(parents=True, exist_ok=False)
    # Preserve the exact manifest even if packaging metadata changes later.
    (args.output / "manifest-at-measurement.toml").write_bytes((REPO / "Cargo.toml").read_bytes())
    fixtures = args.output / "fixtures"
    fixtures.mkdir()
    env = os.environ.copy()
    threads = {name: "1" for name in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS", "NUMEXPR_NUM_THREADS")}
    env.update(threads)
    cpu_lines = Path("/proc/cpuinfo").read_text().splitlines() if Path("/proc/cpuinfo").exists() else []
    backends = (*BACKENDS, RIPSER_BACKEND) if args.include_ripser else BACKENDS
    packages = ("gudhi", "numpy", "ripser", "scipy", "scikit-learn") if args.include_ripser else ("gudhi", "numpy")
    metadata = {"created_utc": datetime.now(timezone.utc).isoformat(), "source_sha256": source_hash(),
                "python": sys.version, "platform": platform.platform(),
                "cpu": next((line.split(":", 1)[1].strip() for line in cpu_lines if line.startswith("model name")), platform.processor()),
                "affinity": sorted(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None,
                "threads": threads, "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
                "cargo": subprocess.check_output(["cargo", "--version"], text=True).strip(),
                "rustflags": env.get("RUSTFLAGS"), "packages": {p: importlib.metadata.version(p) for p in packages},
                "gudhi_wheel": importlib.metadata.distribution("gudhi").read_text("WHEEL"),
                "ripser_wheel": importlib.metadata.distribution("ripser").read_text("WHEEL") if args.include_ripser else None,
                "ripser_python_sha256": hashlib.sha256(
                    Path(importlib.metadata.distribution("ripser").locate_file("ripser/ripser.py")).read_bytes()
                ).hexdigest() if args.include_ripser else None,
                "build": "cargo --release; driver rustc -C opt-level=3; no target-cpu override",
                "input_layout": {"cocycle": "condensed lower triangle f64", "gudhi": "contiguous square float64 ndarray"},
                "samples": args.samples, "iterations": args.iterations, "warmups": 1,
                "timeout_seconds": args.timeout, "quick": args.quick,
                "backends": backends, "order_seed": 20260917, "input_seed": 1729,
                "precision": "float32-exact distances and cutoff, stored as f64" if args.include_ripser else "float64",
                "comparison": "exact distance-diagram multisets; point norms only use tolerance",
                "omitted_input_modes": ["points"] if args.include_ripser else [],
                "comparisons_passed": 0, "comparisons_excluded": 0,
                "status": "running"}
    save_json(args.output / "environment.json", metadata)
    records = []
    order_rng = random.Random(metadata["order_seed"])
    try:
        with tempfile.TemporaryDirectory(prefix="cocycle-gudhi-") as temp:
            driver = build_driver(Path(temp))
            cases = [("semantic", case) for case in semantic_cases()]
            cases += [("benchmark", case) for case in performance_cases(args.quick)]
            if args.include_ripser:
                cases = [(kind, float32_case(case)) for kind, case in cases if case.mode == "distances"]
            for kind, case in cases:
                fixture = fixtures / f"{case.name}.bin"
                case.write(fixture)
                order = list(backends)
                order_rng.shuffle(order)
                record = {"case": case.name, "kind": kind, "mode": case.mode, "n": case.n,
                          "homology_max": case.q, "cutoff": case.cutoff,
                          "fixture_sha256": hashlib.sha256(fixture.read_bytes()).hexdigest(),
                          "order": order, "results": {}, "status": "running"}
                records.append(record)
                print(f"{kind}: {case.name}", flush=True)
                try:
                    for backend in order:
                        record["results"][backend] = run_worker(
                            backend, driver, fixture, args.samples if kind == "benchmark" else 1,
                            args.iterations if kind == "benchmark" else 1, args.timeout, env)
                    for backend in backends[1:]:
                        if record["results"][backend].get("status") == "unsupported":
                            metadata["comparisons_excluded"] += 1
                            continue
                        compare(record["results"]["cocycle"], record["results"][backend], case.mode)
                        metadata["comparisons_passed"] += 1
                    record["status"] = "passed"
                except Exception as exc:
                    record["status"] = "failed"
                    record["error"] = repr(exc)
                    if isinstance(exc, subprocess.CalledProcessError):
                        record["stderr"] = exc.stderr
                    raise
                finally:
                    save_json(args.output / "results.json", records)
            metadata["status"] = "passed_with_exclusions" if metadata["comparisons_excluded"] else "passed"
    finally:
        if metadata["status"] == "running":
            metadata["status"] = "failed"
        save_json(args.output / "environment.json", metadata)
        summarize(args.output, records)
    print(f"PASS: {len(records)} fixtures, {metadata['comparisons_passed']} diagram comparisons, "
          f"{metadata['comparisons_excluded']} explicit exclusions; {args.output}")


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--worker":
        _, _, backend, fixture, samples, iterations = sys.argv
        if backend not in (*BACKENDS[1:], RIPSER_BACKEND) or int(samples) < 1 or int(iterations) < 1:
            raise ValueError("invalid worker parameters")
        print(json.dumps(python_worker(backend, fixture, int(samples), int(iterations)), allow_nan=False))
    else:
        main()
