# Current architecture

[Documentation](../README.md) / Development

This page describes implemented code. Proposed capabilities and their acceptance
gates belong in the [kernel design](../design/kernel.md).

Cocycle is one Rust crate with a small public API. Modules follow mathematical
responsibilities without requiring each computation to construct every possible
intermediate object. The [mathematical specification](../reference/mathematics.md)
defines invariants; rustdoc defines public signatures and error behavior.

## Production code

```text
src/
  lib.rs                         public modules and Error/Result exports
  error.rs                       shared error types
  geometry/
    mod.rs                       public input types and private distance export
    point_cloud.rs               borrowed coordinate validation and access
    dissimilarity.rs             condensed symmetric dissimilarities
    euclidean.rs                 Euclidean distance construction
  diagram/
    mod.rs                       public result types
    interval.rs                  validated intervals and endpoint semantics
    persistence_diagram.rs       owned interval multiset and coverage validation
  descriptors/
    mod.rs                       public descriptor operations
    lifetime_statistics.rs       finite lifetimes and entropy
    betti_curve.rs               coverage-aware Betti curves
  filtration/
    mod.rs                       internal filtration access
    rips.rs                      simplex IDs, order, cofacets, cone bound
  persistence/
    mod.rs                       public re-exports and result normalization
    rips/
      mod.rs                     Rips entry points and caller-visible coverage
      options.rs                 validated RipsOptions
      h0.rs                      H0-only edge scan and pairing
      union_find.rs              shared private connectivity for H0/H1
      cohomology/
        mod.rs                   implicit F2 H1 reduction
        tests.rs                 optimization and duality checks
        profiling.rs             explicitly invoked diagnostic tests
    tests.rs                     H0/reference parity
    reference/                   explicit oracle, compiled only with cfg(test)
```

| Module | Responsibility | Dependencies within the crate |
| --- | --- | --- |
| `geometry` | Input validation and distance access | Error utilities |
| `filtration` | Rips ordering and simplex/cofacet access | Geometry, error utilities |
| `persistence` | Algorithms, options, and interval assembly | Geometry, filtration, diagram |
| `diagram` | Algorithm-independent result ownership and validation | Error utilities |
| `descriptors` | Read diagrams without recomputing persistence | Diagram, error utilities |

Geometry and diagram code do not call persistence. Filtration code does not call
persistence. Descriptors do not inspect source coordinates or algorithm state.
There are no public complex builders, boundary-matrix traits, backend registries,
or optimization flags.

Each implemented domain has a directory, even while its implementation is small.
The domain's `mod.rs` documents its scope and exports its public API; named child
files own concrete data invariants or computations. These files are private
implementation modules, so existing public type and function paths stay stable.
`lib.rs` and the shared `error.rs` remain crate-level files.

Here `geometry` includes nonmetric dissimilarities, `filtration` provides ordered
topological access, and `persistence` computes persistent homology using that
access. `diagram` is a mathematical result container, not plotting, and
`descriptors` computes measurements from diagrams. Geometric distance operations
belong to `geometry`; any future diagram-distance module needs a name that makes
its different input domain explicit.

## Computation path

Point clouds first become one condensed distance buffer. H0-only requests use
union-find. H0/H1 requests use the implicit Rips path: ordered edges supply H0
merges and candidate H1 births; triangle cofacets are generated during reverse
coboundary reduction. Stored change-of-basis columns reconstruct reduced columns.

Within `src/persistence/rips/cohomology/mod.rs`, `run` first classifies edges in forward
order, then reduces their coboundaries in reverse order. `find_shortcut` handles
apparent and emergent pairs on original columns; an unsuccessful search falls back to ordinary
reduction. `TransformColumn` stores `EdgePosition` values in the ordered edge
array, whereas `pivot_owners` maps triangle simplex IDs to `ColumnPosition` values
in the stored-column array. These private types prevent treating the two array
positions as interchangeable; combinatorial simplex IDs remain separate. The
explicit reduced-column payload exists only in tests.

H0 and H1 both use `src/persistence/rips/union_find.rs`, with private state.
The component only tracks connectivity; its callers decide when to stop scanning
and how a merge contributes persistence pairs. H1 does not import H0's algorithm.

`resolve_rips_range` selects the cutoff and caller-visible coverage. Both paths
normalize raw intervals in `assemble_diagram`. That step applies requested
dimensions, zero-lifetime removal, and the public coverage/censoring convention.
The internal cone stopping bound must not replace the caller's coverage.
Results own their data and contain no simplex IDs or borrowed work buffers.

The private `RipsFiltration` type supplies simplex access; `SimplexEntry` combines
a combinatorial ID with a filtration value. Neither name denotes a persistence
engine or an ordered-array position.

The current numeric choices are `f64` filtration values and F2 coefficients.
Generic interval types allow finite signed scales and arbitrary dimensions;
Rips entry points and Betti queries enforce their narrower supported contracts.
This representational flexibility is not a promise of additional algorithms.

## Test reference implementation

`src/persistence/reference/` contains `complex`, `boundary`, `explicit`, `rips`,
`column`, and `reduction`. The entire module is gated by `cfg(test)` and is absent
from production builds. It materializes a small 2-skeleton and performs ordinary
left-to-right sparse boundary reduction, independently of the optimized path.

Its `FilteredBoundary` contract requires finite nondecreasing filtration values,
strictly ordered boundary indices before their column, adjacent dimensions, and
boundary squared equal to zero. Sparse-column storage and reduction strategy are
separate. A hand-built filtered triangle verifies the reducer independently of
Rips construction. This is an internal oracle, not an advertised extension API.

Correctness tests belong beside private invariants or under `tests/` for public
contracts. Diagnostic counters and ignored profiling entry points are test-only;
`tools/profile_*.py` invokes them explicitly. Production timings come from the
public API workers, never instrumented test builds.

## Repository support code

- `examples/`: runnable public-API usage.
- `tests/`: external callers' contracts, hand calculations, and properties.
- `benches/rips.rs`: dependency-free Rust benchmark.
- `tools/`: optional external comparisons, benchmark controllers, and diagnostics.
- `docs/`: usage, reference, development, design, and upstream research.
- `benches/`: dated reports and raw experiment evidence, outside the crate payload.

## Extension rules

Keep public paths stable when splitting a file into private submodules. Add a new
public abstraction only when a concrete capability needs it and its validation
contract can be specified. A second filtration need not use a simplex-based
representation: it can share diagrams while owning its own input and algorithm.

Higher-dimensional Rips belongs at the filtration/cohomology boundary. Diagram
distances and landscapes can consume existing diagrams without changing Rips.
Other coefficient fields require explicit algebra and column semantics; F2 index
sets are not already a general field abstraction. Mapper need not flow through
persistent homology at all. These are boundaries for future work, not empty
modules to create now. See the [roadmap](../design/roadmap.md).

The [kernel design direction](../design/kernel.md) explains how these boundaries
can grow into a native Rust data analysis kernel. Its future capability plans
remain proposals. The [GUDHI C++ study](../research/gudhi-cpp.md) records the upstream
source evidence behind that direction.
