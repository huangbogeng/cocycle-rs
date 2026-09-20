//! Shared checked scalar operations for actual distance consumers.

use crate::{Error, Result, canonical_zero};

pub(crate) fn nonnegative(value: f64, field: &'static str, index: Option<usize>) -> Result<f64> {
    if !value.is_finite() {
        return Err(Error::NonFiniteValue { field, index });
    }
    if value < 0.0 {
        return Err(Error::NegativeValue { field, index });
    }
    Ok(canonical_zero(value))
}

pub(crate) fn cutoff(value: Option<f64>) -> Result<Option<f64>> {
    value.map(|v| nonnegative(v, "max_edge", None)).transpose()
}
