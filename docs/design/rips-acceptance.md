# Rips subsystem acceptance

[Documentation](../README.md) / Design

This audit links the implemented R1-R10 contracts to executable evidence. Local
Linux validation and configured hosted CI are distinct. A workflow definition is
not evidence that the current source passed on Windows or macOS. Publishing and
release-commit approval remain separate from this implementation audit.

## Capability and evidence matrix

| Target | Implemented contract | Evidence | Practical boundary |
| --- | --- | --- | --- |
| R1: matrix layouts | Borrowed lower/upper/square views, finite nonnegative symmetric data, zero diagonal | `tests/matrix.rs`, native exact fixtures | Matrix validation is O(n²); no triangle inequality is inferred |
| R2: points and callbacks | Euclidean pair evaluation and declared symmetric callbacks | `tests/euclidean.rs`, `tests/rips_construction.rs`, `tests/sparse_rips.rs` | Custom sparse callbacks cache all unordered pairs; finite-cutoff points stream distances |
| R3: graph paths | Exact Rips coverage distinct from supplied flag absence | `tests/flag.rs`, `tests/graph.rs`, native GUDHI/Ripser comparisons | Sparse representation does not change the filtration or silently fill absent edges |
| R4: explicit topology | Frozen face-closed simplices, oriented boundaries/cofaces and original labels | `tests/rips_expansion.rs`, native simplex/value comparisons | Explicit materialization may be exponential; standalone expansion is uncontrolled |
| R5: dimensions | Dimension-generic oriented cohomology with killing cofaces and separate skeleton sufficiency | H2-H4 sphere fixtures, independent boundary oracle, native comparisons | Generic computation has input-dependent enumeration and fill-in; no promised maximum feasible dimension |
| R6: fields | Validated prime-u32 coefficients and overflow-safe arithmetic | `tests/prime_fields.rs`, flag RP2, native shared-prime fixtures | Native coefficient limits are explicit exclusions; large-u32 support has independent Rust validation |
| R7: representatives | Optional persistent cycles and query-scale dual cocycles, interval multiplicity and original IDs | Independent closure/rank/duality tests; sparse noncontiguous-label regression | Requires a materialized skeleton/transformations; no shortest-basis or native representative-vector parity claim |
| R8: approximation | Deterministic greedy order, modified edges and higher-simplex blocker | `tests/sparse_rips.rs`, 156-case native sparse suite | Conditional ideal-arithmetic theorem; explicit metric policy, subset target and floating-point limits |
| R9: results | Owned diagram, field/source/coverage context, requested bases and approximation provenance | Source-drop, mapping and repeated-call tests | Discarding context via `into_diagram` intentionally discards provenance |
| R10: resources | Per-call work/cancellation controls, checked sizes and no partial success | `tests/rips_resources.rs`, existing limit/index tests, native pipeline measurements | No hard library RSS/time cap, no universal allocation-failure recovery; constructors are outside computation budgets |

The checked condensed pair-count operation is shared by matrix validation,
Euclidean conversion and sparse callback caching. Its wide-integer regression
checks both actual overflow and representable results whose undivided product
would overflow the target `usize`.

## Resource ownership and costs

| Operation | Retained/additional storage | Controlled work |
| --- | --- | --- |
| Borrowed matrix/point validation | Views borrow the caller's buffer | Standalone validation has no execution controls |
| Threshold construction | O(n+m) graph and adjacency; every unordered distance considered | Standalone construction has no execution controls |
| Sparse approximation | O(n) sampling/provenance plus O(n+m) graph; callback cache adds O(n²) | O(n²) distance evaluations; explicit metric checking adds O(n³); no constructor budget |
| Explicit expansion | All selected simplices and oriented incidence | Standalone expansion has no budget and may be exponential |
| F2 H1 implicit path | Graph/distance access, O(n) indexing, edges, pivots and transformations | Counted scans, intersections, reductions and heap work |
| Generic prime/dimension path | Current dimension, tuple keys, pivots and transformation fill-in | Counted dimension traversal, cofaces and coefficient operations |
| Requested representatives | Skeleton through requested homology dimension + 1, boundary transformations, bases | Materialization, reduction, dual solving and output terms count toward computation limits |
| Owned output | Interval multiset, optional representative terms and O(n) approximate provenance | Assembly/sorting and some allocations are not interruptible internally |

Distance-evaluation counts are not full runtime bounds: graph ordering/validation,
lookup, coefficient arithmetic, output size and allocator behavior also contribute.
The [execution API](../../src/persistence/execution.rs) defines counted units.
Cooperative cancellation is observed at checkpoints; an allocation or sort may
finish before cancellation is returned. A cancelled call never resets the user's
flag. Recovery tests reuse the same input after failures over F2 and F3, including
all six richer input paths with and without representative requests. Concurrent
calls over different fields share only immutable input and retain separate state.

## Measurement evidence and interpretation

Historical measurement outputs and reports have been removed. Fresh evidence
belongs in CI artifacts or external storage under the
[reporting rules](../../benches/reporting.md). The
[pipeline protocol](../../benches/pipeline/README.md) specifies phase timing,
end-to-end time, raw samples, process memory and native correctness checks.
It separates public computation from construction/expansion and interval export;
Rust's compute phase includes owned result normalization. References do not
pretend to provide representative extraction or exhaustive metric certification.
Those cases have native diagram correctness checks but no equal-work timing ratio.

The finite fixture matrix is a resource snapshot, not a worst-case memory proof
or a promise that Rust outperforms either reference. Unexpected timeouts,
process errors, diagram mismatches and unsupported capabilities have separate
statuses. Changing precision, field, filtration or sampling to obtain a favorable
number is not permitted. Measurement reports preserve unfavorable observations.

## Verification and release boundary

Local required checks include debug/release tests, Rust 1.91, Clippy, strict
rustdoc, Markdown examples, Python protocol tests, source checks, package contents
and packaged examples. Native correctness suites compare exact Rips/flags and
sparse approximation independently of the timed workers. CI runs the portable
library suite on Linux, macOS and Windows, and the resource smoke on Linux.

Local results cannot certify a source revision on hosted operating systems. The release gate still requires a committed revision, hosted CI for that
revision, review of the package and the [release procedure](../../CONTRIBUTING.md#release-procedure).
Do not mark this last gate passed by citing an older commit's CI or a local Linux
success. Capability implementation, resource evidence and release readiness are
separate statuses.
