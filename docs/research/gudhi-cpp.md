# GUDHI C++ capability and architecture study

[Documentation](../README.md) / Research

Status: source research, recorded 2026-09-20. This is a capability map and a
reading plan, not a claim of feature parity or a verification of every algorithm.

## Scope and evidence

The source baseline is the official GUDHI development repository at
`cba915e3ab8e1f5b1fe26eb44b407285f7af4e78`, committed on 2026-09-15 and
identified by its version file as `3.14.0a0`. Links below pin that revision.
Documentation for another release must not silently replace this baseline.

The inventory covers all 22 C++ capability directories registered by the module
build, plus `common` and build support. Python wrappers and GudhUI are excluded.
Coverage means reading module descriptions, entry headers, and selected call
paths; it does not mean every implementation, test, or proof has been audited.
No GUDHI binaries were built or performance measurements made for this report.

The most closely traced paths are Rips construction, Simplex tree storage,
filtered-complex access, CAM persistence, cubical adaptation, persistence
matrices, zigzag, and the integrated Ripser engine. Remaining modules are mapped
from their C++ entry points and source documentation; their detailed mathematical
and numerical audits remain reading tasks below.

The [kernel design](../design/kernel.md) separates our proposed architecture from
these source observations. [Current architecture](../development/architecture.md) describes
only what Cocycle implements today.

## What the project contains

The grouping is analytical; these are not literal parent directories in GUDHI.
A module can span construction, storage, and operations.

| Source module | Input and responsibility | Output or connection | Rust implementation consideration |
| --- | --- | --- | --- |
| [Rips_complex][g-rips] | Distances/points to a filtered proximity graph; clique expansion through a receiving complex | Explicit filtered simplicial complex; also includes sparse Rips approximation | Separate exact graph storage from the approximation algorithm |
| [Ripser][g-ripser] | Distance representations, implicit simplex encoding and cofacet enumeration | Specialized persistent cohomology with output callbacks | Preserve a specialized path without explicit simplex materialization |
| [Alpha_complex][g-alpha-doc] | Delaunay/regular triangulations and geometric filtration construction; weighted and 3D variants | Receiving simplicial complex | Robust predicates, degeneracies, and numerical construction are a substantial separate project |
| [Cech_complex][g-cech] | Points, proximity graph, minimal enclosing balls | Cech complex and MEB filtration assignment | Radius conventions differ from Rips edge lengths and some squared-radius APIs |
| [Witness_complex][g-witness] | Witness-to-landmark distance tables; weak/strong and Euclidean variants | Receiving simplicial complex | Separate landmark selection, distance access, and witnessing rules |
| [Bitmap_cubical_complex][g-cubical-doc] | Scalar values on grid vertices or top cells, including periodic boundary variants | Grid-based filtered complex implementing persistence access | Define layout, value propagation, masks, and boundary conditions explicitly |
| [Simplex_tree][g-tree] | Mutable simplices and their filtration values | Stored topology, traversal, expansion, filtration access | One possible explicit storage backend, not a prerequisite for persistence |
| [Hasse_complex][g-hasse] | Explicit codimension-one incidence relations | Filtered-complex access | Marked private in the header; do not mistake every installed header for a stable public API |
| [Skeleton_blocker][g-blocker] | Graph plus missing faces | Implicit simplicial storage supporting links and modifications | Useful for particular operations; not interchangeable with Ripser's implicit representation |
| [Toplex_map][g-toplex] | Maximal simplices and vertex-to-maximal-simplex incidence | Compressed storage and structural operations | Select storage by required operations, not by one universal complex type |
| [Persistent_cohomology][g-cam] | Filtered complex and coefficient field | CAM persistent pairs; also contains line/rectangle specializations | Keep immutable input separate from reducer state; specializations may coexist |
| [Persistence_matrix][g-matrix] | Boundary columns and matrix operations | Configurable matrices, pairings, representatives, and updates | Start with actual use cases rather than porting the entire option space |
| [Zigzag_persistence][g-zigzag] | Cell insertion/removal event sequence | Interval events using persistence matrices | Ordinary persistence and zigzag have different indexing and interval contracts |
| [Collapse][g-collapse] | Filtered flag graph | Smaller flag filtration preserving persistent homology under its algorithm's conditions | A useful exact preprocessing path; specify preserved semantics and mappings |
| [Contraction][g-contraction] | Simplicial structure plus validity/cost/placement policies | Simplified complex, based on Skeleton blocker | A homotopy-preserving contraction is not automatically a persistence-preserving filtration transformation |
| [Spatial_searching][g-search] | Geometric points and neighbor queries | Index-based CGAL search wrapper | Need a native Rust implementation or appropriate Rust dependency for a pure Rust core |
| [Subsampling][g-sampling] | Points and distance operations | Random, farthest-point, and separation-based subsets | Return source indices; specify randomness and assumptions about metrics |
| [Bottleneck_distance][g-bottleneck] | Persistence-diagram ranges | Diagram distance using matching and geometric search | Define infinity/censoring policies; geometry acceleration is an implementation choice |
| [Persistence_representations][g-representations] | Persistence intervals/diagrams | Landscapes, grid landscapes, heat maps, vectors, distances and kernels | Important analysis capabilities beyond computing diagrams |
| [Nerve_GIC][g-gic-doc] | Point cover, scalar filter, and graph | Nerve/graph-induced complexes plus statistical workflows | Has graph-valued output; must not be forced through a persistence-diagram API |
| [Tangential_complex][g-tangential-doc] | Samples of a low-dimensional manifold | Local triangulation-based reconstruction with consistency handling | Requires geometry and numerical infrastructure; perturbation is not a universal success guarantee |
| [Coxeter_triangulation][g-coxeter] | Ambient triangulation and intersection oracle for a manifold | Piecewise-linear manifold approximation | A distinct reconstruction capability, not ordinary persistence computation |

`common` provides assorted I/O, distance operations, points, allocators, pools,
random helpers, and utilities. These are shared support, not a single universal
mathematical abstraction layer.

### Important capabilities inside broader modules

- [Sparse_rips_complex][g-sparse] builds a mathematically controlled approximation.
  A sparse distance representation in Ripser is a different concept. Omitting
  distant edges at a declared cutoff can preserve the exact restricted flag
  filtration; arbitrary edge deletion cannot be called exact Rips.
- [Persistence_on_a_line][g-line] provides a specialized linear-time computation
  for sublevel persistence of a PL function on a line.
- [Persistence_on_rectangle][g-rectangle] contains a specialized 2D cubical
  computation. Its entry point is marked private, and its input layout differs
  from the generic bitmap cubical implementation.
- [Sliced_Wasserstein][g-sw] implements a sliced-Wasserstein representation/kernel.
  This is not the same operation as an ordinary diagram p-Wasserstein matching
  distance. Do not infer C++ capability parity from Python API names.

## Physical organization and dependency boundaries

A typical development module contains:

```text
src/Module/
  include/gudhi/Public_header.h
  include/gudhi/Module/...
  concept/
  doc/
  example/
  test/
  benchmark/
  utilities/
```

Subdirectories vary by module. The build registers modules and conditionally
builds examples/tests/utilities; release packaging merges headers into
`include/gudhi/`. A source module is not necessarily an isolated compiled
library. See [module registration][g-modules] and [release packaging][g-package].

The code primarily uses header-based templates. Its documented concepts describe
structural requirements, not a common virtual base class. The inspected build
uses C++17; these concept documents should not be read as C++20 constraints.

The following selected edges were checked against includes in module headers:

| Consumer | Included capability modules, excluding common support |
| --- | --- |
| Persistent_cohomology | No direct Simplex tree or Persistence matrix include |
| Zigzag_persistence | Persistence_matrix |
| Nerve_GIC | Simplex_tree, Rips_complex, Persistent_cohomology, Bottleneck_distance |
| Tangential_complex | Simplex_tree, Spatial_searching |
| Contraction | Skeleton_blocker |
| Rips_complex | Subsampling, through its sparse construction |
| Witness_complex | Spatial_searching, through Euclidean variants |
| Persistence_representations | Bottleneck_distance, through the distance-enabled intervals header |

This is a static header scan, including conditional includes. It is not a claim
that every instantiation requires every listed module. Conversely, template
requirements can create an integration dependency without a direct include.

Some C++ features directly depend on CGAL, Eigen, Boost, or optional TBB.
For example, Alpha construction relies on CGAL triangulations, while ordinary
Rips construction does not require that geometry machinery. Dependency cost must
be assessed per feature and transitive include path.

## Four computation paths to understand

### Explicit simplicial construction and CAM

`Rips_complex::create_complex` calls `insert_graph` and `expansion` on a
template argument satisfying [SimplicialComplexForRips][g-rips-contract].
Simplex tree is a receiving model, not a type hard-coded into this constructor.

Simplex tree stores topology and filtration values together and offers multiple
storage policies. Filtration ordering, storage handles, and algorithm keys are
different concepts. Mutable filtration operations also carry cache and
monotonicity responsibilities.

`Persistent_cohomology<FilteredComplex, CoefficientField>` traverses a
[filtered-complex contract][g-contract] and computes a compressed annotation
matrix. It owns its CAM columns, transverse structures, union-find, and pools.
It does not use `Persistence_matrix::Matrix`.

The algorithm holds a mutable reference to the input, writes keys, and returns
pairs containing input handles. Thus storage implementation is abstracted, but
input state and result lifetime are still coupled to the computation.
The actual implementation also calls operations such as `dimension()` and
`endpoints()`; adapting a new model requires checking calls as well as the
concept document.

### Cubical storage and the same CAM engine

The [cubical example][g-cubical-example] instantiates the same CAM engine directly
with `Bitmap_cubical_complex`. No Simplex tree conversion is required.

The grid determines incidence implicitly, while the adapter exposes the
filtration, boundary, key, and handle operations needed by the engine. This is
concrete evidence that GUDHI is not globally tied to Simplex tree.

### Boundary columns and persistence matrices

The [Simplex-tree-to-matrix example][g-adapter] enumerates simplices in filtration
order, assigns indices, constructs sorted boundary columns, and inserts them into
matrices. Matrix options select storage and operations such as pairings,
representative cycles, removals, and swaps.

Zigzag is an actual consumer of this reusable matrix machinery. Its event stream
and interval indexing are separate from a static ordinary filtration.

### Implicit Rips and its specialized engine

The [Ripser header][g-ripser] states its upstream base and subsequent heavy
refactoring. It includes distance representations, simplex encodings, dense and
sparse cofacet enumeration, and a specialized persistence engine.

The implementation does not route computation through Simplex tree, CAM's
FilteredComplex adapter, or the generic persistence matrix. Multiple engines
coexist because they exploit different information about their input.

These four paths describe major architecture relationships, not every specialized
algorithm in the source tree.

## Is Simplex tree the mandatory core?

No. It is important to distinguish three levels:

1. Rips/Alpha/Witness construction uses receiving-complex template requirements.
2. CAM needs its access and mutable-key protocol, which cubical storage implements.
3. Some higher-level modules have concrete coupling: [Nerve_GIC][g-gic] directly
   creates Simplex tree objects internally; [Tangential_complex][g-tangential]
   includes it and documents an export API around it, alongside another export.

The useful lesson is to permit specialized representations. The engineering
improvement is to make each operation's actual contract and ownership explicit,
rather than to promise that every backend implements one universal complex API.

## Rust porting implications

The implementation unit should be a mathematical operation with an observable
contract, not a C++ directory or class.

| Concern | Keep | Redesign or make explicit |
| --- | --- | --- |
| Mathematical basis | Definitions, algorithm invariants, proofs and applicable hypotheses | Executable validation and independent tests for the implemented domain |
| Specialized algorithms | Implicit Rips, grid-specific algorithms, dedicated matrix operations | A consistent result boundary without forcing identical internal storage |
| Input access | Ranges and operation-specific interfaces | Borrowed validated Rust views and clear layout/copy costs |
| Working state | Efficient indexing and reusable buffers | Reducer-owned arrays, typed indices, no mutation of shared topology just to store pivots |
| Results | Multiplicity and mathematical interval information | Owned results, coverage and censoring, scale conventions, optional witnesses |
| Geometry | Robust construction requirements | Native predicates/triangulation or audited Rust dependencies; floating-point translation alone is insufficient |
| Policies | Real choices in representation or computation | Small option types tied to supported use cases instead of a universal option matrix |
| Analysis | Landscapes, distances, sampling, graph outputs | Composable array/range APIs, explicit preprocessing and reproducibility |

Native Rust means no foreign TDA or geometry runtime hidden beneath the public
API. It does not require rewriting every general-purpose Rust dependency.
Dependency selection needs a concrete capability, license, maintenance, and MSRV
review. No third-party Rust library is selected by this study.

The current GUDHI license is MIT for GUDHI code, while dependencies and bundled
third-party files have separate terms. Consult the
[official licensing page](https://gudhi.inria.fr/licensing/) and exact source
revision before incorporating implementation code. A translation of source code
still needs attribution; a paper citation alone is not a software license.

## Reading sequence and evidence to produce

For every reading unit, record input/output, assumptions, storage, ordering,
mutable state, complexity, numerical guarantees, failure modes, dependencies,
and a tiny hand-computable example. Separate source behavior from proposed Rust
behavior. Benchmark claims require a runnable experiment, not reading comments.

| Unit | Read | Completion evidence |
| --- | --- | --- |
| 1. Rips construction | Rips constructor, receiving concept, Simplex tree expansion | Trace a four-vertex square through graph, triangle birth, and cutoff |
| 2. Explicit storage | Simplex tree handles, keys, filtration cache; contrast Hasse, Toplex, Skeleton blocker | List operations and invalidation rules; do not claim all models satisfy the same contract |
| 3. Ordinary persistence | FilteredComplex, CAM columns, field code, H0 special case | Hand-reduce a triangle; explain every key mutation and returned handle |
| 4. Specialized Rips | Ripser encoding, cofacets, apparent/emergent pairs, sparse dispatch | Reconstruct one H1 pivot and tie case; compare to an independent explicit reduction |
| 5. Scalar/grid persistence | Bitmap cubical, line and rectangle algorithms | Contrast vertex versus top-cell values, negative scales, and periodic boundaries |
| 6. Analysis outputs | Diagram ranges, bottleneck, landscapes, heat maps, sliced Wasserstein | Work a two-interval example; state infinity and approximation policies |
| 7. Scaling and simplification | Sampling, spatial queries, sparse Rips, edge collapse | Distinguish changed input, controlled approximation, and exact simplification |
| 8. Reusable algebra | Persistence matrix options and the explicit adapter | Identify the minimum operations needed for ordinary persistence and representatives |
| 9. Time-varying topology | Zigzag insert/remove and filtered wrapper | Trace a valid event sequence and interval endpoint conventions |
| 10. Geometric constructions | Alpha, Cech, Witness | Explain units, degeneracies, metric assumptions, and required geometry services |
| 11. Graph and reconstruction | Nerve/GIC, Contraction, Tangential, Coxeter | Identify non-diagram outputs and the hypotheses behind preservation or reconstruction |

This report completes the source map and selected architecture traces. The
worked examples, proof audits, native comparison harnesses, and Rust algorithms
in these units are subsequent work, not completed deliverables.

## What to compare when implementing a capability

Use native GUDHI C++ entry points for the relevant operation, pinned to a source
revision. Python wrappers are not the implementation baseline for this plan.

- Align coefficient field, requested homology dimensions, filtration convention,
  cutoff inclusion, tie handling, and zero-length interval policy.
- Account for one higher simplex dimension when it is needed to kill classes in
  the requested homology dimension.
- Compare interval multisets, not arbitrary storage handles or representative
  choices. Separately verify representative correctness when requested.
- Normalize external infinite endpoints only with known computation coverage.
  An unpaired interval at an incomplete cutoff is not evidence of essentiality.
- Distinguish radius from squared radius and Rips diameter/edge-length parameters.
- For approximate constructions, test the declared guarantee and assumptions;
  exact diagram equality is generally the wrong acceptance condition.
- Measure preprocessing, computation, result conversion, and peak memory
  separately; retain difficult inputs, failures, and regressions.
- Preserve a mathematically independent small oracle. Differential agreement
  alone cannot rule out a shared implementation error.

A generic source organization is not a performance result, and the presence of a
module is not a verified correctness guarantee.

[g-modules]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/cmake/modules/GUDHI_modules.cmake
[g-package]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/cmake/modules/GUDHI_user_version_target.cmake
[g-tree]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Simplex_tree/include/gudhi/Simplex_tree.h
[g-rips]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Rips_complex/include/gudhi/Rips_complex.h
[g-sparse]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Rips_complex/include/gudhi/Sparse_rips_complex.h
[g-rips-contract]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Rips_complex/concept/SimplicialComplexForRips.h
[g-alpha]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Alpha_complex/include/gudhi/Alpha_complex.h
[g-alpha-doc]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Alpha_complex/doc/Intro_alpha_complex.h
[g-cech]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Cech_complex/doc/Intro_cech_complex.h
[g-witness]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Witness_complex/include/gudhi/Witness_complex.h
[g-cubical]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Bitmap_cubical_complex/include/gudhi/Bitmap_cubical_complex.h
[g-cubical-doc]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Bitmap_cubical_complex/doc/Gudhi_Cubical_Complex_doc.h
[g-cubical-example]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Bitmap_cubical_complex/example/Random_bitmap_cubical_complex.cpp
[g-hasse]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Hasse_complex/include/gudhi/Hasse_complex.h
[g-blocker]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Skeleton_blocker/include/gudhi/Skeleton_blocker.h
[g-toplex]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Toplex_map/doc/Intro_Toplex_map.h
[g-contract]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Persistent_cohomology/concept/FilteredComplex.h
[g-cam]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Persistent_cohomology/include/gudhi/Persistent_cohomology.h
[g-line]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Persistent_cohomology/include/gudhi/Persistence_on_a_line.h
[g-rectangle]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Persistent_cohomology/include/gudhi/Persistence_on_rectangle.h
[g-matrix]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Persistence_matrix/include/gudhi/Matrix.h
[g-adapter]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Persistence_matrix/example/example_simplex_tree_to_matrix.cpp
[g-zigzag]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Zigzag_persistence/include/gudhi/zigzag_persistence.h
[g-ripser]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Ripser/include/gudhi/ripser.h
[g-collapse]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Collapse/doc/intro_edge_collapse.h
[g-contraction]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Contraction/include/gudhi/Edge_contraction.h
[g-search]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Spatial_searching/doc/Intro_spatial_searching.h
[g-sampling]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Subsampling/doc/Intro_subsampling.h
[g-gic]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Nerve_GIC/include/gudhi/GIC.h
[g-gic-doc]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Nerve_GIC/doc/Intro_graph_induced_complex.h
[g-tangential]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Tangential_complex/include/gudhi/Tangential_complex.h
[g-tangential-doc]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Tangential_complex/doc/Intro_tangential_complex.h
[g-coxeter]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Coxeter_triangulation/doc/intro_coxeter_triangulation.h
[g-bottleneck]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Bottleneck_distance/include/gudhi/Bottleneck.h
[g-representations]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Persistence_representations/doc/Persistence_representations_doc.h
[g-sw]: https://github.com/GUDHI/gudhi-devel/blob/cba915e3ab8e1f5b1fe26eb44b407285f7af4e78/src/Persistence_representations/include/gudhi/Sliced_Wasserstein.h
