use std::collections::HashMap;

use super::FilteredBoundary;
use super::complex::Simplex;
use crate::{Error, Result, canonical_zero};

#[derive(Clone, Copy, Debug)]
pub(crate) struct FilteredSimplex {
    pub(crate) simplex: Simplex,
    pub(crate) value: f64,
}

pub(crate) struct ExplicitFiltration {
    cells: Vec<FilteredSimplex>,
    indices: HashMap<Simplex, usize>,
}

impl ExplicitFiltration {
    pub(crate) fn new(mut cells: Vec<FilteredSimplex>) -> Result<Self> {
        for cell in &mut cells {
            if !cell.value.is_finite() || !cell.simplex.is_valid() {
                return Err(Error::InternalInvariant {
                    reason: "invalid filtered simplex",
                });
            }
            cell.value = canonical_zero(cell.value);
        }
        cells.sort_unstable_by(|a, b| {
            a.value
                .total_cmp(&b.value)
                .then_with(|| a.simplex.dimension().cmp(&b.simplex.dimension()))
                .then_with(|| a.simplex.cmp(&b.simplex))
        });
        let mut indices = HashMap::new();
        indices
            .try_reserve(cells.len())
            .map_err(|_| Error::AllocationFailed {
                context: "simplex indices",
            })?;
        for (index, cell) in cells.iter().enumerate() {
            if indices.insert(cell.simplex, index).is_some() {
                return Err(Error::InternalInvariant {
                    reason: "duplicate simplex",
                });
            }
        }
        // Closure and face-before-coface order imply a valid simplicial boundary.
        for (index, cell) in cells.iter().enumerate() {
            for face in cell.simplex.faces().into_iter().flatten() {
                if !indices.get(&face).is_some_and(|&row| row < index) {
                    return Err(Error::InternalInvariant {
                        reason: "missing face or face after coface",
                    });
                }
            }
        }
        Ok(Self { cells, indices })
    }
}

impl FilteredBoundary for ExplicitFiltration {
    fn len(&self) -> usize {
        self.cells.len()
    }
    fn dimension(&self, index: usize) -> usize {
        self.cells[index].simplex.dimension()
    }
    fn value(&self, index: usize) -> f64 {
        self.cells[index].value
    }

    fn write_boundary(&self, index: usize, output: &mut Vec<usize>) -> Result<()> {
        output.clear();
        output.try_reserve(3).map_err(|_| Error::AllocationFailed {
            context: "boundary column",
        })?;
        for face in self.cells[index].simplex.faces().into_iter().flatten() {
            let row = *self.indices.get(&face).ok_or(Error::InternalInvariant {
                reason: "missing boundary face",
            })?;
            output.push(row);
        }
        output.sort_unstable();
        Ok(())
    }
}
