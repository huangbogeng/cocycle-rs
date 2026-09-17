# Developer tools

These optional tools are separate from the Rust library and are not shipped in
the crate package. For contribution rules see [CONTRIBUTING.md](../CONTRIBUTING.md);
for measurement conventions and retained reports see [benchmarks](../benches/README.md).
Run `python3 tools/check_docs.py` to check local documentation links and language.
It needs only the standard library.

## Independent reference comparison

These are development tools, not runtime dependencies or public library APIs.
`compare_ripser.py` compares Cocycle with Ripser.py on the same distance matrices.
It compiles the crate and a small Rust adapter in a temporary directory.

From the repository root, with `uv`, Rust and Python 3.12 available:

```sh
uv venv --python 3.12 /tmp/cocycle-reference-env
uv pip install --python /tmp/cocycle-reference-env/bin/python -r tools/requirements-reference.txt
/tmp/cocycle-reference-env/bin/python tools/compare_ripser.py
```

The pinned environment was verified on Linux x86_64; adapt interpreter paths for
other platforms. Python is not needed for `cargo test`.

The comparison uses seed 1729, 64 inputs with 1–8 vertices, dimensions 0 and 1,
and four cutoff settings, for 512 diagram comparisons. Values are quarter-integers
so both Ripser's float32 and Cocycle's float64 represent the input exactly.
Zero-length intervals are omitted on both sides. Infinite endpoints from Ripser
are classified as essential or censored using the input diameter and cutoff.
The coefficient field is F2, homology is ordinary, and scales are edge lengths.

`tests/rips.rs` separately checks hand examples, independent dense F2 ranks and
small-diagram perturbation bounds. Agreement with Ripser supplements those checks.

## GUDHI benchmark

The optional benchmark compares Cocycle with GUDHI's explicit SimplexTree and
one-pass edge-collapse paths. Install its separate pinned environment:

```sh
uv venv --python 3.12 /tmp/cocycle-gudhi-env
uv pip install --python /tmp/cocycle-gudhi-env/bin/python -r tools/requirements-gudhi.txt
/tmp/cocycle-gudhi-env/bin/python -m unittest discover -s tools -p 'test_*.py'
/tmp/cocycle-gudhi-env/bin/python tools/benchmark_gudhi.py --quick --samples 2 --output /tmp/cocycle-gudhi-quick
/tmp/cocycle-gudhi-env/bin/python tools/benchmark_gudhi.py --output /tmp/cocycle-gudhi-full
```

Output directories must be new. Optionally pass `--cpu N` on Linux to pin all
workers to an available CPU; use `os.sched_getaffinity(0)` to find allowed IDs.
`--samples`, `--iterations` and `--timeout` control the experiment, without changing
library parameters. Compilation uses Cargo's release profile and `rustc -O` for
the private adapter. No benchmark dependency is added to the Rust crate.

See [benchmark protocol and records](../benches/README.md) for input families,
exact-vs-tolerant comparisons, measured boundaries, memory caveats and artifacts.
The default run has 18 semantic and 13 performance fixtures, compared with both
GUDHI paths. Quick mode uses smaller fixtures. All timings use worker-internal
clocks, so startup and compilation are excluded. Python API overhead remains
included for GUDHI. A mismatch, crash or timeout fails the command.

## Ripser.py in the three-library benchmark

Use the combined pinned environment (Ripser.py **0.6.14**, GUDHI **3.13.0**):

```sh
uv venv --python 3.12 /tmp/cocycle-benchmark-env
uv pip install --python /tmp/cocycle-benchmark-env/bin/python -r tools/requirements-benchmark.txt
/tmp/cocycle-benchmark-env/bin/python tools/benchmark_gudhi.py --include-ripser --quick --samples 2 --output /tmp/cocycle-ripser-quick
/tmp/cocycle-benchmark-env/bin/python tools/benchmark_gudhi.py --include-ripser --output /tmp/cocycle-ripser-full
```

The historical script name remains valid. Without `--include-ripser`, it still
runs the original float64 GUDHI suite. With the flag, **every backend receives
the same float32-quantized distances and cutoff**, stored losslessly as float64
fixtures. This is a separate experiment, not a change to Cocycle's f64 API.
Direct point-cloud cases are omitted because independent distance construction
would not provide the same quantized filtration. Diagram comparisons remain exact.

The default Ripser suite has 18 semantic and 12 performance fixtures, four paths
across three libraries, and 88 successful diagram comparisons. Two empty-input
comparisons are explicitly excluded: the tested Ripser.py dense interface infers
one vertex from an empty condensed array. Its observed output is retained under
`status: unsupported`; the overall run is `passed_with_exclusions`. No fake empty
Ripser diagram or timing is substituted. Other discrepancies still fail the run.

This measures the released Python binding's public API, including its conversion
and wrapper costs, not the upstream C++ CLI in isolation. Sampling and cocycle
extraction are disabled (`n_perm=None`, `do_cocycles=False`), with F2 and the same
closed threshold. Imports and data preparation remain outside timed sections.

## Private H1 optimization profiling

`profile_rips.py` builds release library tests and runs six cumulative stages:
explicit coboundary reduction, clearing, implicit reconstruction, cone stopping,
apparent shortcuts, and initial-column emergent shortcuts. Every stage is checked
against the independent explicit boundary reducer after timing and RSS sampling.

```sh
python3 tools/profile_rips.py \
  benches/results/ripser-2026-09-17/fixtures/uniform_h1_128.bin \
  benches/results/ripser-2026-09-17/fixtures/circle_h1_128.bin \
  benches/results/ripser-2026-09-17/fixtures/equal_h1_128.bin \
  --cpu 0 --output /tmp/cocycle-ablation.json
```

Use a new output path. Each fixture/stage gets a fresh process, one warmup and
five timed calls. The JSON retains source/fixture hashes, compiler, samples and
Linux process HWM. `cofacets` counts accepted cofacets yielded during both shortcut
search and reconstruction (including repeated generation); `column_additions`
counts pivot cancellations. `stored_entries` counts off-diagonal entries of V in
implicit mode, or entries of R in explicit mode; these have different meanings
and element sizes. `peak_heap_entries` counts the working coboundary heap's
entries including uncancelled duplicates, not the transformation heap or bytes.

These are diagnostic, instrumented test builds. Production has no counters or
optimization switches in its public API. Use the cross-library suite for actual
public-entry timings. Ordinary `cargo test` skips the profiling test and has no
timing assertions; it does run the seven-mode correctness comparisons.

## Bounded scaling and difficult inputs

`benchmark_scaling.py` is a separate Linux experiment. It reuses the original
public-API workers and exact diagram comparison, without changing the historical
baseline suite. The default set contains 23 H1 cases: uniform 2D and circle
inputs at 128/256/512/1024 vertices, normalized 8D cube samples and 64-level
nonmetric matrices at 128/256/512, square grids at 64/144/256, and complete
bipartite filtrations at 64/128/256 with both full range and cutoff 1. Normalized
cube samples lie on S^7 but are not uniformly sampled on the sphere.

```sh
/tmp/cocycle-benchmark-env/bin/python tools/benchmark_scaling.py \
  --cpu 0 --samples 3 --timeout 60 --address-space-mib 2048 \
  --output /tmp/cocycle-scaling
```

All backends receive the same float32-exact distances. Each backend/case runs in
a fresh process, with one warmup and three measured calls. The 60-second limit
covers startup, imports, warmup, all calls and output serialization; it is NOT a
single-call timeout. RLIMIT_AS limits virtual address space, NOT RSS, and includes
the language runtime and imports. Input preparation remains outside timed calls.

GUDHI's two explicit paths are scheduled only for n <= 256 by default. Larger
cases contain a `not_scheduled` record stating that predeclared policy; no timing
or speed ratio is invented. `--gudhi-max-n`, `--families`, `--quick`, sample count
and resource limits are explicit experiment parameters recorded with the output.
Keep outputs from different protocols separate. Core computations have no such
automatic resource limits; these are benchmark subprocess controls only.

To schedule all four paths for the entire fixture set, including GUDHI at
512/1024 vertices, use an explicit larger experiment budget:

```sh
python tools/benchmark_scaling.py --cpu 0 --samples 3 --timeout 180 \
  --address-space-mib 16384 --gudhi-max-n 1024 \
  --output /tmp/cocycle-scaling-threeway
```

This command can still time out on difficult cases; it is not a completion
guarantee. Its independent results are documented in
[the full-schedule comparison](../benches/scaling-threeway.md).

The controller continues after worker timeout/error and saves all statuses.
Mismatched diagrams, process errors or malformed output cause a nonzero exit;
timeouts produce `completed_with_limits`. An unverified survivor is not ranked.
When Cocycle fails but external workers complete, their diagrams are still
compared. Complete bipartite cases additionally use the analytic multiset in
mathematics.md section 10. Validation compares endpoint multiplicities, not only
Betti numbers. Raw timings of unverified results remain in JSON, not summary
rankings. `summary.csv` retains failures and planned omissions as explicit rows.

Inspect algorithm work separately, after the benchmark completes:

```sh
/tmp/cocycle-benchmark-env/bin/python tools/profile_scaling.py \
  --benchmark /tmp/cocycle-scaling \
  --cases uniform_h1_512 sphere8_h1_512 nonmetric_h1_512 bipartite_h1_256_cutoff \
  --output /tmp/cocycle-scaling-work.json
```

This compiles private release-test counters and runs one computation per selected
fixture, using the benchmark's CPU and address-space settings. It requires the
same source hash and an independently validated Cocycle result, then compares
the entire instrumented diagram before accepting counters. Large cases do not
invoke the old cubic boundary reference. `cofacets` counts yielded, accepted
cofacets including repeated generation, not rejected candidate vertices;
`stored_transform_entries` and `largest_transform` exclude implicit diagonal
entries. Heap counters include duplicates awaiting F2 cancellation and are
counts of entries, not bytes. Counters are absent from production builds.
