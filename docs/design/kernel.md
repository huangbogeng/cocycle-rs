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
current behavior. The [roadmap](roadmap.md) selects the next work. The
[complete Rips subsystem design](rips.md) specializes this direction into the
selected Rips capability matrix, API draft and acceptance gates.

## Starting point

The [Rips guide](../guides/rips.md) describes the implemented input, persistence,
diagram, and descriptor contracts. This design builds on those contracts.

Important existing limits:

- The legacy point API and unrestricted richer point calls materialize a dense
  condensed buffer; finite-cutoff richer calls stream a threshold graph.
- The specialized H1 path uses compact edge/triangle indexing. Higher-dimensional
  and odd-prime requests use tuple-indexed implicit cohomology and clearing; apparent/emergent
  pair shortcuts remain specialized to H1.
- Legacy `RipsOptions` admits H0/H1; `PersistenceOptions` accepts arbitrary
  dimensions. Frozen explicit simplicial expansion is available. There is no
  public general boundary-reducer API or cubical computation. Prime fields are
  validated explicitly; a private forward reducer serves requested representatives.
- Richer results now add exact-Rips/supplied-flag context and coefficient field.
  Representatives now carry original simplex vertices, coefficients and interval
  identities. Approximation metadata remains unimplemented.
- Intervals permit signed scales; the existing Betti-curve grid deliberately
  permits only nonnegative values. Scalar-field support must address that mismatch
  explicitly.
- Cooperative work limits and cancellation exist for persistence. Construction
  is not covered by those controls; no batch scheduler or hard RSS limit exists.
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

Richer Rips calls now return owned computation context; see the
[construction guide](../guides/rips-construction.md) and
[approximation guide](../guides/sparse-rips.md). Preserve this separation for
future filtrations. Their ordinary-persistence records should be able to state:

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

Matrix layouts, frozen simplicial storage, prime-field algebra, owned computation
context and cooperative execution controls are already implemented. Their current
locations belong in the architecture page. The remaining map identifies future
ownership; create files only with concrete consumers, not one file per class.

| Concrete addition | Likely home | Boundary to preserve |
| --- | --- | --- |
| Additional input layouts or separate spatial operations | Named files within the existing `geometry/` directory | Re-export existing types; internal storage stays private |
| Scalar-line persistence | `persistence/line.rs` initially | Borrow samples; keep its ordering and workspace out of Rips |
| Cubical topology and values | `complex/cubical/` | Grid incidence and validated data are distinct from reduction state |
| Cubical filtration access, when needed separately | `filtration/cubical.rs` | Adapt the grid; do not duplicate the grid or its incidence implementation |
| Cubical persistence | `persistence/cubical.rs` or a directory when needed | Select or implement a reducer without routing through simplex storage |
| General ordered boundary access for another consumer | Extend existing `algebra/reduction/` with a demonstrated access contract | Preserve independence from any concrete complex |
| Landscapes or persistence images | Add named operation files to `descriptors/` | Consume diagrams only; share validated feature configurations when semantics agree |
| Diagram matching distances | `diagram_distances/{mod,bottleneck}.rs` initially | Keep diagram matching separate from geometric point distances |
| Context for another filtration family | Extend `diagram` or use an operation-owned result wrapper | Preserve engine-independent mathematical interpretation |
| Hard resource limits or reusable workspaces beyond current cooperative controls | Feature-local first; `execution/` only for actual shared policy | Controls do not alter mathematical input or silently change the algorithm's guarantee |

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
| Union-find for current Rips H0/H1 | Shared within `persistence::flag`; preserve allocation/error semantics |
| Flag ordering and cofacet generation | Owned by `filtration::flag`; algorithm workspaces do not define a second order |
| Combinatorial arithmetic | Extract only if callers agree on domains, overflow behavior, and index convention; similar formulas alone are insufficient |
| Sparse columns and field arithmetic | Share across production algorithms only when the algebraic/storage contract and measured cost agree |
| Small allocation/error helpers | Keep local until genuinely shared; do not create a catch-all `utils` module |
| Explicit reference reducer | Keep mathematically independent from production; sharing output formatting does not justify sharing the algorithm under test |

Two consumers are evidence to investigate reuse, not proof that a common trait
is appropriate. In particular, an H0 algorithm with nonzero vertex births may
need an elder-rule representative separate from the union-find root; sharing
connectivity mechanics must not silently share an invalid pairing policy.

## Capability sequence and acceptance gates

The selected first workstream is the [complete Rips subsystem](rips.md), including
inputs, explicit and implicit construction/access, higher-dimensional computation,
prime fields, representatives and sparse approximation. Its
[delivery sequence](rips.md#delivery-sequence-and-exit-gates) owns the detailed
order and gates. The broader capability groups below describe subsequent or
shared work, not a competing instruction to prioritize scalar-line analysis.

| Capability group | Deliverable | Gate before claiming support |
| --- | --- | --- |
| Complete Rips, selected first | All R1-R10 capabilities in the dedicated design | Every Rips delivery gate, independently validated and documented; H0/H1 construction alone is insufficient |
| Analysis closure | Better diagram-to-feature composition, a scalar-line capability, runnable multi-sample examples | Hand-computable outputs, negative/tied scalar cases, coverage-aware queries, explicit feature settings |
| Additional data domains | A specified cubical input/construction and computation | Independently assembled tiny-grid boundaries, scalar conventions and resource evidence |
| Algebra reuse beyond Rips | Adapt established field/reduction components to another actual consumer | Verify the consumer's boundary, ordering and result contracts independently |
| Geometry and simplification | Sampling/collapse outside the Rips target; Alpha when geometry infrastructure is ready | Preservation tests with hypotheses, robust degeneracy cases and native dependency review |
| Specialized expansion | Zigzag, cover complexes, or reconstruction selected by real demand | Own mathematical/output contract and complete end-to-end example |

Deliver complete operations within the Rips workstream rather than creating all
proposed modules simultaneously. Finish each operation's contract, independent
validation, API, documentation and example without dropping the remaining Rips
completion requirements. Future library groups do not weaken that target.

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
