//! Vietoris-Rips persistence and its computation options.
//!
//! Rips computation targets ordinary homology over F2, with an
//! edge-length filtration and a closed cutoff. H1 uses implicit persistent
//! cohomology; H0-only requests use an independent union-find path.

use super::assemble_diagram;
use crate::Result;
use crate::diagram::{Coverage, PersistenceDiagram};
use crate::geometry::{DissimilarityView, PointCloudView, euclidean_distances};

mod cohomology;
mod h0;
mod options;
mod union_find;

pub use options::RipsOptions;

/// Compute ordinary Rips persistence over F2 from a symmetric dissimilarity matrix.
///
/// The cutoff is inclusive and measured in edge lengths. Zero-length intervals
/// are omitted. If the cutoff is below the input diameter, surviving classes are
/// right-censored; otherwise coverage is complete. No triangle inequality is
/// required. H1 uses the 2-skeleton, including triangles that kill cycles.
///
/// H1 stores edges and change-of-basis columns, generating triangle cofacets
/// on demand. It avoids materializing the full 2-skeleton, but reduction fill-in
/// and repeated cofacet enumeration can still be large. No automatic sampling,
/// approximation or process memory bound is applied. An internal cone bound may
/// stop computation early without changing the public coverage convention.
///
/// # Errors
/// Returns an error if simplex indexing exceeds the supported integer range,
/// a fallible allocation fails, or an internal invariant is violated. It never
/// returns a partial diagram on failure.
///
/// ```
/// use cocycle::geometry::DissimilarityView;
/// use cocycle::persistence::{RipsOptions, rips_from_dissimilarities};
/// let input = DissimilarityView::new(&[2.0], 2)?;
/// let diagram = rips_from_dissimilarities(input, &RipsOptions::default())?;
/// assert_eq!(diagram.intervals_in_dimension(0)?.count(), 2);
/// # Ok::<(), cocycle::Error>(())
/// ```
pub fn rips_from_dissimilarities(
    input: DissimilarityView<'_>,
    options: &RipsOptions,
) -> Result<PersistenceDiagram> {
    let (cutoff, coverage) = resolve_rips_range(input, options);
    let raw = if options.max_dimension() == 0 {
        h0::compute(input, cutoff)?
    } else {
        cohomology::compute(input, cutoff)?
    };
    assemble_diagram(options.max_dimension(), coverage, raw)
}

/// Compute ordinary Rips persistence over F2 from a Euclidean point cloud.
///
/// Computes one condensed distance buffer, then follows
/// [`rips_from_dissimilarities`]. Coordinates are not normalized or deduplicated.
/// Euclidean norms use scaled `hypot` operations to avoid unnecessary square-sum
/// overflow and underflow. The returned diagram does not borrow the point cloud.
///
/// # Errors
/// In addition to persistence errors, returns an error if distance storage cannot
/// be reserved or a Euclidean distance is not representable as a finite `f64`.
pub fn rips_from_points(
    input: PointCloudView<'_>,
    options: &RipsOptions,
) -> Result<PersistenceDiagram> {
    let distances = euclidean_distances(input)?;
    rips_from_dissimilarities(DissimilarityView::new(&distances, input.len())?, options)
}

pub(super) fn resolve_rips_range(
    input: DissimilarityView<'_>,
    options: &RipsOptions,
) -> (f64, Coverage) {
    match options.max_edge() {
        Some(cutoff) if cutoff < input.diameter() => (cutoff, Coverage::Through(cutoff)),
        _ => (input.diameter(), Coverage::Complete),
    }
}
