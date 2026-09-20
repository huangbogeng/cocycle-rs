//! Perturbed sparse edge values in edge-length units.
use crate::{Error, Result};

pub(super) fn value(d: f64, earlier: Option<f64>, later: f64, epsilon: f64) -> Result<Option<f64>> {
    // Follow the same binary64 comparisons in ordinary representable ranges.
    // Detect overflow instead of using accidental infinity comparisons to decide topology.
    let scaled = finite(d * epsilon)?;
    let twice_later = finite(2.0 * later)?;
    if scaled <= twice_later {
        return Ok(Some(d));
    }
    if let Some(earlier) = earlier
        && scaled > finite(earlier + later)?
    {
        return Ok(None);
    }
    // In this branch later / epsilon < d / 2; the quotient cannot overflow.
    let value = finite(2.0 * (d - later / epsilon))?;
    if super::blocker::allows(
        value,
        super::blocker::factor(epsilon),
        [Some(later)].into_iter(),
    ) {
        Ok(Some(value))
    } else {
        Ok(None)
    }
}
fn finite(value: f64) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Error::NumericalFailure {
            context: "sparse Rips edge arithmetic overflow",
        })
    }
}
