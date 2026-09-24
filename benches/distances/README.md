# Native diagram-distance experiments

[Benchmarks](../README.md) / [Reporting rules](../reporting.md)

Protocol: `cocycle-distance-v1`. This suite is independent of both VR protocols.
Workers execute real Rust code and pinned Topp C++20. Python's standard library
controls processes; an isolated GUDHI/POT Python worker provides correctness
evidence and never qualifies as a native timing reference.

## Setup and commands

Requirements are Rust 1.91+, Python 3.10+, a GCC-compatible C++20 compiler and Git.
Linux is required for formal resource measurements; Windows supports correctness.
The library gains no dependencies. [sources.json](sources.json) pins Topp and the
exact GUDHI/POT/NumPy versions. Use a dedicated clean external source checkout:

```sh
git clone https://github.com/proffitteoy/Topp.git target/native-sources/topp
git -C target/native-sources/topp checkout --detach ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234
python3 -m venv target/distance-oracle-venv
target/distance-oracle-venv/bin/python -m pip install gudhi==3.11.0 numpy==2.4.6 POT==0.9.6.post1
python3 tools/compare_distances.py --quick --gudhi-python target/distance-oracle-venv/bin/python --output target/distance-correctness-001
python3 tools/compare_distances.py --suite stress --gudhi-python target/distance-oracle-venv/bin/python --output target/distance-stress-001
python3 tools/benchmark_distances.py --quick --samples 1 --exploratory --groups baseline arena --families uniform sparse --gudhi-python target/distance-oracle-venv/bin/python --output target/distance-smoke-001
```

Every output directory must be new. `--topp-source`, `--cargo`, `--rustc`, `--cxx`
and `--gudhi-python` accept explicit paths. The builder rejects wrong/dirty Topp
sources and never installs packages, resets, or modifies an external checkout.
It fingerprints source, native binaries, commands and toolchains. Topp uses its
portable scalar build: MSVC-only AVX2 dispatch is disabled. Weighted Topp matching
uses compiler-dependent `long double`; Rust uses f64. Preserve these representation
differences when interpreting results. This builder does not claim MSVC support.
The correctness `default` variant keeps Topp's unrestricted adaptive defaults.
Its `dense_parallel` candidate route can spawn threads at 262144 pairs. Measured
C++ variants instead use a serial configuration: the same pinned four-row density
test replaces only that route with `dense_blocked`. Prepared diagrams and the
extra routing check are counted inside the clock; inputs are prepared once.
Original Topp source remains unchanged. Records identify `cpp_threads: 1`, whether
the serial override applied, and a post-call observed thread count. The latter
alone is not a peak-thread measurement or the reason to assert serial behavior.

## Inputs, outputs and independent correctness

All workers read the same binary fixture: `COCDST1\0` (eight bytes), little-endian
u64 left/right counts, then left and right interleaved little-endian f64 endpoint
pairs. Exact length is checked; no f32 quantization occurs. Original bits, order
and multiplicity remain in the fixture.

Correctness compares one complete computed dimension with finite births and
ordered finite deaths or positive infinity. The Rust public adapter explicitly
removes finite diagonal points when making typed intervals to match Topp's raw
convention; this is not acceptance of diagonal points by `PersistenceInterval`.
The library tests separately exercise censored coverage, uncomputed dimensions,
invalid endpoints and provenance rejection. Performance fixtures are strictly
finite and off-diagonal.

Each process receives `fixture metric variant`. Metrics are `bottleneck`
(L-infinity), `w1` (order 1, L-infinity), and `w2` (order 2, Euclidean, final square
root included). JSON identifies protocol/backend/metric/variant and retains the
scalar, time, process memory and counters. Infinity is `value_kind: "infinite"`
with `value: null`. NaN, negative results, crashes and malformed output are errors.

The tiny independent oracle enumerates every partial injection, including
unmatched points' diagonal costs, using exact rational arithmetic before the W2
square root. It shares no production search, graph or matcher code. Hand-derived,
empty, repeated, unequal-size, near-diagonal, negative-scale, tied and essential
examples supplement deterministic random inputs. Every supported backend is
checked against independent expectations, not merely Rust/Topp mutual agreement.

GUDHI uses bottleneck `e=0`, or explicit Wasserstein order/internal norm with
`keep_essential_parts=True`, no autodiff and POT's exact transport solver. The
isolated worker uses NumPy and disables optional POT GPU/autodiff imports.
Versions/settings are retained; a missing reference makes validation fail.
Small dyadic bottleneck/W1 cases use zero tolerance. Other cases use the fixed
bound `64 * f64_epsilon * (n+m+1) * max(cost_scale, |expected|)`, with scale from
endpoint differences and diagonal costs, not the absolute coordinate origin.
Never increase tolerances after observing a mismatch.

`--suite supported` is the default. `--suite stress` checks very large/small and
adjacent-float cases; `--suite all` includes both. Pinned Topp's rounded-midpoint
diagonal projection disagrees with `(death-birth)/2` at adjacent floats. Stress
retains the real disagreement and returns nonzero, rather than labeling it as
agreement or broadening tolerance. Rust correctness and reference disagreement
remain separate evidence.

## Timing, sampling and ablations

One fresh native process computes one pair. Parsing and initial raw-pair layout
finish before timing. Validation, diagonal projection, preparation, solve and
temporary cleanup are counted; startup, fixture I/O and JSON formatting/transport
are excluded. All Rust native variants share the same finite validation/copying
boundary; the separate `public` adapter is correctness-only. Statistics counters
run inside the clock for both languages: these are instrumented time-to-result
measurements, not uninstrumented kernels. Raw inputs survive computation; solver
temporaries are destroyed before the clock stops. Copies/parser high-water marks
remain part of process memory.

Retain/discard one fresh warmup per cell; formal comparisons use at least twelve
measured processes. Sample counts round upward to a multiple of active variants.
Seeded shuffled cyclic blocks balance execution positions. Workers run serially,
single-threaded, without concurrent compilation or testing. Optional `--cpu` pins
to an allowed Linux CPU. Selected/inherited affinity and uncontrolled frequency
and host load are recorded.

Development seed is `20260922`, holdout seed is `20260923`. Families include
uniform, clustered, near-diagonal, duplicates, imbalance, separated, threshold
shell and sparse/dense adversarial cases. Default sizes are 8/32/128/512; request
2048/4096 explicitly where limits permit. `--families`, `--sizes`, `--metrics` and
`--groups` preregister a smaller study; its conclusions remain scoped to that set.

| Group | Controlled contrast |
| --- | --- |
| `baseline` | Topp serial adaptive configuration and Rust adaptation, always retained; unrestricted default remains a correctness reference |
| `search` | Forced quickselect versus binary with other settings held fixed within each language; adaptive baseline remains a separate row |
| `scratch` | Rust forced refinement with/without scratch reuse |
| `matching` | Rust forced refinement with/without matching reuse |
| `clipping` | Forced quickselect with/without candidate clipping in both languages |
| `arena` | Forced sparse vectors/arena in both languages, plus Rust `adaptive_arena` under its ordinary default routing |

Topp has no corresponding exposed scratch/matching-reuse controls or adaptive
arena layout switch; these missing counterparts are limitations, never invented
equivalent experiments. Local sparse variants disable duplicate/component/greedy
shortcuts and keep those settings fixed within their pair. They cannot select an
adaptive default alone: `adaptive_arena` tests the actual candidate integration.
Route/counter records show whether the intended path ran. Rust
`direct_cost_fallbacks > 0` or `sparse_solves == 0` does not test arena layout.
Per-language retained/disabled ratios answer whether an optimization survives
migration; cross-language absolute times answer a different question.

## Memory, selection and artifacts

Workers read pre-call `VmRSS`/`VmHWM` and final `VmHWM` after cleanup, before JSON.
Report absolute peak and high-water growth separately; zero growth does not mean
no allocations. RSS includes runtime, input, allocator retention and outputs.
Explicit container capacities are not allocator peaks. `--address-space-mib`
(default 2048 MiB) caps virtual address space, and `--timeout` (default 60 seconds)
limits whole-process wall time. Neither is a library budget; timeout is censored
evidence, not a measured runtime equal to the limit.

`results.json` retains every sample, warmup, order, exit and mismatch. After a
worker fails, its remaining attempts are `not_run` while others continue. No
mismatched/incomplete case is ranked. Valid summaries include median/min/max and
RSS. Changed sources invalidate a run. `--quick` and `--exploratory` are resource
snapshots and never select algorithms; formal measurements reject dirty sources.

Selection compares candidate/R0 time and peak-RSS ratios, with equal size weight
within each family and equal family weight. Holdout composite
`sqrt(time_ratio * peak_rss_ratio)` must be at most 0.95; neither metric may exceed
1.10 in any default-applicable family. A first qualifying result is only
`eligible_pending_independent_repeat`: repeat the same gates in a second fresh
run on identical sources/fixtures before deciding. Noise and fixed process
overhead are inconclusive; retain the simpler safe baseline on ties. Forced
local ablations are not automatically global default candidates.

Commit implementation/harness before formal measurements. Use fresh
`target/benchmarks/commit-<sha12>/distance/run-<NNN>/` artifacts; preserve raw
fixtures, JSON, logs and metadata outside Git. A later concise report identifies
the measured commit separately from the report commit. No external storage or
publicly accessible evidence is implied by a local artifact path.

```sh
python3 -m unittest discover -s tools -p 'test_*distances.py'
rustfmt --edition 2024 --check benches/distances/cocycle.rs
```
