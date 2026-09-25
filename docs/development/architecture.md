# Current architecture

[Documentation](../README.md) / Development

This page describes implemented code. Proposed capabilities and their acceptance
gates belong in the [kernel design](../design/kernel.md).

Cocycle is one Rust crate with a small public API. Modules follow mathematical
responsibilities without requiring each computation to construct every possible
intermediate object. The [mathematical specification](../reference/mathematics.md)
defines invariants; rustdoc defines public signatures and error behavior.

Algorithm authors can start with the [contribution paths](algorithm-contributions.md)
for focused diagram-analysis and explicit-construction workflows. This page owns
the broader dependency map used for kernel integration.

## Production code

```text
src/
  lib.rs                         public domains and Error/Result exports
  error.rs                       shared structured errors
  execution/
    mod.rs                       shared controls and private per-operation budget
  algebra/
    field/mod.rs                 validated PrimeField and modular arithmetic
    column/mod.rs                private ordered sparse coefficient columns
    reduction/boundary.rs        boundary reduction; optional basis transformations
  geometry/
    point_cloud.rs               borrowed coordinate validation
    dissimilarity.rs             existing condensed input contract
    matrix.rs                    borrowed lower/upper/square matrix layouts
    metric.rs                    explicit metric policy and exhaustive triangle checks
    distance.rs                  checked scalar distance/cutoff operations
    euclidean.rs                 shared pair evaluation and dense conversion
  complex/
    filtered.rs                  public four-method FilteredComplex contract
    graph/
      mod.rs                     owned WeightedGraph, edges and queries
      adjacency.rs               compact sorted adjacency construction
    simplicial/
      mod.rs                     validated SimplicialComplex, lookup and stored incidence
      simplex.rs                 canonical vertices, IDs and shared filtration order
      incidence.rs               oriented codimension-one boundary terms
  filtration/
    provenance.rs                construction coverage and source-kind identity
    context.rs                   typed source metadata and declared scale convention
    expansion/mod.rs             source-specific controlled expansion adapters
    simplicial/
      mod.rs                     contextual explicit result and source certificates
      access.rs                  private zero-born coface access for Rips/flag paths
    rips/
      builder.rs                 exact borrowed settings and one-shot callbacks
      exact.rs                   ThresholdRips construction, provenance, cone bound
      expansion.rs               explicit Rips and dimension/scale provenance
      approximation/
        builder.rs               approximate settings and one-shot callbacks
        metadata.rs              owned sampling provenance and conditional bounds
        options.rs               epsilon, sampling and range parameters
        greedy.rs                deterministic farthest-point permutation
        edges.rs                 modified sparse edge values
        blocker.rs               hereditary insertion-radius constraint
        access.rs                original-label blocker-aware cofaces
        expansion.rs             frozen sparse topology and provenance
    flag/
      mod.rs                     supplied FlagFiltration
      access.rs                  private dense/sparse access contract
      dense.rs                   matrix-backed edges and cofacets
      sparse.rs                  sorted neighbor-intersection cofacets
      index.rs                   checked edge/triangle IDs and decoding
      order.rs                   compact H1 entry comparison adapter
      cliques.rs                 dense/sparse clique enumeration
      expansion.rs               shared explicit clique construction
  persistence/
    mod.rs                       public exports and result normalization
    execution.rs                 compatibility control imports
    builder.rs                   borrowed analysis settings and validated execution
    source.rs                    sealed extension and private source dispatch
    options.rs                   compatibility options and signed-scale validation
    filtered.rs                  generic filtered-cell boundary input and diagrams
    union_find.rs                connectivity shared by specialized H0/H1 paths
    simplicial/
      mod.rs                     explicit analysis and certified range handling
      cohomology.rs              zero-born prime-field coface reduction with clearing
      representatives/
        basis.rs                 shared cycles and interval association
        request.rs               dimension, signed scale and selection
        complex.rs               zero-born implicit skeleton materialization
        dual.rs                  scale-specific dual cocycle basis
    rips/
      mod.rs                     legacy/new Rips entry points and source coverage
      options.rs                 compatible RipsOptions
      expanded.rs                explicit incidence computation and dimension checks
      approximation.rs           sparse Rips computation and context assembly
    flag/
      mod.rs                     shared dispatch and supplied-graph entry point
      h0.rs                      independent H0 edge scan
      cohomology/
        mod.rs                   shared dense/sparse implicit F2 H1 reduction
        tests.rs                 optimization/duality tests
        profiling.rs             explicit diagnostic tests
    tests.rs                     independent Rips/graph oracle comparisons
    reference/                   test-only explicit boundary reduction
  diagram/
    interval.rs                  interval validation and endpoint semantics
    persistence_diagram.rs       owned multiset and coverage
    computation.rs               owned PersistenceResult and source context
    representative.rs            owned chain/cochain terms and local interval IDs
  descriptors/                   diagram-only lifetimes and Betti curves
```

Each domain has a documenting/exporting `mod.rs`; the tree lists the substantive
files. Public paths are re-exported from domains, not every private directory.

| Module | Responsibility | Dependencies within the crate |
| --- | --- | --- |
| `algebra` | Validated prime fields and private sparse arithmetic/reduction | Error utilities |
| `geometry` | Input validation and distance access | Error utilities |
| `complex` | Checked graphs, simplicial incidence and filtered-cell contract | Geometry scalar validation, execution, error utilities |
| `execution` | Immutable controls and private per-operation budget | Error utilities |
| `filtration` | Construction, source context, coverage and private coface access | Geometry, complex, execution, error utilities |
| `persistence` | Algorithms, options, interval and representative assembly | Algebra, geometry, filtration, diagram, execution |
| `diagram` | Algorithm-independent result ownership and validation | Algebra field identity, filtration provenance, error utilities |
| `descriptors` | Read diagrams without recomputing persistence | Diagram, error utilities |

Geometry and diagram code do not call persistence. Filtration code does not call
persistence. Descriptors do not inspect source coordinates or algorithm state.
Public graph construction and frozen explicit simplicial expansion are available.
The public `FilteredComplex` contract drives generic boundary reduction. It does
not require simplex vertex lists, construction methods or mutable algorithm keys.
No backend registry or Alpha/cubical construction is implied. See the
[filtered-complex guide](../guides/filtered-complexes.md) for source semantics.

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

## Public workflow and ownership

`RipsBuilder` and `ApproximateRipsBuilder` configure borrowed inputs.
`build_complex(max_simplex_dimension)` performs explicit construction and returns
an owned `SimplicialFiltration`; only expanded topology exposes simplex queries.
Alternatively, importing `persistence::PersistenceExt` enables `.persistence()`
and the returned request's `.compute()`. The extension is implemented entirely
in the consumer module, so filtration never calls persistence.

Advanced `prepare()` returns an owned exact/approximate source for reuse. Callback
builders are consumed by preparation or explicit construction; analyses never
replay them. `Execution` is shared configuration, not a shared counter. A new
private budget spans each terminal's preparation and computation. Legacy free
functions retain their existing persistence-only scope and test coverage.

## Computation path

The legacy point API and unrestricted richer point API use a condensed distance
buffer. The richer point API with a finite cutoff streams distances into a
threshold graph. Supplied graphs never require a dense matrix. H0-only requests use
union-find. F2 H0/H1 requests use the specialized implicit Rips path: ordered edges supply H0
merges and candidate H1 births; triangle cofacets are generated during reverse
coboundary reduction. Stored change-of-basis columns reconstruct reduced columns.

Within `src/persistence/flag/cohomology/mod.rs`, `run_access` first classifies edges in forward
order, then reduces their coboundaries in reverse order. `find_shortcut` handles
apparent and emergent pairs on original columns; an unsuccessful search falls back to ordinary
reduction. `TransformColumn` stores `EdgePosition` values in the ordered edge
array, whereas `pivot_owners` maps triangle simplex IDs to `ColumnPosition` values
in the stored-column array. These private types prevent treating the two array
positions as interchangeable; combinatorial simplex IDs remain separate. The
explicit reduced-column payload exists only in tests.

Higher-dimensional and odd-prime implicit Rips/flag requests dispatch to
`simplicial/cohomology.rs`. This zero-born path
classifies H0 edges, then advances through dimensions with clearing. It retains
one ordered simplex dimension, pivot owners and coefficient-bearing transformation
columns; reduced
coboundaries are regenerated on demand. Ordered vertex tuples avoid binomial-ID
overflow in sparse high-dimensional input. `flag/cliques.rs` supplies dense candidates or common neighbors from the shortest
sparse adjacency list through the private `ZeroBornSimplicialAccess` contract.
The compatibility expansion entry points also adapt their zero-born stored
incidence to this contract. The specialized H1
path keeps its existing apparent/emergent shortcuts; the generic path currently
uses clearing without those shortcuts. Oriented cofacets use the sign of their
omitted vertex; pivot columns are normalized over the selected field. No claim
of performance parity is made.

Explicit expansion uses unique increasing-vertex extensions and freezes all
stored simplices with lookup and both directions of incidence. Its ordering and
the compact H1 entry order use the comparison authority in `complex/simplicial`.
`SimplicialFiltration` retains construction dimension and scale provenance
separately for exact Rips, approximation and supplied flags. The old expansion
types remain available during migration.
Explicit builder computation uses the filtered-cell boundary reducer and reads
stored incidence, rejecting insufficient skeletons unless expansion certified
clique exhaustion. Graph construction and expansion do not
invoke persistence, and computation does not mutate stored topology.

H0 and H1 both use `src/persistence/union_find.rs`, with private state.
The component only tracks connectivity; its callers decide when to stop scanning
and how a merge contributes persistence pairs. H1 does not import H0's algorithm.

`resolve_rips_range` selects the cutoff and caller-visible coverage. Both paths
normalize raw intervals in `assemble_diagram`. That step applies requested
dimensions, zero-lifetime removal, and the public coverage/censoring convention.
The internal cone stopping bound must not replace the caller's coverage.
Results own their data and contain no simplex IDs or borrowed work buffers.

Private `DenseFlag` and `SparseFlag` implement `FlagAccess`: forward-ordered
edges, decreasing-ID cofacets and latest-facet lookup. The engine provides a
checkpoint callback so filtration never depends on persistence execution types.
`SimplexEntry` combines a combinatorial ID and value, not an array position.
Sparse cofacets intersect two sorted adjacency lists and retain the dense tie
order. The cone bound applies only to complete pairwise input, not arbitrary
supplied graphs.

`ThresholdRips` certifies coverage against all original pairs. `FlagFiltration`
defines the supplied graph itself, including permanently absent edges. Their
entry points select coverage before calling the shared engine. `PersistenceResult`
adds owned source context without changing the diagram or descriptors. Explicit
CSR-like offsets and neighbors belong to graph storage; pivots and heaps stay
private to each computation.

Requested representatives use `simplicial/representatives`. Implicit sources
materialize the required skeleton and assemble oriented boundaries; explicit
builder sources read stored boundaries through `FilteredComplex`. `algebra/reduction` owns
ordinary forward reduction and transformations, independently of filtration and
diagram types. `algebra/column` supplies sparse coefficient operations shared
with implicit cohomology and dual solves. The private test oracle remains separate.
Finite cycles use reduced death columns; unpaired cycles use birth transformations.
`representatives/dual.rs` solves scale-specific boundary-annihilation and cycle
pairing constraints. Terms leave the computation as original vertex lists and
canonical coefficients, associated with positions in this result's sorted diagram.
These opt-in entry points return the same diagram as their implicit counterparts.

The numeric choices are `f64` filtration values and validated prime-u32 fields,
with F2 the default.
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

The high-dimensional oracle in `tests/rips_expansion.rs` independently enumerates
vertex subsets by bitmask and reduces the ordinary boundary matrix forward.
It does not reuse production clique access, clearing or implicit reconstruction.

Correctness tests belong beside private invariants or under `tests/` for public
contracts. Diagnostic counters and ignored profiling entry points are test-only;
`tools/profile_*.py` invokes them explicitly. Production timings come from the
public API workers, never instrumented test builds.

## Repository support code

- `examples/`: runnable public-API usage.
- `tests/`: external callers' contracts, hand calculations, and properties.
- `benches/rips.rs`: dependency-free Rust benchmark.
- `benches/native/`: native H0/H1 workers, shared source pins and setup.
- `benches/pipeline/`: native complete-workflow workers and timing boundaries.
- `tools/`: optional external comparisons, benchmark controllers, and diagnostics.
- `docs/`: usage, reference, development, design, and upstream research.
- `benches/reporting.md`: cross-suite report and evidence rules.
- `benches/reports/`: maintained comparisons with measured revisions and evidence status.
- `target/`: ignored local build and experiment output; generated artifacts never
  enter the source repository.

## Extension rules

Keep public paths stable when splitting a file into private submodules. Add a new
public abstraction only when a concrete capability needs it and its validation
contract can be specified. A second filtration need not use a simplex-based
representation: it can share diagrams while owning its own input and algorithm.

Higher-dimensional Rips belongs at the filtration/cohomology boundary. Diagram
distances and landscapes can consume existing diagrams without changing Rips.
Non-prime coefficient rings would require different algebra and reduction
semantics; the implemented prime-field column API does not support them. Mapper need not flow through
persistent homology at all. These are boundaries for future work, not empty
modules to create now. See the [roadmap](../design/roadmap.md).

The [kernel design direction](../design/kernel.md) explains how these boundaries
can grow into a native Rust data analysis kernel. Its future capability plans
remain proposals. The [GUDHI C++ study](../research/gudhi-cpp.md) records the upstream
source evidence behind that direction.

## Sparse approximation boundary

Sparse Rips construction owns its compact graph, insertion radii and blocker.
The graph alone is insufficient to reconstruct its higher topology. Its access
implementation and explicit incidence implement the same `SimplicialAccess`
contract, including zero-born original vertex labels and a compact position map
for union-find. Oriented generic cohomology and representative reduction reuse
this access without approximation-specific algebra. The blocker is hereditary,
so unique-extension enumeration may prune rejected simplices safely.

`filtration/rips/approximation/metadata.rs` owns numerical parameters, full greedy order and radii,
retained IDs, metric evidence and conditional bound targets. This module depends
only on geometry's evidence enum, not on a borrowed distance input or algorithm.
Descriptors still consume ordinary diagrams; a caller choosing to discard result
context also discards approximation provenance. See the [sparse guide](../guides/sparse-rips.md).

## Shared filtered-complex boundary

`complex::SimplicialComplex` is the canonical concrete container;
`FilteredSimplicialComplex` is a compatibility alias. It validates face closure,
unique simplices and filtration monotonicity before freezing. The public
`FilteredComplex` trait exposes only cells, dimensions, values and integer
boundaries. `persistence/filtered.rs` reads this contract without graph or Rips
assumptions. Its diagram-only reducer retains no representative transformations.
Explicit simplicial representatives reuse this boundary input and the existing
basis/dual extraction. Direct Rips engines retain their zero-born coface contract.

`filtration::SimplicialFiltration` is a certificate-bearing construction result,
not a second topology container. It owns `SimplicialComplex` plus source coverage
and dimension evidence. `ComputationContext` reuses `FiltrationContext`, whose
typed source keeps Rips approximation metadata local to the Rips variant. Stored
simplex values determine the maximum filtration value; maximum edge value is not
a generic completeness certificate. Source units are explicit or unspecified.

Supplied signed filtrations and non-simplicial cell adapters are covered by
`tests/filtered_complex.rs`. Alpha geometric predicates and triangulation remain
future geometry work; no empty directories or placeholder constructors exist.
