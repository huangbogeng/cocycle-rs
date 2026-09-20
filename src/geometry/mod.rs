//! Borrowed, validated numerical inputs.
//!
//! Point clouds use row-major coordinates. Precomputed dissimilarities use a
//! condensed lower triangle: `[d(1,0), d(2,0), d(2,1), ...]`. No input is copied,
//! deduplicated or modified. Finite coordinates and nonnegative dissimilarities
//! do not, on their own, certify a metric.

mod dissimilarity;
mod euclidean;
mod point_cloud;

pub use dissimilarity::DissimilarityView;
pub(crate) use euclidean::euclidean_distances;
pub use point_cloud::PointCloudView;
