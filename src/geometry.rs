//! Borrowed, validated numerical inputs.
//!
//! Point clouds use row-major coordinates. Precomputed dissimilarities use a
//! condensed lower triangle: `[d(1,0), d(2,0), d(2,1), ...]`. No input is copied,
//! deduplicated or modified. Finite coordinates and nonnegative dissimilarities
//! do not, on their own, certify a metric.

use crate::{Error, Result, canonical_zero};

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

/// A validated symmetric dissimilarity matrix stored as a condensed lower triangle.
///
/// The diagonal is implicitly zero. Zero off-diagonal values are allowed and
/// do not merge vertices. No triangle-inequality check is performed.
///
/// ```
/// use cocycle::geometry::DissimilarityView;
///
/// let values = [1.0, 4.0, 2.0]; // d(1,0), d(2,0), d(2,1)
/// let distances = DissimilarityView::new(&values, 3)?;
/// assert_eq!(distances.get(0, 2), Some(4.0));
/// assert_eq!(distances.get(2, 2), Some(0.0));
/// # Ok::<(), cocycle::Error>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct DissimilarityView<'a> {
    values: &'a [f64],
    vertex_count: usize,
    diameter: f64,
}

impl<'a> DissimilarityView<'a> {
    /// Validate a condensed lower triangle with an explicit number of vertices.
    ///
    /// Empty buffers represent either zero or one vertex, distinguished by
    /// `vertex_count`. The maximum dissimilarity is cached during validation.
    ///
    /// # Errors
    /// Returns an error for shape overflow, a mismatched buffer length, or any
    /// non-finite or negative dissimilarity. Validation takes O(buffer length).
    pub fn new(values: &'a [f64], vertex_count: usize) -> Result<Self> {
        let expected = pair_count(vertex_count).ok_or(Error::SizeOverflow {
            operation: "vertex_count choose 2",
        })?;
        if values.len() != expected {
            return Err(Error::ShapeMismatch {
                input: "dissimilarities",
                expected,
                actual: values.len(),
            });
        }
        let mut diameter: f64 = 0.0;
        for (index, &value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(Error::NonFiniteValue {
                    field: "dissimilarities",
                    index: Some(index),
                });
            }
            if value < 0.0 {
                return Err(Error::NegativeValue {
                    field: "dissimilarities",
                    index: Some(index),
                });
            }
            diameter = diameter.max(canonical_zero(value));
        }
        Ok(Self {
            values,
            vertex_count,
            diameter,
        })
    }

    /// Number of vertices, not the length of the condensed buffer.
    pub fn len(&self) -> usize {
        self.vertex_count
    }

    /// Whether the matrix has zero vertices.
    pub fn is_empty(&self) -> bool {
        self.vertex_count == 0
    }

    /// Original condensed buffer, with its signed-zero bit patterns unchanged.
    pub fn values(&self) -> &'a [f64] {
        self.values
    }

    /// Maximum dissimilarity, or positive zero when there are fewer than two vertices.
    pub fn diameter(&self) -> f64 {
        self.diameter
    }

    /// Symmetric dissimilarity, or `None` if either vertex index is out of bounds.
    ///
    /// Both the implicit diagonal and any negative-zero entries return positive zero.
    pub fn get(&self, i: usize, j: usize) -> Option<f64> {
        if i >= self.vertex_count || j >= self.vertex_count {
            return None;
        }
        if i == j {
            return Some(0.0);
        }
        let (row, column) = if i > j { (i, j) } else { (j, i) };
        // This prefix fits because the constructor validated the whole shape.
        let offset = pair_count(row)?.checked_add(column)?;
        self.values.get(offset).copied().map(canonical_zero)
    }
}

/// Divide before multiplying so representable binomial coefficients are accepted.
fn pair_count(n: usize) -> Option<usize> {
    if n < 2 {
        Some(0)
    } else if n.is_multiple_of(2) {
        (n / 2).checked_mul(n - 1)
    } else {
        n.checked_mul((n - 1) / 2)
    }
}

/// Stable Euclidean norms in the same condensed order used by DissimilarityView.
pub(crate) fn euclidean_distances(input: PointCloudView<'_>) -> Result<Vec<f64>> {
    let count = pair_count(input.len()).ok_or(Error::SizeOverflow {
        operation: "point count choose 2",
    })?;
    let mut values = Vec::new();
    values
        .try_reserve(count)
        .map_err(|_| Error::AllocationFailed {
            context: "Euclidean distances",
        })?;
    for i in 0..input.len() {
        for j in 0..i {
            let mut norm: f64 = 0.0;
            let a = input.point(i).ok_or(Error::InternalInvariant {
                reason: "missing point",
            })?;
            let b = input.point(j).ok_or(Error::InternalInvariant {
                reason: "missing point",
            })?;
            for (&x, &y) in a.iter().zip(b) {
                // hypot avoids forming squared magnitudes that overflow/underflow.
                norm = norm.hypot(x - y);
            }
            if !norm.is_finite() {
                return Err(Error::NumericalFailure {
                    context: "Euclidean distance",
                });
            }
            values.push(canonical_zero(norm));
        }
    }
    Ok(values)
}
