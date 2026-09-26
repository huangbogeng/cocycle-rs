//! Persistence and representatives of explicit simplicial topology.
pub(super) mod representatives;

use super::{PersistenceOptions, RepresentativeRequest};
use crate::complex::SimplicialComplex;
use crate::diagram::{PersistenceDiagram, Representative};
use crate::execution::WorkBudget;
use crate::filtration::Coverage;
use crate::filtration::simplicial::ZeroBornExplicitAccess;
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
    let result = if !requests.is_empty() {
        let (diagram, representatives) =
            representatives::compute_explicit(source, options, requests, coverage, budget)?;
        (diagram, Some(representatives))
    } else if options.max_edge().is_none_or(|t| t >= 0.) && zero_born(source, budget)? {
        // Select by the actual simplex invariant, not source/scale metadata.
        // Stored cofaces retain non-flag topology and arbitrary simplex values.
        let access = ZeroBornExplicitAccess {
            complex: source,
            vertex_count: source.vertex_count(),
            cutoff: options
                .max_edge()
                .unwrap_or_else(|| source.max_filtration_value().unwrap_or(0.)),
        };
        (
            super::assemble_diagram(
                options.max_homology_dimension(),
                coverage,
                cohomology::compute(
                    &access,
                    options.max_homology_dimension(),
                    options.field(),
                    budget,
                )?,
            )?,
            None,
        )
    } else {
        (
            super::filtered::diagram(source, options, Some(coverage), budget)?.0,
            None,
        )
    };
    budget.check()?;
    Ok(result)
}

fn zero_born(source: &SimplicialComplex, budget: &mut WorkBudget<'_>) -> Result<bool> {
    // Face monotonicity and (value, dimension, vertices) order imply that all
    // vertices form this prefix exactly when every vertex is born at zero.
    // This also establishes the compact vertex positions required by union-find.
    for simplex in source.simplices().iter().take(source.vertex_count()) {
        budget.step()?;
        if simplex.dimension() != 0 || simplex.value() != 0. {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) mod cohomology;
