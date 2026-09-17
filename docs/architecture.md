# Architecture

Cocycle is one Rust crate with a small public API. Modules follow mathematical
responsibilities without requiring each computation to construct every possible
intermediate object. The [mathematical specification](mathematics.md) defines
invariants; rustdoc defines public signatures and error behavior.

## Production code

```text
src/
  lib.rs                         public modules and Error/Result exports
  error.rs                       shared error types
  geometry.rs                    borrowed inputs and Euclidean distances
  diagram.rs                     owned intervals, coverage, and diagrams
  descriptors.rs                 statistics and curves on existing diagrams
  filtration/
    mod.rs                       internal filtration access
    rips.rs                      simplex IDs, order, cofacets, cone bound
  persistence/
    mod.rs                       public entry points and result normalization
    options.rs                   validated RipsOptions
    h0.rs                        union-find and H0 computation
    rips_cohomology.rs            implicit F2 H1 reduction
    rips_cohomology/
      tests.rs                   optimization and duality checks
      profiling.rs               explicitly invoked diagnostic tests
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

## Computation path

Point clouds first become one condensed distance buffer. H0-only requests use
union-find. H0/H1 requests use the implicit Rips path: ordered edges supply H0
merges and candidate H1 births; triangle cofacets are generated during reverse
coboundary reduction. Stored change-of-basis columns reconstruct reduced columns.

Both paths normalize raw intervals in one place. That step applies requested
dimensions, zero-lifetime removal, and the public coverage/censoring convention.
The internal cone stopping bound must not replace the caller's coverage.
Results own their data and contain no simplex IDs or borrowed work buffers.

The current numeric choices are `f64` filtration values and F2 coefficients.
Generic interval types allow finite signed scales and arbitrary dimensions;
Rips entry points and Betti queries enforce their narrower supported contracts.
This representational flexibility is not a promise of additional algorithms.

## Test reference implementation

`persistence/reference/` contains `complex`, `boundary`, `explicit`, `rips`,
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
- `docs/`: current user, mathematical, architecture, and contributor references.
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
modules to create now. See the [roadmap](roadmap.md).
