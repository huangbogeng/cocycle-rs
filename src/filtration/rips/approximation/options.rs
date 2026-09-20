//! Validated sparse Rips construction parameters.
use crate::geometry::{
    MetricPolicy,
    distance::{cutoff, nonnegative},
};
use crate::{Error, Result};

/// Sparse Rips parameters in GUDHI's edge-length scale convention.
///
/// Default start is vertex zero and equal-distance ties use the smallest original
/// ID. The metric policy is required explicitly. Epsilon >= 1 is permitted but
/// has no approximation bound. Construction is separate from persistence limits.
#[derive(Clone, Copy, Debug)]
pub struct SparseRipsOptions {
    pub(super) epsilon: f64,
    pub(super) policy: MetricPolicy,
    pub(super) start: Option<usize>,
    pub(super) min_radius: f64,
    pub(super) max_scale: Option<f64>,
}
impl SparseRipsOptions {
    /// Set epsilon and the metric hypothesis; use no scale or radius truncation.
    /// # Errors
    /// Rejects nonfinite or nonpositive epsilon and an underflowed blocker factor.
    pub fn new(epsilon: f64, policy: MetricPolicy) -> Result<Self> {
        nonnegative(epsilon, "epsilon", None)?;
        if epsilon == 0.0 {
            return Err(Error::InvalidParameter {
                parameter: "epsilon",
                reason: "must be strictly positive",
            });
        }
        if epsilon < 1.0 && epsilon * (1.0 - epsilon) / 2.0 == 0.0 {
            return Err(Error::NumericalFailure {
                context: "sparse Rips blocker factor underflow",
            });
        }
        Ok(Self {
            epsilon,
            policy,
            start: None,
            min_radius: 0.0,
            max_scale: None,
        })
    }
    /// Choose the first original vertex. Construction validates its range.
    pub fn with_start_vertex(mut self, vertex: usize) -> Self {
        self.start = Some(vertex);
        self
    }
    /// Omit noninitial vertices whose insertion radius is below this value.
    /// # Errors
    /// Rejects negative or nonfinite values.
    pub fn with_min_insertion_radius(mut self, radius: f64) -> Result<Self> {
        self.min_radius = nonnegative(radius, "min_insertion_radius", None)?;
        Ok(self)
    }
    /// Retain sparse edge filtration values at most this threshold (inclusive).
    /// # Errors
    /// Rejects negative or nonfinite thresholds.
    pub fn with_max_scale(mut self, scale: Option<f64>) -> Result<Self> {
        self.max_scale = cutoff(scale)?;
        Ok(self)
    }
}
