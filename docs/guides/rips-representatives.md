# Prime fields and requested representatives

[Documentation](../README.md) / Guides

Exact Rips and supplied flag computations support any prime `u32` characteristic.
The default remains F2. The legacy `RipsOptions` functions retain their original
F2 H0/H1 contract; use `PersistenceOptions` with the richer `compute_*` functions
for field selection. See [construction](rips-construction.md) for input layouts,
explicit expansion and coverage.

## Select a coefficient field

```rust
use cocycle::algebra::PrimeField;
use cocycle::persistence::PersistenceOptions;

let field = PrimeField::new(5)?;
let options = PersistenceOptions::new(2, Some(1.5))?.with_field(field);
assert_eq!(options.field().characteristic(), 5);
assert_eq!(field.multiply(4, 4), 1);
assert_eq!(field.inverse(2)?, 3);
assert!(PrimeField::new(9).is_err());
# Ok::<(), cocycle::Error>(())
```

Characteristics zero, one and composites are rejected. Modular arithmetic accepts
`u32` residues and returns canonical values; multiplication uses `u64`
intermediates. Inversion rejects a residue equal to zero. Prime validation uses
exact trial division, so large primes require more setup work. No field is
silently replaced or narrowed to another characteristic.

Changing the field can change topology: torsion may be visible over one field
and disappear over another. The test suite includes a flag subdivision of the
real projective plane, whose positive-dimensional F2 homology differs from its
odd-prime homology. Field selection is a mathematical input, not a speed switch.

## Request a basis at a scale

Each richer computation has an opt-in `_with_representatives` counterpart taking
`(input, options, requests, limits)`. `RepresentativeRequest` specifies a dimension,
finite nonnegative scale and `Cycles`, `Cocycles` or `Both`. One representative of
each selected kind is returned for every interval active at that query.

```rust
use cocycle::algebra::PrimeField;
use cocycle::diagram::RepresentativeKind;
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout};
use cocycle::persistence::{
    ExecutionLimits, PersistenceOptions, RepresentativeRequest,
    RepresentativeSelection, compute_rips_from_distances_with_representatives,
};

// A four-cycle at scale 1; diagonal edges fill it at scale 2.
let values = [1., 2., 1., 1., 2., 1.];
let matrix = DissimilarityMatrixView::new(&values, 4, MatrixLayout::LowerTriangle)?;
let options = PersistenceOptions::new(1, None)?.with_field(PrimeField::new(3)?);
let requests = [RepresentativeRequest::new(1, 1., RepresentativeSelection::Both)?];
let result = compute_rips_from_distances_with_representatives(
    matrix, &options, &requests, &ExecutionLimits::default(),
)?;
let representatives = result.representatives().unwrap();
assert_eq!(representatives.len(), 2);
assert_eq!(representatives[0].kind(), RepresentativeKind::Cycle);
assert_eq!(representatives[1].kind(), RepresentativeKind::Cocycle);
assert_eq!(representatives[0].interval_index(), representatives[1].interval_index());
assert_eq!(result.context().characteristic(), 3);
# Ok::<(), cocycle::Error>(())
```

The same request mechanism is available for points, threshold Rips, supplied flags
and expanded Rips. Custom distances feed a threshold graph as before. The explicit
path keeps its construction-dimension sufficiency check: requesting a representative
does not make an insufficient skeleton valid.

Queries are evaluated after all simplices at the scale have entered. A finite
death is excluded; a censored cutoff is included. A query beyond `Through(T)`
fails with `QueryOutsideCoverage`. With complete coverage, later finite scales
are valid. A query above the computed homology dimension fails with
`DimensionNotComputed`. Invalid requests never return an apparently empty success.

## Interpret the owned payload

`PersistenceResult::representatives()` returns `None` for ordinary calls or an
empty request slice. Nonempty requests return `Some`, possibly containing an
empty slice when no intervals are active. The result owns all terms and can
outlive its source buffers and expanded complex.

Each representative records:

- The request's position, kind, scale, dimension and field characteristic.
- `interval_index`, a position in this result's sorted diagram. Repeated intervals
  have distinct indices. These IDs are local to the result, not cross-run identifiers.
- Nonzero canonical coefficients and increasing original vertex indices, with
  terms sorted lexicographically. A coefficient of 2 over F3 represents minus one.

Cycles form a basis modulo boundaries at the query scale. For a finite interval,
the selected cycle is born at that interval's birth and becomes a boundary at its
death. In H0, such a finite-interval cycle is generally a difference of vertices,
not a single point. Cocycles form the dual basis at the query scale: their
pairing with the corresponding cycle is one, and with the other active cycles
is zero. This association holds even for repeated intervals.

There is no shortest-support or canonical-across-algorithms claim. At different
scales, cocycles are solved independently; identical coefficient vectors across
scales are not promised. The mathematical construction and independent checks
are specified in [mathematics](../reference/mathematics.md#14-persistent-representative-bases).

## Computation and resource costs

Diagram-only F2 H0/H1 retains its optimized path. Higher dimensions and odd primes
use implicit oriented coboundary reduction with clearing. Empty representative
requests retain the same path and allocate no representative skeleton.

Nonempty requests explicitly opt into materializing the required skeleton through
q+1 and retaining forward boundary transformations. Finite-interval cycles come
from reduced death columns; unpaired cycles come from birth transformation
columns. Cocycles additionally require sparse constraint elimination at each
requested scale. This can be substantially more expensive than computing only a
diagram, including when a particular request has no active classes. An explicit
input already owns topology, but representative work adds its own ordered
workspace and coefficients. No large-input performance parity is claimed.

The cooperative execution budget covers representative skeleton enumeration,
reduction, dual solves and output construction. Allocation and sorting phases
remain internally uninterruptible; no hard memory/RSS bound is provided. Failures
return no partial result. Standalone graph construction and `expand` remain
outside persistence execution controls.

Run `cargo run --locked --example rips_representatives`. Native correctness checks
compare diagrams against pinned GUDHI and Ripser C++ with matching fields; they
do not compare representative vectors with an upstream output feature that is
absent from these adapters. Independent Rust checks verify closure, nontriviality,
rank, dual pairings and death association. See [testing](../development/testing.md).
