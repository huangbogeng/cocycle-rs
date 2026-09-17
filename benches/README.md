# Benchmarks

Benchmarks are developer tools, not runtime dependencies or CI speed gates.
Start with the [same-size three-library comparison](scaling-threeway.md) for the
largest completed study. Reports describe their recorded source hashes; subsequent
refactors do not turn those timings into fresh measurements.

## Reports and retained evidence

| Report | Scope |
| --- | --- |
| [Same-size comparison](scaling-threeway.md) | All 23 scaling fixtures scheduled for all four paths, including GUDHI at 512/1024 vertices |
| [Scaling and difficult inputs](scaling.md) | Original bounded run, circle supplement, and algorithm work counters |
| [Implementation history](history.md) | Explicit reference baseline, implicit H1 transition, and ablation study |
| [Project cleanup verification](project-cleanup.md) | Checks after code/documentation reorganization; not a new timing claim |

Raw results remain under `results/`, with environment, fixture hashes, diagrams,
samples, statuses, and CSV summaries. The original dependency-free Rust baseline
is [linux-x86_64.csv](linux-x86_64.csv). Retained reports and raw records must agree;
do not edit old outputs when changing source or tools.

## Compared paths

| Backend | Path |
| --- | --- |
| Cocycle | Release Rust public API: H0 union-find, H1 implicit cohomology |
| GUDHI direct | `RipsComplex` → `create_simplex_tree(q+1)` → persistence |
| GUDHI collapse | Graph → one `collapse_edges` iteration → `expansion(q+1)` → persistence |
| Ripser.py | Python `ripser` API with precomputed distances |

Historical reports explicitly label runs made before implicit H1. GUDHI uses
F2, ordinary homology, closed edge thresholds, and zero-length interval removal.
The worker enables top-dimensional persistence when required, including a
triangle-free H1 cutoff. Both GUDHI paths materialize a simplex tree; other GUDHI
interfaces are outside this comparison. Ripser.py timing is not upstream C++ CLI
timing. No path uses sampling or sparse-Rips approximation.

## Input and correctness protocol

Generate each fixture once, then send the same file to every backend. Record
versions, dimensions, scale conventions, cutoff, and precision. Condensed
Cocycle distances and GUDHI's contiguous square input contain the same values;
prepare their layouts outside the timed region.

GUDHI-only runs retain f64 inputs, including direct point-cloud cases. Three-library
runs use float32-exact distances and cutoffs because the pinned Ripser.py dense
path converts to float32. Values are stored losslessly as f64 for Cocycle/GUDHI;
their kernels remain f64. Do not mix the two experiments or inflate tolerances to
hide different filtrations. Zero-point Ripser exclusions remain explicit.

Compare diagram multisets, not library-specific ordering or representatives.
Interpret infinite endpoints according to complete/censored coverage. Record
failures and skipped cases; never substitute an empty diagram or fabricate timing.

## Timing and memory

Each backend gets a fresh process, one warmup, then the report's stated number of
measured calls. Public API construction, reduction, and output destruction are
included; imports, fixture parsing, and external input-layout preparation are not
inside the timed call. Point-cloud API timing includes distance construction.
Python wrapper overhead is included. Run backends serially; pin CPU affinity and
thread counts for controlled local comparisons.

Whole-worker time limits include startup, imports, warmup, every sample, and
serialization. A timeout is not a lower bound on one call's runtime. `RLIMIT_AS`
limits virtual address space, not RSS. The core library has no such automatic cap.

Linux memory records include prepared-input RSS, previous process high-water mark,
and final peak RSS. Peak growth is not an exact live-allocation measurement.
Python runtime baselines differ from Rust; do not rank algorithm memory by raw
cross-language process RSS ratios. Work counters come from instrumented test
builds and must not be mixed with production timings.

## Running and retaining experiments

Run `cargo bench --locked --bench rips` for the dependency-free benchmark.
Installation and commands for cross-library runs and profiling are centralized in
[tools/README.md](../tools/README.md).

Use `target/` for exploratory output; never overwrite a prior run. To retain a
reviewed experiment, copy its complete directory under `benches/results/`, add a
dated English report, and link it here. Record unfavorable results, resource
limits, and exact source/fixture hashes alongside successful comparisons.

Large raw records are repository/research artifacts and are excluded from the
crate package. The configured manual benchmark workflow uploads CI artifacts;
shared-runner timings are not directly comparable with pinned local measurements.
