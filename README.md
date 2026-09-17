# Cocycle

[![CI](https://github.com/huangbogeng/cocycle-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/huangbogeng/cocycle-rs/actions/workflows/ci.yml)

A general-purpose topological data analysis library in pure Rust.

Cocycle computes Vietoris–Rips persistence and summarizes persistence diagrams.
It has no runtime dependencies and forbids unsafe Rust. **The project is in
pre-release development; version 0.1.0 has not been published.**

## What it supports

- Euclidean point clouds and precomputed symmetric dissimilarities.
- Ordinary persistent homology over F2, in dimensions H0 and H1.
- Closed edge-length cutoffs with explicit right-censoring semantics.
- Owned diagrams preserving interval multiplicity.
- Finite lifetime summaries, persistence entropy in nats, and Betti curves.

H0 uses union-find. H1 uses implicit persistent cohomology, generating triangle
cofacets on demand. The library does not require a foreign TDA backend.

Higher dimensions, representative cycles, other coefficient fields, other
filtrations, and diagram distances are not implemented. Language bindings and
application-specific research belong in separate projects.

## Quick start

Requires **Rust 1.91 or later**. Until publication, use the Git repository:

```toml
[dependencies]
cocycle = { git = "https://github.com/huangbogeng/cocycle-rs", branch = "main" }
```

Your application's `Cargo.lock` records the resolved commit. For a local checkout,
use `cocycle = { path = "../cocycle-rs" }` instead.

```rust
use cocycle::descriptors::betti_curve;
use cocycle::geometry::PointCloudView;
use cocycle::persistence::{RipsOptions, rips_from_points};

fn main() -> cocycle::Result<()> {
    let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
    let points = PointCloudView::new(&coordinates, 4, 2)?;
    let diagram = rips_from_points(points, &RipsOptions::default())?;
    assert_eq!(betti_curve(&diagram, 1, &[0., 1., 2.])?, [0, 1, 0]);
    Ok(())
}
```

The square has one H1 interval `[1, sqrt(2))`. Run the complete example with
`cargo run --locked --example square`.

A cutoff is an edge length, not a ball radius or squared distance. Classes still
alive at an incomplete cutoff are right-censored, not observed deaths. See the
[user guide](docs/guide.md) before interpreting truncated diagrams.

## Documentation

- [User guide](docs/guide.md): inputs, options, endpoints, and descriptors.
- [API documentation](src/lib.rs): build locally with `cargo doc --no-deps --open`.
- [Mathematics](docs/mathematics.md): definitions, invariants, and algorithm basis.
- [Architecture](docs/architecture.md): code boundaries and extension rules.
- [Contributing](CONTRIBUTING.md): development, review, and release requirements.
- [Documentation index](docs/README.md): testing, references, and roadmap.

## Performance

Work depends on the input geometry, cutoff, and reduction fill-in. H1 avoids
materializing the full 2-skeleton, but it is not memory-bounded or approximate.
There is no universal supported point-count limit.

The [benchmark guide](benches/README.md) describes reproducible comparisons with
GUDHI and Ripser.py, including unfavorable cases and timeouts. Measurements are
versioned evidence, not a claim that one library is always faster.

## Development

```sh
git clone https://github.com/huangbogeng/cocycle-rs.git
cd cocycle-rs
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the complete checks. Public interfaces,
source comments, and project documentation are written in English.

Report bugs and propose features in the [issue tracker](https://github.com/huangbogeng/cocycle-rs/issues).
See [CI runs](https://github.com/huangbogeng/cocycle-rs/actions/workflows/ci.yml) for
the checks on each commit.

Licensed under the [MIT License](LICENSE). Changes are recorded in the
[changelog](CHANGELOG.md).
