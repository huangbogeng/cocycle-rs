use crate::{Error, Result, canonical_zero};

/// Validated options for Rips persistence.
///
/// Dimension one requests both H0 and H1. `None` for the maximum edge length
/// requests the complete filtration. These are mathematical parameters, not
/// memory or execution-time guarantees.
///
/// The default requests H0/H1 with no edge cutoff.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RipsOptions {
    max_dimension: usize,
    max_edge: Option<f64>,
}

impl RipsOptions {
    /// Create options with an inclusive maximum homology dimension and edge cutoff.
    ///
    /// # Errors
    /// Returns an error for dimensions greater than one or a non-finite/negative
    /// edge cutoff. Negative zero is accepted and canonicalized to positive zero.
    pub fn new(max_dimension: usize, max_edge: Option<f64>) -> Result<Self> {
        if max_dimension > 1 {
            return Err(Error::UnsupportedDimension {
                requested: max_dimension,
                max_supported: 1,
            });
        }
        if let Some(value) = max_edge {
            if !value.is_finite() {
                return Err(Error::NonFiniteValue {
                    field: "max_edge",
                    index: None,
                });
            }
            if value < 0.0 {
                return Err(Error::NegativeValue {
                    field: "max_edge",
                    index: None,
                });
            }
        }
        Ok(Self {
            max_dimension,
            max_edge: max_edge.map(canonical_zero),
        })
    }

    /// Largest homology dimension requested, including all lower dimensions.
    pub fn max_dimension(&self) -> usize {
        self.max_dimension
    }

    /// Inclusive maximum edge length, or `None` for a complete computation.
    pub fn max_edge(&self) -> Option<f64> {
        self.max_edge
    }
}

impl Default for RipsOptions {
    fn default() -> Self {
        Self {
            max_dimension: 1,
            max_edge: None,
        }
    }
}
