//! Cocycle is a general-purpose topological data analysis library in pure Rust.
//!
//! Computes Vietoris-Rips persistent homology in dimensions zero and one over F2
//! and descriptors of the resulting persistence diagrams. Algorithms are
//! implemented in Rust without a foreign TDA backend or runtime dependencies.
//!
//! # Status
//!
//! Version 0.1 computes H1 by implicit Rips persistent cohomology; H0-only uses
//! union-find. The explicit boundary reducer is retained as a test oracle.
//! Work and memory can still grow substantially with input size and reduction
//! fill-in. See the mathematical specification and benchmark records for limits.
//!
//! ```
//! use cocycle::descriptors::betti_curve;
//! use cocycle::geometry::PointCloudView;
//! use cocycle::persistence::{RipsOptions, rips_from_points};
//!
//! let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
//! let points = PointCloudView::new(&coordinates, 4, 2)?;
//! let diagram = rips_from_points(points, &RipsOptions::default())?;
//! assert_eq!(betti_curve(&diagram, 1, &[0., 1., 2.])?, [0, 1, 0]);
//! # Ok::<(), cocycle::Error>(())
//! ```

pub mod descriptors;
pub mod diagram;
mod error;
mod filtration;
pub mod geometry;
pub mod persistence;

pub use error::{Error, Result};

/// Canonicalize signed zero without changing any other finite value.
pub(crate) fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
