//! Persistence from frozen Rips incidence with dimension and range checks.
use crate::diagram::{ComputationContext, Coverage, FiltrationKind, PersistenceResult};
use crate::filtration::{RipsExpansion, RipsInputKind, flag::ExplicitAccess};
use crate::persistence::{
    ExecutionLimits, PersistenceOptions, RepresentativeRequest, execution::WorkBudget, flag,
};
use crate::{Error, Result};

/// Compute ordinary prime-field persistence from an explicit exact Rips expansion.
///
/// Computing Hq requires construction through q+1, unless expansion certified
/// that no higher simplices exist. This prevents artificial essential classes
/// caused by a truncated skeleton. Scale coverage retains the original input's
/// meaning. Reduction reads the stored incidence without reconstructing a graph.
///
/// # Errors
/// Rejects insufficient construction dimension or an unavailable scale range;
/// also returns execution, allocation and invariant errors without partial output.
pub fn compute_expanded_rips(
    input: &RipsExpansion,
    options: &PersistenceOptions,
    limits: &ExecutionLimits<'_>,
) -> Result<PersistenceResult> {
    compute_expanded_rips_with_representatives(input, options, &[], limits)
}

/// Compute explicit Rips persistence and requested cycle/cocycle bases.
///
/// Reads the existing incidence and retains additional boundary transformations
/// for nonempty requests. Uses the same dimension and scale sufficiency checks
/// as [`compute_expanded_rips`]. Execution limits include representative work.
///
/// # Errors
/// Includes [`compute_expanded_rips`] errors, uncomputed dimensions and query
/// scales outside coverage. No partial output is returned.
pub fn compute_expanded_rips_with_representatives(
    input: &RipsExpansion,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    limits: &ExecutionLimits<'_>,
) -> Result<PersistenceResult> {
    let mut budget = WorkBudget::new(limits)?;
    if !input.complete && options.max_homology_dimension() >= input.max_simplex_dimension {
        return Err(Error::InsufficientSkeleton {
            requested_homology_dimension: options.max_homology_dimension(),
            constructed_simplex_dimension: input.max_simplex_dimension,
        });
    }
    let (cutoff, coverage) = match input.coverage {
        Coverage::Through(through) => {
            let cutoff = options.max_edge().unwrap_or(through);
            if cutoff > through {
                return Err(Error::IncompleteFiltration {
                    requested: cutoff,
                    through,
                });
            }
            (cutoff, Coverage::Through(cutoff))
        }
        Coverage::Complete => match options.max_edge() {
            Some(cutoff) if cutoff < input.max_edge => (cutoff, Coverage::Through(cutoff)),
            _ => (input.max_edge, Coverage::Complete),
        },
    };
    let access = ExplicitAccess {
        complex: &input.complex,
        vertex_count: input.vertex_count,
        cutoff,
    };
    let (diagram, representatives) = flag::finish(
        &access,
        options,
        requests,
        coverage,
        &mut budget,
        |budget| {
            flag::compute_simplicial(
                &access,
                options.max_homology_dimension(),
                options.field(),
                budget,
            )
        },
    )?;
    Ok(PersistenceResult {
        diagram,
        representatives,
        context: ComputationContext {
            approximation: None,
            field: options.field(),
            kind: match input.kind {
                RipsInputKind::Dissimilarities => FiltrationKind::RipsDissimilarities,
                RipsInputKind::Euclidean => FiltrationKind::RipsEuclidean,
                RipsInputKind::Custom => FiltrationKind::RipsCustom,
            },
            vertex_count: input.vertex_count,
            requested_cutoff: options.max_edge(),
            construction_cutoff: input.requested_cutoff,
        },
    })
}
