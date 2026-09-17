use super::complex::Simplex;
use super::explicit::{ExplicitFiltration, FilteredSimplex};
use crate::geometry::DissimilarityView;
use crate::{Error, Result};

pub(crate) fn build(
    input: DissimilarityView<'_>,
    max_dimension: usize,
    cutoff: f64,
) -> Result<ExplicitFiltration> {
    let n = input.len();
    let mut cells = Vec::new();
    cells.try_reserve(n).map_err(|_| Error::AllocationFailed {
        context: "vertices",
    })?;
    for i in 0..n {
        cells.push(FilteredSimplex {
            simplex: Simplex::Vertex(i),
            value: 0.0,
        });
    }
    for a in 0..n {
        for b in a + 1..n {
            let ab = distance(input, a, b)?;
            if ab > cutoff {
                continue;
            }
            push(&mut cells, Simplex::Edge([a, b]), ab)?;
            if max_dimension == 0 {
                continue;
            }
            // H1 requires the 2-skeleton: a triangle enters at its longest edge.
            for c in b + 1..n {
                let value = ab.max(distance(input, a, c)?).max(distance(input, b, c)?);
                if value <= cutoff {
                    push(&mut cells, Simplex::Triangle([a, b, c]), value)?;
                }
            }
        }
    }
    ExplicitFiltration::new(cells)
}

fn push(cells: &mut Vec<FilteredSimplex>, simplex: Simplex, value: f64) -> Result<()> {
    cells.try_reserve(1).map_err(|_| Error::AllocationFailed {
        context: "Rips simplices",
    })?;
    cells.push(FilteredSimplex { simplex, value });
    Ok(())
}

pub(crate) fn distance(input: DissimilarityView<'_>, i: usize, j: usize) -> Result<f64> {
    input.get(i, j).ok_or(Error::InternalInvariant {
        reason: "distance index out of bounds",
    })
}
