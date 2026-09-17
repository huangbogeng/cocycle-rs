//! Test-only explicit boundary reduction, independent of production cohomology.
//!
//! Keep this oracle simple: its purpose is to catch errors in the optimized path.

mod boundary;
mod column;
mod complex;
mod explicit;
pub(super) mod reduction;
mod rips;

use super::{RipsOptions, finish, range};
use crate::Result;
use crate::diagram::PersistenceDiagram;
use crate::geometry::DissimilarityView;
pub(super) use boundary::FilteredBoundary;

pub(super) fn compute(
    input: DissimilarityView<'_>,
    options: &RipsOptions,
) -> Result<PersistenceDiagram> {
    let (cutoff, coverage) = range(input, options);
    let filtration = rips::build(input, options.max_dimension(), cutoff)?;
    let reduced = reduction::reduce(&filtration)?;
    let paired = reduced.pairs.into_iter().map(|(i, j)| {
        (
            filtration.dimension(i),
            filtration.value(i),
            Some(filtration.value(j)),
        )
    });
    let unpaired = reduced
        .unpaired
        .into_iter()
        .map(|i| (filtration.dimension(i), filtration.value(i), None));
    finish(options.max_dimension(), coverage, paired.chain(unpaired))
}

mod tests;
