//! Persistence and representatives of explicit simplicial topology.
pub(super) mod representatives;

use super::{PersistenceOptions, RepresentativeRequest};
use crate::complex::SimplicialComplex;
use crate::diagram::{PersistenceDiagram, Representative};
use crate::execution::WorkBudget;
use crate::filtration::Coverage;
use crate::{Error, Result};
pub(super) fn source_range(
    coverage: Coverage,
    maximum: Option<f64>,
    requested: Option<f64>,
) -> Result<(Option<f64>, Coverage)> {
    match coverage {
        Coverage::Through(through) => {
            let value = requested.unwrap_or(through);
            if value > through {
                return Err(Error::IncompleteFiltration {
                    requested: value,
                    through,
                });
            }
            Ok((Some(value), Coverage::Through(value)))
        }
        Coverage::Complete => Ok((
            requested,
            match (requested, maximum) {
                (Some(t), Some(m)) if t < m => Coverage::Through(t),
                _ => Coverage::Complete,
            },
        )),
    }
}
pub(super) fn compute(
    source: &SimplicialComplex,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    coverage: Coverage,
    budget: &mut WorkBudget<'_>,
) -> Result<(PersistenceDiagram, Option<Vec<Representative>>)> {
    let result = if requests.is_empty() {
        (
            super::filtered::diagram(source, options, Some(coverage), budget)?.0,
            None,
        )
    } else {
        let (diagram, representatives) =
            representatives::compute_explicit(source, options, requests, coverage, budget)?;
        (diagram, Some(representatives))
    };
    budget.check()?;
    Ok(result)
}

pub(super) mod cohomology;
