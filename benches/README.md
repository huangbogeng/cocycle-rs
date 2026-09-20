# Benchmarks

Current cross-library measurements execute Cocycle's Rust API, GUDHI's C++ API,
and upstream Ripser C++. Python's standard library prepares fixtures, starts
executables and validates outputs; no Python TDA binding runs in a measured worker.

Start with the [reporting rules](reporting.md) to define a comparable experiment,
then select its execution protocol below. Use the [report template](report-template.md)
for retained conclusions. The two native timing suites have different preparation,
warmup, export and memory boundaries; their samples must not be pooled.

## Choose a suite

| Question | Tool and contract | Evidence |
| --- | --- | --- |
| How does precomputed F2 H0/H1 compare, including GUDHI edge collapse? | [Native setup](native/README.md), [H0/H1 protocol](protocol.md); `tools/benchmark_native.py` | Time to owned intervals and process memory |
| What do complete public Rips workflows cost? | [Pipeline protocol](pipeline/README.md); `tools/benchmark_rips_pipeline.py` | Validation/construction/expansion/compute/export, end-to-end time and process memory |
| Do exact construction, fields and higher dimensions agree? | [Exact correctness](../tools/README.md#native-rips-correctness-checks); `tools/compare_rips.py` | Topology and persistence validation, not performance |
| Do sparse sampling, blockers and persistence agree? | [Sparse correctness](../tools/README.md#native-sparse-rips-checks); `tools/compare_sparse_rips.py` | Approximation validation, not performance |
| What does the Rust API cost without external comparisons? | [rips.rs](rips.rs), `cargo bench --locked --bench rips` | Rust-only timings, not a cross-library baseline |

The pipeline covers matrix/graph/point construction, explicit complexes, prime
fields, representatives and approximation. Its current default of three samples
is a resource snapshot. Native references doing less work are correctness-only;
Ripser approximation is explicitly unsupported. See the protocol for row-level
comparison scopes and the reporting rules for stronger comparative studies.

## Layout

| Location | Responsibility |
| --- | --- |
| [reporting.md](reporting.md) | Shared comparability, sampling, claim and evidence rules |
| [report-template.md](report-template.md) | Reusable PR/commit-bound experiment report structure |
| [native/](native/README.md) | H0/H1 workers, shared upstream source pins and setup |
| [protocol.md](protocol.md) | `cocycle-native-v1` H0/H1 execution contract |
| [pipeline/](pipeline/README.md) | Complete-workflow workers and their execution contract |
| [reports/](reports/README.md) | Commit-bound reports, native drafts and source-indexed historical archives |
| `results/` | Immutable retained fixtures, raw samples, environments and summaries |

Fixture and build helpers are shared where their contracts agree; correctness
instrumentation and timing adapters remain separate. In particular, the sparse
correctness sampler is not the sampler used by the timed GUDHI workflow.

## Compared implementations

| Path | Native execution |
| --- | --- |
| Cocycle | Rust public APIs, specialized F2 H1 or generic prime-field computation; requested construction/bases depend on the workflow |
| GUDHI direct | C++ Rips/flag construction, `Simplex_tree` expansion and CAM persistence |
| GUDHI collapse | One native flag edge-collapse call before expansion/CAM; available in the H0/H1 suite |
| GUDHI sparse approximation | Original metric greedy sampler with documented start/accessor instrumentation, sparse construction, blocker expansion and CAM; pipeline only |
| Upstream Ripser | C++ implicit exact dense or sparse-threshold persistence; no equivalent approximate constructor |

Backend IDs are suite-specific and retained in the raw results. GUDHI's integrated
Ripser is not the upstream Ripser backend. The adapters do not cover every engine
or feature of either library. Native execution removes Python wrapper overhead,
but does not remove required native conversions, allocations or copies.

## Evidence lifecycle

Use a new `target/` directory for each exploratory run. Review correctness,
comparison scope, failures and the full sample matrix before retaining a uniquely
named run under `results/` and adding a PR/commit-bound report. Follow the
[retention rules](reporting.md#retain-and-review-evidence) for measured source,
fixtures, raw outputs, environments and immutable history.

Benchmarks are not CI speed gates. Smoke checks validate the harness; shared-runner
timings do not establish a local performance baseline. Raw experiments, native
sources and binaries are outside the Rust crate payload.

The [Rips acceptance report](reports/draft-9e6000c557ca-rips-pipeline.md) records the
working-tree resource snapshot `9e6000c557ca` and independent correctness evidence.
It remains a draft without a verified measured-commit binding. The
[native validation draft](reports/draft-e1493a10ae91-native-h0h1.md) retains the
first H0/H1 C++ baseline and small scaling checks. Earlier GUDHI/Ripser.py studies
remain [historical wrapper evidence](reports/README.md#historical-python-wrapper-evidence),
with original source identities and measurement meanings.
