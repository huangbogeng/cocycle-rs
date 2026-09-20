//! Test-only explicit boundary reduction, independent of production cohomology.
//!
//! Keep this oracle simple: its purpose is to catch errors in the optimized path.

mod boundary;
mod column;
mod complex;
mod explicit;
pub(super) mod reduction;
mod rips;

use super::rips::resolve_rips_range;
use super::{RipsOptions, assemble_diagram};
use crate::Result;
use crate::diagram::PersistenceDiagram;
use crate::geometry::DissimilarityView;
pub(super) use boundary::FilteredBoundary;

pub(super) fn compute(
    input: DissimilarityView<'_>,
    options: &RipsOptions,
) -> Result<PersistenceDiagram> {
    let (cutoff, coverage) = resolve_rips_range(input, options);
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
    assemble_diagram(options.max_dimension(), coverage, paired.chain(unpaired))
}

mod tests;

/// Independent small-graph oracle: enumerate vertex triples, not production cofacets.
pub(super) fn compute_graph(
    n: usize,
    edges: &[crate::complex::WeightedEdge],
    q: usize,
    cutoff: f64,
    coverage: crate::diagram::Coverage,
) -> Result<PersistenceDiagram> {
    use complex::Simplex;
    use explicit::{ExplicitFiltration, FilteredSimplex};
    let mut cells: Vec<_> = (0..n)
        .map(|v| FilteredSimplex {
            simplex: Simplex::Vertex(v),
            value: 0.0,
        })
        .collect();
    let weights: std::collections::BTreeMap<_, _> =
        edges.iter().map(|e| (e.vertices, e.value)).collect();
    for a in 0..n {
        for b in a + 1..n {
            let Some(&ab) = weights.get(&[a, b]).filter(|&&v| v <= cutoff) else {
                continue;
            };
            cells.push(FilteredSimplex {
                simplex: Simplex::Edge([a, b]),
                value: ab,
            });
            if q == 0 {
                continue;
            }
            for c in b + 1..n {
                if let (Some(&ac), Some(&bc)) = (weights.get(&[a, c]), weights.get(&[b, c])) {
                    let value = ab.max(ac).max(bc);
                    if value <= cutoff {
                        cells.push(FilteredSimplex {
                            simplex: Simplex::Triangle([a, b, c]),
                            value,
                        });
                    }
                }
            }
        }
    }
    let filtration = ExplicitFiltration::new(cells)?;
    let reduced = reduction::reduce(&filtration)?;
    let paired = reduced.pairs.into_iter().map(|(a, b)| {
        (
            filtration.dimension(a),
            filtration.value(a),
            Some(filtration.value(b)),
        )
    });
    let unpaired = reduced
        .unpaired
        .into_iter()
        .map(|a| (filtration.dimension(a), filtration.value(a), None));
    assemble_diagram(q, coverage, paired.chain(unpaired))
}
