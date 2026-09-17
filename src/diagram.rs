//! Owned persistence intervals and their computation range.
//!
//! These types validate interval and range consistency, not whether a diagram
//! was actually produced by a particular dataset or algorithm. Dimensions are
//! general nonnegative integers and scales may be negative. Rips-specific facts
//! such as zero-dimensional births at zero belong to the Rips computation.

use std::cmp::Ordering;

use crate::{Error, Result, canonical_zero};

/// How a persistence interval ends.
///
/// Raw variants can carry invalid floats. [`PersistenceInterval::new`] validates
/// them before storing an interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IntervalEnd {
    /// A finite death scale, excluded from the interval.
    Finite(f64),
    /// A class that never dies in the complete filtration.
    Essential,
    /// A class still alive at the inclusive cutoff, with its eventual death unknown.
    RightCensored {
        /// Last scale through which the class is known to exist.
        through: f64,
    },
}

/// A positive-length finite interval, an essential interval, or a censored interval.
///
/// Finite intervals use `[birth, death)`. A censored interval includes its
/// `through` scale and may have `birth == through`. Zero-length finite intervals
/// are not stored in public diagrams.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PersistenceInterval {
    dimension: usize,
    birth: f64,
    end: IntervalEnd,
}

impl PersistenceInterval {
    /// Validate an interval without imposing assumptions about a particular complex.
    ///
    /// Negative scales and any homology dimension are permitted. Finite scale
    /// values are canonicalized to positive zero when equal to zero.
    ///
    /// # Errors
    /// Returns an error for non-finite scales, finite death at or before birth,
    /// or a censoring cutoff before birth. A finite lifetime need not itself be
    /// representable as `f64`; computations using it must check overflow.
    pub fn new(dimension: usize, birth: f64, end: IntervalEnd) -> Result<Self> {
        let birth = finite_scale(birth, "birth")?;
        let end = match end {
            IntervalEnd::Finite(death) => {
                let death = finite_scale(death, "death")?;
                if death <= birth {
                    return Err(Error::InvalidInterval {
                        reason: "finite death must be strictly after birth",
                    });
                }
                IntervalEnd::Finite(death)
            }
            IntervalEnd::Essential => IntervalEnd::Essential,
            IntervalEnd::RightCensored { through } => {
                let through = finite_scale(through, "through")?;
                if through < birth {
                    return Err(Error::InvalidInterval {
                        reason: "censoring cutoff must be at or after birth",
                    });
                }
                IntervalEnd::RightCensored { through }
            }
        };
        Ok(Self {
            dimension,
            birth,
            end,
        })
    }

    /// Homology dimension of this interval.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Inclusive birth scale.
    pub fn birth(&self) -> f64 {
        self.birth
    }

    /// Finite death, essential status, or inclusive censoring cutoff.
    pub fn end(&self) -> IntervalEnd {
        self.end
    }
}

/// The range over which a diagram was computed.
///
/// This describes coverage of the filtration, not a memory limit. A raw `Through`
/// value is validated by [`PersistenceDiagram::new`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Coverage {
    /// All changes in the filtration are accounted for.
    Complete,
    /// All changes up to and including the given finite scale are accounted for.
    Through(f64),
}

/// An owned persistence diagram with explicitly recorded dimensions and coverage.
///
/// Computed dimensions are `0..=max_dimension`, even if their interval lists are
/// empty. The interval multiset is sorted deterministically; multiplicities are
/// preserved. No original input, simplex IDs or matrix indices are retained.
///
/// ```
/// use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
///
/// let intervals = vec![PersistenceInterval::new(1, 1.0, IntervalEnd::Finite(2.0))?];
/// let diagram = PersistenceDiagram::new(1, Coverage::Complete, intervals)?;
/// assert_eq!(diagram.intervals_in_dimension(0)?.count(), 0);
/// assert_eq!(diagram.intervals_in_dimension(1)?.count(), 1);
/// assert!(diagram.intervals_in_dimension(2).is_err());
/// # Ok::<(), cocycle::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceDiagram {
    max_dimension: usize,
    coverage: Coverage,
    intervals: Vec<PersistenceInterval>,
}

impl PersistenceDiagram {
    /// Take ownership of intervals and validate them against the declared range.
    ///
    /// Complete diagrams may contain finite and essential intervals. For a
    /// truncated diagram, all births and finite deaths must be at or before its
    /// cutoff, all censoring cutoffs must equal it, and essential intervals are
    /// rejected. This conservative convention does not infer eventual survival.
    ///
    /// Sorting is by dimension, birth, endpoint kind (finite, essential, censored),
    /// and then endpoint value. Equal intervals are retained. Validation and
    /// sorting take O(m log m) time for m intervals and do not copy the vector.
    ///
    /// # Errors
    /// Returns an error for a non-finite cutoff, an interval outside the computed
    /// dimensions, or an interval inconsistent with coverage. Errors referring to
    /// an interval index refer to its position before sorting.
    pub fn new(
        max_dimension: usize,
        coverage: Coverage,
        mut intervals: Vec<PersistenceInterval>,
    ) -> Result<Self> {
        let coverage = match coverage {
            Coverage::Complete => Coverage::Complete,
            Coverage::Through(value) => Coverage::Through(finite_scale(value, "coverage")?),
        };
        for (index, interval) in intervals.iter().enumerate() {
            if interval.dimension > max_dimension {
                return Err(Error::DimensionNotComputed {
                    requested: interval.dimension,
                    computed_max: max_dimension,
                });
            }
            validate_coverage(interval, coverage, index)?;
        }
        intervals.sort_unstable_by(compare_intervals);
        Ok(Self {
            max_dimension,
            coverage,
            intervals,
        })
    }

    /// Largest dimension recorded as computed, including every lower dimension.
    pub fn max_dimension(&self) -> usize {
        self.max_dimension
    }

    /// Complete or inclusive finite computation range.
    pub fn coverage(&self) -> Coverage {
        self.coverage
    }

    /// All intervals, in deterministic order and with multiplicities preserved.
    pub fn intervals(&self) -> &[PersistenceInterval] {
        &self.intervals
    }

    /// Iterate over intervals in a computed dimension, which may have no intervals.
    ///
    /// # Errors
    /// Returns [`Error::DimensionNotComputed`] for a dimension above the recorded
    /// maximum. An empty diagram does not imply that a dimension was uncomputed.
    pub fn intervals_in_dimension(
        &self,
        dimension: usize,
    ) -> Result<impl Iterator<Item = &PersistenceInterval>> {
        if dimension > self.max_dimension {
            return Err(Error::DimensionNotComputed {
                requested: dimension,
                computed_max: self.max_dimension,
            });
        }
        let start = self
            .intervals
            .partition_point(|interval| interval.dimension < dimension);
        let end = self
            .intervals
            .partition_point(|interval| interval.dimension <= dimension);
        Ok(self.intervals[start..end].iter())
    }
}

fn finite_scale(value: f64, field: &'static str) -> Result<f64> {
    if !value.is_finite() {
        Err(Error::NonFiniteValue { field, index: None })
    } else {
        Ok(canonical_zero(value))
    }
}

fn validate_coverage(
    interval: &PersistenceInterval,
    coverage: Coverage,
    index: usize,
) -> Result<()> {
    let reason = match coverage {
        Coverage::Complete => match interval.end {
            IntervalEnd::RightCensored { .. } => {
                Some("complete coverage cannot contain censored intervals")
            }
            _ => None,
        },
        Coverage::Through(cutoff) => {
            if interval.birth > cutoff {
                Some("birth exceeds the computation cutoff")
            } else {
                match interval.end {
                    IntervalEnd::Essential => {
                        Some("truncated coverage cannot certify an essential interval")
                    }
                    IntervalEnd::Finite(death) if death > cutoff => {
                        Some("death exceeds the computation cutoff")
                    }
                    IntervalEnd::RightCensored { through } if through != cutoff => {
                        Some("interval and diagram censoring cutoffs differ")
                    }
                    _ => None,
                }
            }
        }
    };
    match reason {
        Some(reason) => Err(Error::InconsistentDiagram {
            interval: index,
            reason,
        }),
        None => Ok(()),
    }
}

fn compare_intervals(a: &PersistenceInterval, b: &PersistenceInterval) -> Ordering {
    a.dimension
        .cmp(&b.dimension)
        .then_with(|| a.birth.total_cmp(&b.birth))
        .then_with(|| compare_ends(a.end, b.end))
}

fn compare_ends(a: IntervalEnd, b: IntervalEnd) -> Ordering {
    match (a, b) {
        (IntervalEnd::Finite(a), IntervalEnd::Finite(b)) => a.total_cmp(&b),
        (IntervalEnd::Essential, IntervalEnd::Essential) => Ordering::Equal,
        (IntervalEnd::RightCensored { through: a }, IntervalEnd::RightCensored { through: b }) => {
            a.total_cmp(&b)
        }
        (IntervalEnd::Finite(_), _)
        | (IntervalEnd::Essential, IntervalEnd::RightCensored { .. }) => Ordering::Less,
        _ => Ordering::Greater,
    }
}
