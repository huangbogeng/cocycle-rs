//! Vietoris-Rips persistence and its computation options.
//!
//! Rips computation targets ordinary homology over F2, with an
//! edge-length filtration and a closed cutoff. H1 uses implicit persistent
//! cohomology; H0-only requests use an independent union-find path.

use crate::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use crate::geometry::{DissimilarityView, PointCloudView, euclidean_distances};
use crate::{Error, Result};

mod h0;
mod options;
#[cfg(test)]
mod reference;
mod rips_cohomology;

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
/// Returns an error if a fallible allocation fails or an internal invariant is
/// violated. It never returns a partial diagram on failure.
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
    if options.max_dimension() == 0 {
        let (cutoff, coverage) = range(input, options);
        finish(0, coverage, h0::compute(input, cutoff)?)
    } else {
        let (cutoff, coverage) = range(input, options);
        finish(1, coverage, rips_cohomology::compute(input, cutoff)?)
    }
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

fn range(input: DissimilarityView<'_>, options: &RipsOptions) -> (f64, Coverage) {
    match options.max_edge() {
        Some(cutoff) if cutoff < input.diameter() => (cutoff, Coverage::Through(cutoff)),
        _ => (input.diameter(), Coverage::Complete),
    }
}

/// Shared by algorithms; None means unpaired in the computed range, not necessarily essential.
fn finish(
    max_dimension: usize,
    coverage: Coverage,
    raw: impl IntoIterator<Item = (usize, f64, Option<f64>)>,
) -> Result<PersistenceDiagram> {
    let mut intervals = Vec::new();
    for (dimension, birth, death) in raw {
        if dimension > max_dimension || death == Some(birth) {
            continue;
        }
        let end = match death {
            Some(value) => IntervalEnd::Finite(value),
            None => match coverage {
                Coverage::Complete => IntervalEnd::Essential,
                Coverage::Through(through) => IntervalEnd::RightCensored { through },
            },
        };
        let interval = PersistenceInterval::new(dimension, birth, end)?;
        intervals
            .try_reserve(1)
            .map_err(|_| Error::AllocationFailed {
                context: "persistence diagram",
            })?;
        intervals.push(interval);
    }
    PersistenceDiagram::new(max_dimension, coverage, intervals)
}

#[cfg(test)]
mod tests;
