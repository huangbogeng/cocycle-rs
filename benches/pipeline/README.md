# Rips pipeline measurements

[Benchmarks](../README.md) / Native pipeline

This suite complements the existing [H0/H1 native protocol](../protocol.md) and
[construction correctness workers](../../tools/README.md#native-rips-correctness-checks).
The shared [reporting rules](../reporting.md) govern evidence classification and
claims; use the [report template](../report-template.md) for retained experiments.
This suite records a protocol description and source fingerprint, without a
separate machine-emitted protocol version ID.

It measures the public APIs added by the complete Rips subsystem. GUDHI and
Ripser execute as native C++ processes; Python only prepares fixtures, builds,
limits processes, checks outputs and summarizes samples.

```sh
python3 tools/benchmark_rips_pipeline.py --quick --samples 1 --output target/rips-pipeline-smoke
python3 tools/benchmark_rips_pipeline.py --samples 3 --output target/rips-pipeline
```

Each output directory must be new. The pinned source setup and optional
`--boost-include` match [the native setup](../native/README.md). Linux is required
for `/proc/self/status` memory counters and per-child address-space limits.
Library correctness tests and public examples remain portable.

## Workflows and boundaries

One fresh process performs one workflow. One discarded warmup and the requested
measured samples use separate processes. Backend order is fixed; the controller
does not currently shuffle runs or set CPU affinity. The default three samples
support a descriptive resource snapshot, not a stable ranking. Measurements run
serially; do not run other compilation or benchmark workloads concurrently.
There are five phase bins:

| Bin | Rust | GUDHI C++ | Upstream Ripser C++ |
| --- | --- | --- | --- |
| Input | Borrowed input validation; upper/square fixture conversion or graph validation/adjacency | Finite-distance scan | Finite-distance scan and float32-exact check |
| Construction | Threshold or approximate graph; streamed Euclidean distances for points; metric checking when selected | Exact graph into tree, or sparse graph with original metric sampler | Dense float32 buffer conversion, or sparse adjacency |
| Expansion | Explicit frozen simplices and bidirectional incidence, when requested | Tree clique/blocker expansion; sparse graph insertion is included here | Not applicable |
| Compute | Public persistence call including owned diagram normalization and requested bases | Coefficient initialization and persistent cohomology | Implicit barcode computation and numeric pair capture |
| Export | Interval JSON payload construction | Pair extraction, sorting and interval JSON payload | Sorting and interval JSON payload |

End-to-end time includes these phases, intervening bookkeeping and destruction
of intermediates released during the workflow. It excludes text fixture I/O,
parsing, the final metrics envelope/transport and destruction of retained results.
Phase sums need not equal the whole workflow. Rust cannot expose a separate
reducer-only timing through its public API; its compute bin includes result
assembly. A GUDHI tree is not the same representation as a Rust frozen complex;
compare complete workflows before interpreting individual bins.

Matrix fixture conversion is explicitly measured, not hidden. Normal Rust users
can borrow an existing upper/square matrix without this conversion. Raw fixture
buffers remain part of process memory; they are not counted as reducer workspace.
Point workflows discard their unused precomputed values before measuring the
public point API, but parsing can already have raised the process high-water mark.

The input geometries include circle, uniform, nonmetric, cross-polytope sphere,
bipartite graphs, isolated vertices, exact point distances and dyadic metric
approximation. Cutoffs, dimensions, fields and representative requests are retained
in each result. Quantization to float32 occurs once when constructing shared
exact fixtures; all three workers receive those exact numbers. Sparse approximation
uses exact dyadic Manhattan metrics without additional rounding.

## Native comparability

GUDHI's sparse constructor uses its original metric sampler, with only the initial
vertex fixed to zero. A read-only accessor exposes the resulting permutation.
Fixtures have unique greedy choices, which the controller independently checks;
it rejects unequal sampling or diagrams. This performance adapter does **not**
use the exhaustive reference sampler from the construction correctness suite.
That separate suite still covers ties, duplicate points and full simplex sets.

Ripser has no sparse approximation constructor with higher-simplex blockers, so
those workers are explicitly excluded. Coefficient limits also produce explicit
exclusions. Native representative computation is not provided by these adapters;
GUDHI/Ripser runs for a representative case check diagram correctness only and
must not be used to claim a comparable basis-computation speedup. The same applies
to native precomputed-distance references for Rust point workflows, native lower
matrices compared with Rust upper/square conversion, Ripser diagram-only runs
compared with requested explicit complexes, and GUDHI comparisons against Rust's
exhaustive metric-checking workflow. Representative payloads are retained and
counted but only intervals are serialized in the measured export phase.

Full interval multisets are compared on every warmup and measured run, preserving
multiplicities and unpaired endpoints. Rust coverage is retained; reference infinity
is interpreted using the fixture's range, never as an unconditional essential class.
Approximate permutations and explicit simplex counts are additionally checked.
This timing harness does not replace independent mathematical or native topology
correctness suites.

## Memory, failures and artifacts

Each worker reads its own `VmRSS` and `VmHWM` before the measured workflow and its
`VmHWM` after computation/export. Peak RSS includes runtime, parser/input storage,
allocator retention, temporaries and outputs. The difference between high-water
marks is **not** allocated bytes or an algorithm-only peak. Do not use the Python
controller's RSS or cumulative child high-water mark as an individual sample.

`--address-space-mib` is a process virtual-address-space cap, not a library RSS
budget. `--timeout` kills that sample process. Failures and timeouts remain in
`results.json`, fail the suite and cannot be silently converted to exclusions.
The library's cooperative work limits are tested separately and do not claim
hard memory or wall-time enforcement.

`environment.json` records source/pin/header/binary hashes, compiler commands,
CPU/platform details and limits. `fixtures/` retains exact inputs, `results.json`
all samples and validation outcomes, `measurements.json` phase medians, ranges,
peak RSS and comparison scopes, and `summary.json` the overall outcome.
`build/build.log` preserves compiler diagnostics. A comparison mismatch makes
all measurements of that case unvalidated; retained timings are not successful
performance evidence. Source changes during a run also invalidate that run.

Generated outputs belong under ignored `target/` or in external artifact storage.
They must not be committed as loose files or archives; see the
[storage policy](../reporting.md#storage-and-evidence-lifecycle).
