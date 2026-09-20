//! Condensed symmetric dissimilarities; no metric assumption.

use crate::{Error, Result, canonical_zero};

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
pub(super) fn pair_count(n: usize) -> Option<usize> {
    if n < 2 {
        Some(0)
    } else if n.is_multiple_of(2) {
        (n / 2).checked_mul(n - 1)
    } else {
        n.checked_mul((n - 1) / 2)
    }
}
