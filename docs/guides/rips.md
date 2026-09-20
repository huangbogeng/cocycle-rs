# Rips persistence guide

[Documentation](../README.md) / Guides

This introductory guide uses the compatible `RipsOptions` API for ordinary
Vietoris–Rips persistence over F2 in H0 or H0/H1. The richer APIs also support
arbitrary homology dimensions, prime fields, explicit complexes and requested
representatives; see the [construction guide](rips-construction.md) and
[field/representative guide](rips-representatives.md).
Input buffers are borrowed; returned diagrams own their data and outlive the
input. See the [mathematical specification](../reference/mathematics.md) for exact
conventions.

For a runnable workflow from points through diagrams to descriptors, use
`cargo run --locked --example square` from the repository root. The
[example source](../../examples/square.rs) demonstrates the complete filtration.

## Point clouds

`PointCloudView::new(&coordinates, n, d)` accepts row-major `f64` coordinates.
The length must be `n * d`, with `d > 0`; all coordinates must be finite. Empty
point clouds are valid. Duplicate points remain distinct vertices.

```rust
use cocycle::geometry::PointCloudView;
use cocycle::persistence::{RipsOptions, rips_from_points};

fn main() -> cocycle::Result<()> {
    let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
    let points = PointCloudView::new(&coordinates, 4, 2)?;
    let diagram = rips_from_points(points, &RipsOptions::default())?;
    Ok(())
}
```

The point-cloud entry point computes Euclidean distances once. An unrepresentable
finite distance produces an error rather than silently becoming infinity.

## Precomputed dissimilarities

`DissimilarityView::new(&values, n)` accepts the strict lower triangle in row order:
`[delta(1,0), delta(2,0), delta(2,1), ...]`. Its length is `n * (n - 1) / 2`;
the diagonal is implicitly zero and symmetry follows from the representation.
Values must be finite and nonnegative. The triangle inequality is not required.

```rust
use cocycle::geometry::DissimilarityView;
use cocycle::persistence::{RipsOptions, rips_from_dissimilarities};

fn main() -> cocycle::Result<()> {
    let distances = [2.0]; // Two vertices at distance 2.
    let input = DissimilarityView::new(&distances, 2)?;
    let diagram = rips_from_dissimilarities(input, &RipsOptions::default())?;
    assert_eq!(diagram.intervals_in_dimension(0)?.count(), 2);
    Ok(())
}
```

An explicit vertex count distinguishes zero vertices from one vertex, both of
which have an empty condensed buffer. `-0.0` is interpreted as zero.

## Options and scales

| Options | Computation |
| --- | --- |
| `RipsOptions::default()` | H0 and H1, complete filtration |
| `RipsOptions::new(0, None)?` | H0 only, complete filtration |
| `RipsOptions::new(1, Some(1.0))?` | H0 and H1 through edge length 1, inclusive |

The scale is an edge length, not a radius or squared distance. A simplex enters
when its longest edge is present. Triangles are needed to detect deaths in H1,
even though the requested output stops at dimension one. Higher output dimensions
are rejected by this legacy API. Use `PersistenceOptions` and the `compute_*`
entry points in the [construction guide](rips-construction.md) for arbitrary
homology dimensions, prime fields and explicit complexes. Approximation uses
separate [sparse Rips APIs](sparse-rips.md), with its own parameters and hypotheses.

## Reading a diagram

`diagram.intervals()` exposes deterministic, sorted intervals with multiplicity.
`intervals_in_dimension(k)` rejects dimensions that were not computed. An empty
computed dimension is different from an uncomputed dimension.

| Endpoint | Meaning |
| --- | --- |
| `IntervalEnd::Finite(d)` | Observed death; the interval is `[birth, d)` |
| `IntervalEnd::Essential` | Survives the complete filtration |
| `IntervalEnd::RightCensored { through: t }` | Alive at `t`; its full death is unknown |

Zero-length intervals are omitted. A cutoff at or above the input diameter gives
`Coverage::Complete`; otherwise coverage is `Through(t)`. In incomplete results,
all surviving classes, including H0, are conservatively marked right-censored.
A censored endpoint is not a death at the cutoff.

For a unit square, full H1 is `[1, sqrt(2))`. At cutoff 1, that class is born and
still alive, so it is right-censored. A Betti query at 1 counts it; a query beyond
the cutoff is rejected.

## Descriptors

`finite_lifetime_summary(&diagram, k)` reports finite count, total lifetime,
maximum lifetime, and entropy. It excludes essential and censored intervals and
reports both exclusion counts. Entropy uses natural logarithms (nats), without
dividing by the logarithm of the interval count. With no finite positive lifetimes,
the total is zero and the maximum and entropy are `None`.

`betti_curve(&diagram, k, &grid)` uses all intervals. Supply finite, nonnegative,
strictly increasing scales within the computed coverage. Births are included;
finite deaths are excluded; censored classes remain alive at the cutoff itself.
An empty grid is allowed. Descriptors do not rerun persistence.

## Errors and practical limits

Inputs and options return `cocycle::Result`. Invalid shapes, non-finite values,
unsupported dimensions, arithmetic overflow, and out-of-coverage queries are
errors, not empty diagrams. Computation errors do not return partial results.

The legacy point entry point above uses quadratic distance storage. Richer
finite-cutoff point calls can stream a threshold graph. The specialized F2 H1 path uses
implicit triangles but can still require substantial time and memory because of
repeated enumeration and reduction fill-in. Combinatorial triangle numbering must
fit `usize`, even for a sparse cutoff. There is no automatic process resource cap;
cooperative work limits are available on the richer compute entry points.
Rust allocation failures are not uniformly
recoverable. Read the [benchmarks](../../benches/README.md) for measured input-specific
behavior rather than a universal point-count limit.

## Graph construction and additional matrix layouts

The original entry points above remain available. The
[construction guide](rips-construction.md) covers borrowed upper/full matrices,
custom distances, exact threshold graph inspection, supplied flag filtrations,
owned computation context and cooperative execution controls.
