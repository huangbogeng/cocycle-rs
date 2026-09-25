# Contributing an algorithm

[Documentation](../README.md) / Development

Start with the mathematical object your algorithm consumes and produces. You do
not need to learn the whole kernel before making a contribution. The table below
maps research tasks to existing code; it does not imply that every named family
is implemented.

## Choose a contribution path

| Research task | Input and output | Start here |
| --- | --- | --- |
| Geometry and distances | Coordinates or distances to geometric quantities | [Geometry](../../src/geometry/mod.rs); [geometry tests](../../tests/euclidean.rs) |
| Complex and filtration construction | Data to simplices and filtration values, or a specialized cell representation | [Construction tutorial](complex-construction.md); [supplied complex guide](../guides/filtered-complexes.md) |
| Persistent homology | A filtered source to intervals and optional representatives | [Persistence](../../src/persistence/mod.rs); [reference reducer](../../src/persistence/reference/mod.rs); [algebra](../../src/algebra/mod.rs) |
| Diagram descriptors | Diagrams to statistics, curves and features | [Diagram analysis tutorial](diagram-analysis.md); [descriptors](../../src/descriptors/mod.rs) |
| Diagram distances | Two complete diagrams to a matching distance | [Matching contribution path](diagram-analysis.md#contribute-diagram-distances); [distance kernels](../../src/diagram_distances/mod.rs) |
| Analysis examples | Compose existing operations into a reproducible calculation | [Examples](../../examples/); [library guides](../README.md#use-the-library) |

Diagram descriptors, diagram distances and explicit simplicial construction have
contributor paths and focused check commands today. A reduction contributor
walkthrough is later work. Alpha and cubical constructors remain unimplemented;
this navigation does not imply that every research family is available.

## Agree on the mathematics first

Before implementation, write down:

1. The input object, assumptions, units and parameters.
2. The output definition and guarantees, including exact or approximate behavior.
3. Empty inputs, ties, degeneracies and unsupported cases.
4. Numerical precision and overflow behavior, when arithmetic matters.
5. Complexity and at least one hand-derived example or independent property.

Use the relevant section of the [mathematical specification](../reference/mathematics.md)
for shared conventions. A short algorithm needs a short explanation, not a new
design document. Document the specific callable contract beside its Rust function.

## Divide the work explicitly

| Algorithm author | Kernel maintainer |
| --- | --- |
| Definition, hypotheses and algorithm steps | Module placement and public API integration |
| Mathematical implementation and independent expected results | Reusable storage, error types and execution integration |
| Degeneracies, precision requirements and approximation guarantees | Rust ownership, allocation handling and performance integration |
| Complexity and meaningful test cases | CI, packaging, compatibility and relevant external comparisons |

This is a collaboration boundary, not a permission gate. Contributors may handle
both columns. Authors can submit an algorithm and its tests before all integration
work is finished; maintainers complete the applicable verification before merging.
Numerical choices remain a shared discussion because they can change the result.
Cancellation inside a long loop may require explicit cooperation from the algorithm;
an outer wrapper cannot interrupt arbitrary computation.

## Use existing representations first

- A descriptor usually borrows `PersistenceDiagram` and returns an owned result.
  It does not need point clouds, Rips builders or a new trait.
- An explicit simplicial constructor can produce `Simplex` values and pass a
  face-closed collection to `SimplicialComplex::new`. Every face must be supplied;
  the constructor validates and sorts but does not infer missing faces.
- A specialized representation can implement `FilteredComplex` when needed.
  Its cell boundary contract is more general than vertex-labelled simplices;
  this does not require cubical cells to be triangulated.
- Reduction contributors work with internal field/column machinery and reference
  tests. Internal modules are not stable public extension points.

Keep a small algorithm in one named file. Use a directory when several substantial
parts need their own names. Do not add a builder, registration system, options
hierarchy or shared trait solely to accommodate a hypothetical future algorithm.
Public paths follow the current compatibility policy; these guides do not promise
that internal Rust types are stable.

## Make correctness reviewable

Construct the smallest input that exposes the mathematics. Diagram tests can
start from intervals; construction tests inspect cells and filtration values;
reduction tests can start from a small boundary matrix. End-to-end tests then
check composition without replacing these local expectations.

Reuse fixture construction only when it removes actual repetition. Do not use
the production algorithm to calculate its expected answer or hide endpoint
conventions inside a fixture helper. Existing [descriptor tests](../../tests/descriptors.rs)
are a compact example of hand calculations, numerical limits and invariance.

For local checks, choose your [focused command](../../CONTRIBUTING.md#focused-algorithm-checks).
Maintainers and CI also run the applicable
[full verification](../../CONTRIBUTING.md#verification). Explain the algorithm,
resulting behavior and checks actually run in the PR; mark incomplete integration
work explicitly. Generated outputs remain outside Git.
