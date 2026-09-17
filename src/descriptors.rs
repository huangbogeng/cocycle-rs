//! Statistics and curves computed from an existing persistence diagram.
//!
//! These functions never rerun a persistence algorithm. Lifetime statistics use
//! only finite intervals; Betti curves also count essential and censored classes.

use crate::diagram::{Coverage, IntervalEnd, PersistenceDiagram};
use crate::{Error, Result, canonical_zero};

/// Finite positive-lifetime statistics for one homology dimension.
#[derive(Clone, Debug, PartialEq)]
pub struct LifetimeSummary {
    finite_count: usize,
    total_persistence: f64,
    max_persistence: Option<f64>,
    entropy: Option<f64>,
    excluded_essential_count: usize,
    excluded_censored_count: usize,
}

impl LifetimeSummary {
    /// Number of finite intervals included in the summary.
    pub fn finite_count(&self) -> usize {
        self.finite_count
    }
    /// Sum of finite lifetimes (power one), or zero when none are present.
    pub fn total_persistence(&self) -> f64 {
        self.total_persistence
    }
    /// Largest finite lifetime, or `None` when none are present.
    pub fn max_persistence(&self) -> Option<f64> {
        self.max_persistence
    }
    /// Shannon entropy of normalized finite lifetimes, using natural logarithms.
    ///
    /// This is measured in nats, without normalization by the logarithm of the
    /// interval count. No finite intervals yields `None`; one yields zero.
    pub fn entropy(&self) -> Option<f64> {
        self.entropy
    }
    /// Essential intervals excluded in the selected dimension.
    pub fn excluded_essential_count(&self) -> usize {
        self.excluded_essential_count
    }
    /// Censored intervals excluded in the selected dimension.
    pub fn excluded_censored_count(&self) -> usize {
        self.excluded_censored_count
    }
}

/// Summarize the observed finite lifetimes in one computed dimension.
///
/// Uses compensated summation for total persistence. Censored endpoints are not
/// treated as deaths. Runs in O(m) time and O(1) auxiliary space for m selected
/// intervals, following the initial dimension lookup.
///
/// # Errors
/// Returns an error if the dimension was not computed or a lifetime or its total
/// is not representable as a finite `f64`.
pub fn finite_lifetime_summary(
    diagram: &PersistenceDiagram,
    dimension: usize,
) -> Result<LifetimeSummary> {
    let mut summary = LifetimeSummary {
        finite_count: 0,
        total_persistence: 0.0,
        max_persistence: None,
        entropy: None,
        excluded_essential_count: 0,
        excluded_censored_count: 0,
    };
    let mut compensation = 0.0;
    for interval in diagram.intervals_in_dimension(dimension)? {
        match interval.end() {
            IntervalEnd::Essential => summary.excluded_essential_count += 1,
            IntervalEnd::RightCensored { .. } => summary.excluded_censored_count += 1,
            IntervalEnd::Finite(death) => {
                let lifetime = death - interval.birth();
                if !lifetime.is_finite() {
                    return Err(Error::NumericalFailure {
                        context: "finite lifetime",
                    });
                }
                let corrected = lifetime - compensation;
                let next = summary.total_persistence + corrected;
                if !next.is_finite() {
                    return Err(Error::NumericalFailure {
                        context: "total persistence",
                    });
                }
                compensation = (next - summary.total_persistence) - corrected;
                summary.total_persistence = next;
                summary.max_persistence = Some(
                    summary
                        .max_persistence
                        .map_or(lifetime, |old| old.max(lifetime)),
                );
                summary.finite_count += 1;
            }
        }
    }
    if summary.finite_count > 0 {
        let mut entropy = 0.0;
        for interval in diagram.intervals_in_dimension(dimension)? {
            if let IntervalEnd::Finite(death) = interval.end() {
                let probability = (death - interval.birth()) / summary.total_persistence;
                // Extreme ratios may round to zero; their limiting contribution is zero.
                if probability > 0.0 {
                    entropy -= probability * probability.ln();
                }
            }
        }
        if !entropy.is_finite() {
            return Err(Error::NumericalFailure {
                context: "persistence entropy",
            });
        }
        summary.entropy = Some(canonical_zero(entropy));
    }
    Ok(summary)
}

/// Count live intervals at the given finite, nonnegative, strictly increasing scales.
///
/// Births are included and finite deaths excluded. A censored class remains alive
/// at the cutoff itself. Complete diagrams may be queried beyond the largest
/// observed endpoint. An empty grid yields an empty vector after dimension checks.
///
/// Sorts birth/death events, then scans the grid in O(m log m + g) time and O(m + g)
/// space for m selected intervals and g query values. The first version's grid
/// contract is nonnegative, including for manually constructed signed-scale diagrams.
///
/// # Errors
/// Returns an error for an uncomputed dimension, an invalid grid, a query past
/// censored coverage, or a failed memory reservation. Failure returns no partial curve.
pub fn betti_curve(
    diagram: &PersistenceDiagram,
    dimension: usize,
    grid: &[f64],
) -> Result<Vec<usize>> {
    let intervals = diagram.intervals_in_dimension(dimension)?;
    for (index, &value) in grid.iter().enumerate() {
        if !value.is_finite() || value < 0.0 {
            return Err(Error::InvalidGrid {
                index,
                reason: "scale must be finite and nonnegative",
            });
        }
        if index > 0 && value <= grid[index - 1] {
            return Err(Error::InvalidGrid {
                index,
                reason: "scales must be strictly increasing",
            });
        }
        if let Coverage::Through(through) = diagram.coverage()
            && value > through
        {
            return Err(Error::QueryOutsideCoverage {
                index,
                value,
                through,
            });
        }
    }
    if grid.is_empty() {
        return Ok(Vec::new());
    }
    let mut births = Vec::new();
    let mut deaths = Vec::new();
    for interval in intervals {
        births.try_reserve(1).map_err(|_| Error::AllocationFailed {
            context: "Betti births",
        })?;
        births.push(interval.birth());
        if let IntervalEnd::Finite(death) = interval.end() {
            deaths.try_reserve(1).map_err(|_| Error::AllocationFailed {
                context: "Betti deaths",
            })?;
            deaths.push(death);
        }
    }
    births.sort_unstable_by(f64::total_cmp);
    deaths.sort_unstable_by(f64::total_cmp);
    let mut result = Vec::new();
    result
        .try_reserve(grid.len())
        .map_err(|_| Error::AllocationFailed {
            context: "Betti curve",
        })?;
    let (mut born, mut dead) = (0, 0);
    for &value in grid {
        while born < births.len() && births[born] <= value {
            born += 1;
        }
        while dead < deaths.len() && deaths[dead] <= value {
            dead += 1;
        }
        result.push(born - dead); // Every counted death has an earlier counted birth.
    }
    Ok(result)
}
