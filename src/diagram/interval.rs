//! Persistence intervals and validated endpoint semantics.

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

pub(super) fn finite_scale(value: f64, field: &'static str) -> Result<f64> {
    if !value.is_finite() {
        Err(Error::NonFiniteValue { field, index: None })
    } else {
        Ok(canonical_zero(value))
    }
}
