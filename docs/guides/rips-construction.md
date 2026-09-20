# Exact Rips construction and supplied flag filtrations

[Documentation](../README.md) / Guides

These APIs construct weighted graphs, optionally expand inspectable simplicial
complexes, and compute ordinary prime-field persistence in arbitrary dimensions.
F2 is the default; [field selection and representatives](rips-representatives.md)
are opt-in. [Sparse Rips approximation](sparse-rips.md) is a separate implemented
path within the
[Rips design](../design/rips.md).

## Choose the mathematical input

| Input | Construction/computation | Missing-edge meaning |
| --- | --- | --- |
| Euclidean points | `threshold_rips_from_points`, then `compute_threshold_rips` | Omitted above the construction cutoff; original distances may enter later |
| Lower/upper/square matrix | `DissimilarityMatrixView`, `threshold_rips_from_distances` or `compute_rips_from_distances` | A threshold graph certifies only its recorded original-input range |
| Arbitrary objects and distance function | `threshold_rips_with_distance` | Symmetric sampled dissimilarities, with the same threshold semantics |
| User-supplied weighted graph | `WeightedGraph`, `FlagFiltration`, `compute_flag` | Missing edges never enter this supplied graph's clique filtration |

Exact construction requires finite nonnegative symmetric dissimilarities, not
triangle inequalities. Off-diagonal zeros do not identify vertices. Graphs retain
isolated vertices using an explicit count. An edge list does not certify that all
edges of some original metric below a cutoff were included.

## Construct, inspect and compute

```rust
use cocycle::diagram::{Coverage, IntervalEnd};
use cocycle::filtration::threshold_rips_from_points;
use cocycle::geometry::PointCloudView;
use cocycle::persistence::{ExecutionLimits, PersistenceOptions, compute_threshold_rips};

let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
let graph = threshold_rips_from_points(
    PointCloudView::new(&coordinates, 4, 2)?, Some(1.),
)?;
assert_eq!(graph.graph().vertex_count(), 4);
assert_eq!(graph.graph().edge_count(), 4);
assert_eq!(graph.coverage(), Coverage::Through(1.));
let result = compute_threshold_rips(
    &graph, &PersistenceOptions::default(), &ExecutionLimits::default(),
)?;
let interval = result.diagram().intervals_in_dimension(1)?.next().unwrap();
assert_eq!(interval.end(), IntervalEnd::RightCensored { through: 1. });
# Ok::<(), cocycle::Error>(())
```

All cutoffs include equality. Construction scans every pair, validates it, and
stores only retained edges. It owns the graph and releases the source borrow.
For n points in d coordinates and m retained edges, distance work is O(n^2 d),
with O(n+m) graph storage; this implementation does not use a spatial index.
Graph adjacency construction/sorting has O(n + m log m) cost and O(n+m) storage.

A cutoff at least the input diameter gives `Coverage::Complete`, including empty
and singleton inputs. A cutoff below it gives `Through(cutoff)`, even if the
largest stored edge is smaller. Computation with no new cutoff uses this available
range. Requesting a larger scale from an incomplete construction returns
`Error::IncompleteFiltration`; requesting a smaller scale is allowed.

## Borrow matrix layouts

```rust
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout};
use cocycle::persistence::{ExecutionLimits, PersistenceOptions, compute_rips_from_distances};

// Upper triangle in row order: d(0,1), d(0,2), d(1,2).
let values = [1., 2., 1.];
let input = DissimilarityMatrixView::new(&values, 3, MatrixLayout::UpperTriangle)?;
assert_eq!(input.get(2, 0), Some(2.));
let result = compute_rips_from_distances(
    input, &PersistenceOptions::default(), &ExecutionLimits::default(),
)?;
assert_eq!(result.context().characteristic(), 2);
assert_eq!(result.diagram().intervals_in_dimension(1)?.count(), 0);
# Ok::<(), cocycle::Error>(())
```

`LowerTriangle` is the existing condensed order; `UpperTriangle` uses increasing
rows above the diagonal. `Square` borrows n*n row-major values, validates zero
diagonal and exact symmetry, and rejects non-finite/negative entries. Signed zero
is accepted and normalized on access; original buffers are unchanged. Validation
and diameter caching take O(buffer length). Layout adaptation never copies the
matrix. Existing `DissimilarityView` converts with `.into()` without revalidation.

`compute_rips_from_points` is the richer point-cloud computation: with a finite
cutoff it builds a threshold graph directly; without one it materializes a
condensed matrix. The original `rips_from_points` and `rips_from_dissimilarities`
continue returning `PersistenceDiagram` with their existing contracts.

## Custom pairwise distances

`threshold_rips_with_distance(&objects, cutoff, callback)` accepts a closure
returning `Result<f64>`. Pairs are visited once in lower-triangle order, passing
the lower-index object first. Diagonal calls are omitted. The caller supplies a
stable symmetric function with zero self-distance; inspecting one orientation
cannot certify an arbitrary callback's behavior in the other orientation.
Every sampled value is checked, including values above the cutoff. Callback
failure aborts the whole construction. Objects are neither cloned nor retained.

## A supplied graph is its own filtration

```rust
use cocycle::complex::{WeightedEdge, WeightedGraph};
use cocycle::diagram::{Coverage, IntervalEnd};
use cocycle::filtration::FlagFiltration;
use cocycle::persistence::{ExecutionLimits, PersistenceOptions, compute_flag};

let edges = [[0,1], [1,2], [2,3], [0,3]].into_iter()
    .map(|vertices| WeightedEdge { vertices, value: 1. }).collect();
let input = FlagFiltration::new(WeightedGraph::new(5, edges)?); // Vertex 4 is isolated.
let result = compute_flag(&input, &PersistenceOptions::default(), &ExecutionLimits::default())?;
assert_eq!(result.diagram().coverage(), Coverage::Complete);
assert_eq!(result.diagram().intervals_in_dimension(1)?.next().unwrap().end(), IntervalEnd::Essential);
# Ok::<(), cocycle::Error>(())
```

Graph construction normalizes endpoint order and sorts edges, but rejects
self-loops, duplicate undirected edges, invalid endpoints and invalid values.
Zero-weight edges are retained. Neighbor slices are sorted by vertex index. A
complete supplied graph filtration can have essential H1, unlike a complete finite
Rips of all finite pairwise distances. A computation cutoff below the graph's
maximum edge conservatively censors surviving classes.

## Expand and inspect a Rips skeleton

```rust
use cocycle::filtration::threshold_rips_from_distances;
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout};
use cocycle::persistence::{ExecutionLimits, PersistenceOptions, compute_expanded_rips};
use cocycle::diagram::IntervalEnd;

// Three opposite vertex pairs have distance 2; all other pairs have distance 1.
let values: Vec<_> = (0..6).flat_map(|b| (0..b)
    .map(move |a| if a / 2 == b / 2 { 2. } else { 1. })).collect();
let input = DissimilarityMatrixView::new(&values, 6, MatrixLayout::LowerTriangle)?;
let graph = threshold_rips_from_distances(input, None)?;
let expanded = graph.expand(3)?; // H2 deaths require tetrahedra.
let complex = expanded.complex();
let triangle = complex.find(&[0, 2, 4]).unwrap();
assert_eq!(complex.boundary(triangle).unwrap().len(), 3);
assert_eq!(complex.cofacets(triangle).unwrap().len(), 3);
let result = compute_expanded_rips(
    &expanded, &PersistenceOptions::new(2, None)?, &ExecutionLimits::default(),
)?;
let sphere = result.diagram().intervals_in_dimension(2)?.next().unwrap();
assert_eq!((sphere.birth(), sphere.end()), (1., IntervalEnd::Finite(2.)));
# Ok::<(), cocycle::Error>(())
```

`expand(max_simplex_dimension)` stores all cliques through that dimension,
including faces. Dimension zero builds vertices only. `FlagFiltration::expand`
also returns an inspectable `FilteredSimplicialComplex`, with the supplied graph's
missing-edge semantics. Expansion can have exponential time and storage costs.

Simplex vertices are increasing; lookup requires that canonical order. Simplex
IDs refer to positions in one frozen complex, never to another complex's IDs.
The global order is increasing filtration value, then dimension, then decreasing
colexicographic vertex order. Boundary terms omit successive vertices with
alternating +1/-1 coefficients. `cofacets` returns direct, codimension-one
cofaces only. These queries describe stored topology, not missing higher cells.

`RipsExpansion` additionally owns the source kind, scale coverage, requested
construction dimension and an exhaustion certificate. Computing Hq requires
construction through q+1 unless `is_dimension_complete()` proves that no higher
cliques exist. Otherwise `compute_expanded_rips` returns
`Error::InsufficientSkeleton`, even when the skeleton itself has top-dimensional
cycles. Scale coverage is checked independently; graph exhaustion at a cutoff
does not turn a censored Rips interval into an essential one. The current dimension
check is conservative for a subsequent smaller computation cutoff.

## Execution controls and results

`PersistenceOptions::new(max_homology_dimension, cutoff)` accepts arbitrary
dimensions and defaults to F2. `with_field(PrimeField)` selects a validated prime
characteristic. Dimensions above the input vertex count are
computed empty; this does not allocate a buffer per requested dimension. The
legacy `RipsOptions` API retains its H0/H1 limit.
`ExecutionLimits::new(max_work, cancellation_flag)` optionally limits documented
work units and borrows an `AtomicBool` cancellation flag. Checkpoints cover edge
scans, cofacet candidate tests, reduction steps and heap removals. Sparse candidate
tests are neighbor comparisons; dense tests visit candidate vertices. Repeated
work counts again. Dimension-generic computation also counts traversal
checkpoints, candidate vertices, adjacency tests and transformation additions.
These units are not comparable elapsed times.

Sorting, allocations, index-table/vertex initialization and result normalization are not
internally interruptible. Cancellation is checked at surrounding phases. Controls
apply to persistence, not standalone graph/distance construction or explicit
expansion. The optional representative computation includes its own skeleton
materialization in the budget. They are not memory/RSS or
wall-time bounds. A cancelled/limited computation returns an error without a
successful partial diagram; retrying a call starts with fresh private state.

Both dense and sparse H1 use checked combinatorial IDs for edges and triangles.
The total combinatorial count must fit `usize` even for a graph with few edges.
Sparse access uses O(n+m) storage, but reduction fill-in and working heaps can
still grow substantially. Higher-dimensional requests use vertex tuples instead
of bounded combinatorial IDs, retain one dimension at a time, and regenerate
coboundaries from transformation columns. Explicit computations use stored
incidence with the same generic reducer. No fixed point-count threshold guarantees
completion; neither path is a hard memory bound.

`PersistenceResult` owns a diagram and context: source kind, vertex count, field,
requested computation cutoff and optional construction cutoff. Identity mapping
`0..vertex_count` is preserved. Coverage and computed dimensions stay in the
diagram. Existing descriptors accept `result.diagram()` directly. No coordinates,
graph buffers or working columns are retained by the result. Requested
representatives additionally retain owned simplex/coefficient terms.

Run `cargo run --locked --example rips_graph` and
`cargo run --locked --example flag_persistence`. For H2 and topology queries, run
`cargo run --locked --example rips_sphere`. See the
[native comparison command](../../tools/README.md#native-rips-correctness-checks)
for construction and persistence validation against C++ references.
