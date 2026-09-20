//! Validated, borrowed point-cloud coordinates.

use crate::{Error, Result};

/// A validated row-major point cloud borrowing the caller's coordinates.
///
/// Duplicate points are retained. The coordinate dimension is positive, even
/// for an empty cloud. Raw coordinates may contain negative zero.
///
/// ```
/// use cocycle::geometry::PointCloudView;
///
/// let coordinates = [0.0, 1.0, 2.0, 3.0];
/// let points = PointCloudView::new(&coordinates, 2, 2)?;
/// assert_eq!(points.point(1), Some(&[2.0, 3.0][..]));
/// # Ok::<(), cocycle::Error>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PointCloudView<'a> {
    coordinates: &'a [f64],
    point_count: usize,
    dimension: usize,
}

impl<'a> PointCloudView<'a> {
    /// Validate `point_count * dimension` row-major coordinates without copying.
    ///
    /// # Errors
    /// Returns an error for zero dimension, shape overflow, a mismatched buffer
    /// length, or any non-finite coordinate. Validation takes O(buffer length).
    pub fn new(coordinates: &'a [f64], point_count: usize, dimension: usize) -> Result<Self> {
        if dimension == 0 {
            return Err(Error::InvalidParameter {
                parameter: "dimension",
                reason: "coordinate dimension must be positive",
            });
        }
        let expected = point_count
            .checked_mul(dimension)
            .ok_or(Error::SizeOverflow {
                operation: "point_count * dimension",
            })?;
        if coordinates.len() != expected {
            return Err(Error::ShapeMismatch {
                input: "coordinates",
                expected,
                actual: coordinates.len(),
            });
        }
        for (index, value) in coordinates.iter().enumerate() {
            if !value.is_finite() {
                return Err(Error::NonFiniteValue {
                    field: "coordinates",
                    index: Some(index),
                });
            }
        }
        Ok(Self {
            coordinates,
            point_count,
            dimension,
        })
    }

    /// Number of points, including duplicates.
    pub fn len(&self) -> usize {
        self.point_count
    }

    /// Whether the cloud contains no points.
    pub fn is_empty(&self) -> bool {
        self.point_count == 0
    }

    /// Number of coordinates per point.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Original row-major buffer, with its signed-zero bit patterns unchanged.
    pub fn coordinates(&self) -> &'a [f64] {
        self.coordinates
    }

    /// Coordinates of a point, or `None` if the point index is out of bounds.
    pub fn point(&self, index: usize) -> Option<&'a [f64]> {
        if index >= self.point_count {
            return None;
        }
        // Construction proved the entire shape fits; this row is within it.
        let start = index * self.dimension;
        Some(&self.coordinates[start..start + self.dimension])
    }
}
