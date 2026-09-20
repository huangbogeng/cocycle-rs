# Rust TDA kernel design direction

[Documentation](../README.md) / Design

Status: design direction, recorded 2026-09-20. The current source boundaries are
documented in the [architecture](../development/architecture.md). Additional
capabilities and API proposals here are not supported features or a delivery schedule.

## Objective and scope

Build a native Rust topological data analysis kernel, using GUDHI's C++ capability
set and Ripser's specialized computation as reference points. The primary product
is a Rust library with runnable analysis examples. Bindings, a CLI, hosted
services, and a dataframe/query engine are not first-stage deliverables.

"Modern data analysis" means observable data contracts, predictable ownership,
composable operations, clear resource behavior, and reproducible results.
Mathematical definitions and proofs remain essential; usability must expose their
assumptions rather than obscure them.

The [GUDHI study](../research/gudhi-cpp.md) records source observations. The
[architecture](../development/architecture.md) and [guide](../guides/rips.md) remain authoritative for
current behavior. The [roadmap](roadmap.md) selects the next work.

## Starting point

The [Rips guide](../guides/rips.md) describes the implemented input, persistence,
diagram, and descriptor contracts. This design builds on those contracts.

Important existing limits:

- Point-cloud computations materialize a dense condensed distance buffer.
- Filtration indexing and H1 cofacet enumeration specialize in edges/triangles.
- Rips options admit H0/H1 only. There is no production general boundary reducer,
  public explicit complex, sparse graph input, cubical computation, or prime-field
  API.
- Diagrams record dimensions and coverage, but not a complete computation
  provenance record such as filtration convention or coefficient field.
- Intervals permit signed scales; the existing Betti-curve grid deliberately
  permits only nonnegative values. Scalar-field support must address that mismatch
  explicitly.
- No batch scheduler, cancellation contract, or hard process memory limit exists.
- The current source is not a workspace of separate kernel/adapter crates.

These limits must remain visible while the project grows.

## Design decisions

### Data-oriented entry points with explicit mathematical meaning

Public operations should begin with the data that callers have: points,
dissimilarities, a filtered graph, scalar samples, or a grid. Users should be
able to compute a result without first assembling internal containers.

Operation-specific options specify mathematical intent: homology dimensions,
coefficient field, scale cutoff, filtration convention, or approximation
parameters. Resource controls are a separate concern.

Point normalization, deduplication, sampling, and time-delay embedding are
explicit transformations. A point-cloud call must not silently preprocess data
or select a cheaper mathematical problem.

"Distance" must not silently imply the triangle inequality: existing symmetric
dissimilarities deliberately need not be metrics. Algorithms requiring a metric
must state how that precondition is established; checking every triangle is not
a free default.

### Separate input, representation, computation, and result

The logical dependency direction is:

```text
validated input views
    -> operation-specific construction / access
    -> algorithm with private working state
    -> owned mathematical result
    -> descriptors, representations, and distances

explicit complex builders -> frozen incidence/filtration access -> reducer

execution controls and diagnostics accompany a computation;
they do not define the topology of its input.
```

A result-only operation cannot depend on an input point cloud, simplex container,
or reducer state. Geometry must not depend on persistence. A shared algebra
component must not import a Rips-specific filtration.

Builders may be mutable; a computation should borrow a stable read-only input
or snapshot. Cache preparation can occur before freezing. Pivot ownership,
union-find, annotations, and change-of-basis columns belong to a computation or
explicit reusable workspace, not hidden fields in the caller's complex.

A workspace may retain capacity between calls, but logical state must be reset.
It must not retain invalid input references or leak results from the previous
sample.

### Multiple representations and algorithms

Retain distinct execution paths where the problem warrants them:

| Path | Suitable representation | Reusable boundary |
| --- | --- | --- |
| Rips persistence | Dense distances or a declared filtered graph; implicit simplices/cofacets | Validated input and ordinary diagram semantics |
| Scalar function on a line | Borrowed scalar samples and dedicated scan | Ordinary diagram semantics |
| Cubical persistence | Grid shape/values and arithmetic incidence | Boundary access where useful; specialized algorithms remain possible |
| Explicit simplicial persistence | Stored simplices with validated filtration | Ordered boundary columns and algebra |
| Zigzag, later | Validated insertion/deletion events | Selected matrix operations; distinct interval semantics |
| Cover/nerve analysis, later | Covers, memberships, and graph structure | Graph/complex results, not forced into persistence diagrams |

A Simplex tree implementation is optional. The first explicit simplicial
representation can be simpler if it meets actual construction, lookup, and
traversal needs. Do not materialize a Rips complex solely to share that storage.

Introduce narrow internal traits when a concrete second consumer makes their
requirements clear. Potential contracts include distance access, ordered
boundary-column access, and cofacet generation. They are not one universal
`Complex` trait. An implicit enumerator need not expose a cheap total cell count,
mutable insertion, or random access to every simplex.

Use private newtypes where mixing index spaces is a realistic error, for example
a filtration position versus a combinatorial simplex ID. Do not expose a
representation's integer IDs as a universal public cell identity.

### Results carry enough information to be interpreted

Preserve owned diagrams and existing endpoint distinctions. Introduce computation
context alongside results when needed, without turning the basic interval
container into an engine-specific object.

A future ordinary-persistence computation record should be able to state:

- Computed dimensions and coefficient field.
- Filtration family and scale convention, including squared versus unsquared
  quantities and any explicit preprocessing.
- Coverage and endpoint convention.
- Whether the computation is exact for the supplied filtration or has a declared
  approximation guarantee, including its hypotheses and parameters.
- Optional diagnostics and reproducibility information, such as sampling seed
  and implementation version.

Mathematical context is distinct from optional telemetry: an algorithm name is
useful provenance, but a diagram's validity must not depend on one engine name.

Finite scalar-field levels can be negative. Rips retains its own nonnegative
distance contract. Numerical policies for NaN, infinity, masks, overflow, and
signed zero belong to each input/result domain; do not use a finite-distance
validator for every scalar field.

For future masked grids, model exclusion explicitly or document a precise
extended-value convention. Do not accidentally reinterpret a missing sample as
a large finite value.

Representatives are optional outputs with explicit ownership and input mappings.
A representative cycle, a cocycle, and a persistence pair are different objects.
Computing representatives must not become an unavoidable cost for diagram-only
callers. When preprocessing changes the complex, explain how witnesses map back.

Ordinary persistence, extended persistence, and zigzag may need different interval
types. A shared output philosophy does not justify flattening incompatible
mathematical semantics.

### Failure, coverage, and approximation are independent

A caller cutoff defines a restricted mathematical computation. Cancellation or
an exhausted resource budget defines an unsuccessful execution. They cannot
share a generic "incomplete" flag.

Initially, interrupted computations should return a typed error and no public
diagram, consistent with current failure behavior. Certified partial results
would require a separate explicit contract and proof of the completed range;
arbitrary intermediate pairs do not constitute a valid censored diagram.

Similarly, keep these operations distinct:

| Operation | Meaning |
| --- | --- |
| Exact threshold graph | All required edges through a declared scale are available |
| Arbitrary sparse weighted graph | Its own flag filtration; no implied equivalence to the original dense Rips filtration |
| Sparse Rips approximation | A changed filtration with a stated approximation theorem and applicable assumptions |
| Subsampling | A changed dataset with selected indices and parameters |
| Persistence-preserving collapse | A transformation with a specific preservation guarantee |

Do not infer complete coverage of the original dense filtration from the largest
stored edge in a sparse input. Edge-list validation alone cannot certify that
all edges below a cutoff were supplied; the API must distinguish a supplied graph
from a construction with that guarantee.

### Resource behavior and batch composition

For dense points, report or document the known distance storage cost
`n * (n - 1) / 2 * size_of::<f64>()`, with checked arithmetic. This is not the
full memory estimate: edge sorting, indices, heaps, and reduction fill-in add
costs that can dominate.

Resource controls should use quantities the implementation can actually enforce:
tracked work-buffer bytes, generated-entry counts, or cooperative cancellation
checkpoints. An internal allocation budget is not a hard bound on process RSS.
All tracked buffers and transient reallocations need a defined accounting policy
before advertising such a limit. Do not promise recovery from every OOM.

Begin batch usage with ordinary Rust iteration over the single-sample API.
A dedicated batch API becomes justified by reusable workspaces, bounded
concurrency, or per-sample error reporting in real examples.

When added, batch execution should preserve sample identity and deterministic
result ordering, specify fail-fast versus per-sample failure, bound in-flight
memory, and avoid nested uncontrolled thread pools. No global mutable runtime is
needed. Async is not a prerequisite for a CPU computation library.

Determinism claims should specify their scope. Tie policies and random seeds can
be stable while floating-point results still vary across numerical kernels or
platforms.

### Analysis representations are first-class kernel operations

Prioritize capabilities that turn diagrams into comparable measurements:

- Betti curves on a declared grid and dimension.
- Lifetime/entropy summaries with explicit treatment of non-finite lifetimes.
- Grid landscapes or another precisely specified vector representation.
- Diagram distances with a defined supported endpoint domain.

Representations evaluated for multiple samples must share grid, bandwidth,
dimension ordering, and endpoint policy. If parameter fitting is added, freeze
the fitted configuration for new data; do not silently refit each sample or use
held-out data to choose a feature grid.

For an initial finite-diagram distance implementation, reject unsupported
essential/censored intervals or require an explicitly named projection. Never
drop them silently. Numerical approximation tolerances must be documented
separately from filtration coverage.

These operations can expose ordinary slices, iterators, and owned arrays.
Arrow, Polars, ndarray, serialization, and plotting adapters are candidates for
separate optional integration, not requirements for the core API. None is
selected or implemented here.

## Extension boundaries

Keep one crate initially. A workspace becomes worthwhile when a real integration
has different dependencies, release needs, or compilation costs.

The [current architecture](../development/architecture.md) owns the implemented
source tree, dependency direction, and component responsibilities. Follow the
[file and dependency rules](../development/conventions.md#file-and-dependency-boundaries)
when adding code. The map below locates future capabilities; it does not announce
new public namespaces or require empty directories.

### Where future capabilities belong

Create the following files or directories only as their implementations arrive.
The locations identify ownership; they do not prescribe one file per class.

| Concrete addition | Likely home | Boundary to preserve |
| --- | --- | --- |
| A second input layout or separate spatial operations | Named files within the existing `geometry/` directory | Re-export existing types; internal storage stays private |
| Scalar-line persistence | `persistence/line.rs` initially | Borrow samples; keep its ordering and workspace out of Rips |
| Cubical topology and values | `complex/cubical/` | Grid incidence and validated data are distinct from reduction state |
| Cubical filtration access, when needed separately | `filtration/cubical.rs` | Adapt the grid; do not duplicate the grid or its incidence implementation |
| Cubical persistence | `persistence/cubical.rs` or a directory when needed | Select or implement a reducer without routing through simplex storage |
| Explicit simplicial storage | `complex/simplicial/` | Builders establish closure; frozen access provides documented validity |
| General ordered boundary reduction | `algebra/{boundary,reduction,column}.rs` | Algebra depends on a precise access contract, not a concrete complex |
| A production second coefficient field | `algebra/field/` | Oriented coefficients and field operations, not only integer indices |
| Landscapes or persistence images | Add named operation files to `descriptors/` | Consume diagrams only; share validated feature configurations when semantics agree |
| Diagram matching distances | `diagram_distances/{mod,bottleneck}.rs` initially | Keep diagram matching separate from geometric point distances |
| A computation record | Extend `diagram` or introduce an operation-owned result wrapper as needed | Mathematical result ownership remains independent of the engine |
| Enforceable limits or reusable workspaces | Feature-local first; `execution/` only for actual shared policy | Controls do not alter mathematical input or silently change the algorithm's guarantee |

For example, adding a landscape should not require changes to filtration or
persistence. Adding a cubical input should not alter Rips internals. Adding a new
reducer may require an adapter, but should not require rewriting every complex.
These change-impact checks are more useful than counting directories.

### Reuse policy

Reuse has different levels. Prefer shared mathematical results and data contracts
before attempting to share every algorithm's internal representation.

| Candidate | Decision and ownership |
| --- | --- |
| Validated point/dissimilarity views | Already shared through `geometry`; no validation copies in each public entry point |
| Owned intervals and coverage | Shared through `diagram`; interpretation is independent of reducer storage |
| Ordinary interval assembly | One production normalization path; retain separate contract tests for it |
| Union-find for current Rips H0/H1 | Shared within `persistence::rips`; preserve allocation/error semantics |
| Rips ordering and cofacet generation | Owned by `filtration::rips`; algorithm workspaces do not define a second order |
| Combinatorial arithmetic | Extract only if callers agree on domains, overflow behavior, and index convention; similar formulas alone are insufficient |
| Sparse columns and field arithmetic | Share across production algorithms only when the algebraic/storage contract and measured cost agree |
| Small allocation/error helpers | Keep local until genuinely shared; do not create a catch-all `utils` module |
| Explicit reference reducer | Keep mathematically independent from production; sharing output formatting does not justify sharing the algorithm under test |

Two consumers are evidence to investigate reuse, not proof that a common trait
is appropriate. In particular, an H0 algorithm with nonzero vertex births may
need an elder-rule representative separate from the union-find root; sharing
connectivity mechanics must not silently share an invalid pairing policy.

## Capability sequence and acceptance gates

This is a proposed dependency sequence, not equal priority for every GUDHI module.

| Stage | Deliverable | Gate before claiming support |
| --- | --- | --- |
| 0. Rips foundation | Clear private boundaries, stable public semantics, reproducible baseline | Existing independent tests and fresh native C++ comparison on pinned configurations; no unexplained regression |
| 1. Analysis closure | Better diagram-to-feature composition, a scalar-line capability, runnable multi-sample examples | Hand-computable outputs, negative/tied scalar cases, coverage-aware queries, explicit feature settings |
| 2. Data scale and breadth | Exact filtered-graph input, then a specified cubical input/construction | Dense-versus-sparse parity through cutoff; independently assembled tiny-grid boundaries; memory evidence |
| 3. Broader algebra | Higher-dimensional Rips, an explicit simplicial path, then prime fields and requested witnesses | Include cells that kill top requested homology; verify boundary squared is zero, field behavior, and witness equations |
| 4. Controlled approximation and geometry | Selected sampling/sparse approximation/collapse; geometry such as Alpha when infrastructure is ready | Preservation or approximation tests with hypotheses; robust geometric degeneracy cases and native dependency review |
| 5. Specialized expansion | Zigzag, cover complexes, or reconstruction selected by real demand | Own mathematical/output contract and complete end-to-end example |

Stage 1 is a proposal for the next useful capability set, not an instruction to
implement every listed feature simultaneously. Choose one operation and finish
its contract, oracle, API, documentation, and example before the next.

Prime fields require oriented boundary coefficients and modular arithmetic;
replacing an F2 XOR operation alone is not sufficient. Generalized Rips requires
safe combinatorial indexing, dimension-by-dimension algorithms, and new cost
evidence; increasing an option's accepted dimension is not implementation.

Pure Rust Alpha/weighted geometry must account for predicates, triangulation,
degeneracy handling, and filtration accuracy. Scope a first geometry capability
by dimension and supported inputs. Do not promise parity with CGAL's full
geometry stack as an incidental part of a TDA refactor.

## Validation and project development

Examples should be small, deterministic, runnable with Cargo, and explicit about
parameters and result interpretation. Suitable acceptance scenarios include:

- A point-cloud loop with a cutoff, feature curve, and visible censoring.
- A scalar signal with known extrema, including ties and negative values.
- A small image/grid whose connected components and hole can be calculated by hand.
- Several samples converted using the same feature grid, retaining sample IDs.

These are kernel demonstrations, not a new application framework.

The [validation strategy](../development/testing.md) describes current oracles
and regression coverage. For each new capability, require an independent
mathematical example/property, native GUDHI C++ differential checks where
applicable, adversarial cases,
documented failure behavior, and meaningful performance measurements. Separate
dataset preparation, construction, reduction, and output conversion.

A new production generic reducer needs an independent small oracle or direct
expected results. Reusing the current reference reducer as production while
still calling its output an independent check would remove that safeguard.

The project can grow through complete, reviewable operations: a small documented
API, one runnable example, independent evidence, and a compatibility statement.
Release readiness remains a gate, while module count and a full GUDHI feature
checklist are not measures of completion.
