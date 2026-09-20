//! Persistent homology computations and owned result assembly.
//!
//! The current entry points compute ordinary Rips H0/H1 over F2. Algorithm
//! configuration and working state stay in private, operation-specific modules.

use crate::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use crate::{Error, Result};

#[cfg(test)]
mod reference;
mod rips;

pub use rips::{RipsOptions, rips_from_dissimilarities, rips_from_points};

/// Shared by algorithms; None means unpaired in the computed range, not necessarily essential.
fn assemble_diagram(
    max_dimension: usize,
    coverage: Coverage,
    raw: impl IntoIterator<Item = (usize, f64, Option<f64>)>,
) -> Result<PersistenceDiagram> {
    let mut intervals = Vec::new();
    for (dimension, birth, death) in raw {
        if dimension > max_dimension || death == Some(birth) {
            continue;
        }
        let end = match death {
            Some(value) => IntervalEnd::Finite(value),
            None => match coverage {
                Coverage::Complete => IntervalEnd::Essential,
                Coverage::Through(through) => IntervalEnd::RightCensored { through },
            },
        };
        let interval = PersistenceInterval::new(dimension, birth, end)?;
        intervals
            .try_reserve(1)
            .map_err(|_| Error::AllocationFailed {
                context: "persistence diagram",
            })?;
        intervals.push(interval);
    }
    PersistenceDiagram::new(max_dimension, coverage, intervals)
}

#[cfg(test)]
mod tests;
