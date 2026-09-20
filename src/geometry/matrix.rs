//! Borrowed symmetric dissimilarities in explicitly selected layouts.

use super::DissimilarityView;
use super::dissimilarity::pair_count;
use super::distance::nonnegative;
use crate::{Error, Result, canonical_zero};

/// Storage layout of a borrowed dissimilarity matrix. Diagonal entries in
/// condensed layouts are implicit zero; all layouts are row-major.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixLayout {
    /// Entries `(1,0), (2,0), (2,1), ...`.
    LowerTriangle,
    /// Entries `(0,1), (0,2), ..., (1,2), ...`.
    UpperTriangle,
    /// All n by n entries, including the diagonal.
    Square,
}

/// A validated borrowed symmetric matrix, without a triangle-inequality claim.
///
/// Validation never copies or repairs input. Duplicate/zero distances retain
/// distinct vertices. Square input requires exact symmetry and zero diagonal;
/// signed zero is accepted. Empty buffers require an explicit vertex count.
#[derive(Clone, Copy, Debug)]
pub struct DissimilarityMatrixView<'a> {
    values: &'a [f64],
    n: usize,
    layout: MatrixLayout,
    diameter: f64,
}

impl<'a> DissimilarityMatrixView<'a> {
    /// Validate the selected shape and every value in O(buffer length).
    ///
    /// # Errors
    /// Rejects size overflow, a wrong length, non-finite/negative values,
    /// asymmetric square entries or a nonzero diagonal.
    pub fn new(values: &'a [f64], vertex_count: usize, layout: MatrixLayout) -> Result<Self> {
        let expected = match layout {
            MatrixLayout::Square => vertex_count.checked_mul(vertex_count),
            _ => pair_count(vertex_count),
        }
        .ok_or(Error::SizeOverflow {
            operation: "matrix shape",
        })?;
        if values.len() != expected {
            return Err(Error::ShapeMismatch {
                input: "matrix",
                expected,
                actual: values.len(),
            });
        }
        let mut diameter: f64 = 0.0;
        for (index, &value) in values.iter().enumerate() {
            diameter = diameter.max(nonnegative(value, "matrix", Some(index))?);
        }
        if layout == MatrixLayout::Square {
            for i in 0..vertex_count {
                if values[i * vertex_count + i] != 0.0 {
                    return Err(Error::InvalidMatrix {
                        row: i,
                        column: i,
                        reason: "diagonal must be zero",
                    });
                }
                for j in 0..i {
                    if values[i * vertex_count + j] != values[j * vertex_count + i] {
                        return Err(Error::InvalidMatrix {
                            row: i,
                            column: j,
                            reason: "matrix must be symmetric",
                        });
                    }
                }
            }
        }
        Ok(Self {
            values,
            n: vertex_count,
            layout,
            diameter,
        })
    }

    /// Number of vertices.
    pub fn len(&self) -> usize {
        self.n
    }
    /// Whether there are no vertices.
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
    /// Declared input storage layout.
    pub fn layout(&self) -> MatrixLayout {
        self.layout
    }
    /// Borrow the original buffer in its declared layout, unchanged.
    pub fn values(&self) -> &'a [f64] {
        self.values
    }
    /// Maximum dissimilarity, or zero for fewer than two vertices.
    pub fn diameter(&self) -> f64 {
        self.diameter
    }
    /// Symmetric value, canonicalizing signed zero; `None` for invalid indices.
    pub fn get(&self, i: usize, j: usize) -> Option<f64> {
        if i >= self.n || j >= self.n {
            return None;
        }
        if i == j {
            return Some(0.0);
        }
        let (a, b) = if i < j { (i, j) } else { (j, i) };
        let index = match self.layout {
            MatrixLayout::LowerTriangle => pair_count(b)? + a,
            MatrixLayout::UpperTriangle => {
                pair_count(self.n)? - pair_count(self.n - a)? + (b - a - 1)
            }
            MatrixLayout::Square => i * self.n + j,
        };
        self.values.get(index).copied().map(canonical_zero)
    }
}

impl<'a> From<DissimilarityView<'a>> for DissimilarityMatrixView<'a> {
    /// Adapt an already validated lower triangle without copying or revalidating.
    fn from(input: DissimilarityView<'a>) -> Self {
        Self {
            values: input.values(),
            n: input.len(),
            layout: MatrixLayout::LowerTriangle,
            diameter: input.diameter(),
        }
    }
}
