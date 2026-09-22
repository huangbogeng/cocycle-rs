//! Boundary reduction driven by the public filtered-cell contract.
//! Only the requested q+1 skeleton is retained; metadata is read for all cells
//! to establish the range. Boundary validity is checked on the retained skeleton.
use super::{PersistenceOptions, RepresentativeRequest, assemble_diagram};
use crate::algebra::{column::Column, reduction};
use crate::complex::FilteredComplex;
use crate::diagram::{ComputationContext, PersistenceDiagram, PersistenceResult};
use crate::execution::WorkBudget;
use crate::filtration::{Coverage, FiltrationKind};
use crate::{Error, Result};
use std::collections::HashMap;

pub(super) struct BoundaryInput<I> {
    pub(super) cells: Vec<I>,
    pub(super) dimensions: Vec<usize>,
    pub(super) values: Vec<f64>,
    pub(super) columns: Vec<Column<usize>>,
    pub(super) coverage: Coverage,
    pub(super) vertex_count: usize,
}

pub(super) fn read<C: FilteredComplex>(
    source: &C,
    options: &PersistenceOptions,
    budget: &mut WorkBudget<'_>,
) -> Result<BoundaryInput<C::CellId>> {
    let field = options.field();
    let mut input = BoundaryInput {
        cells: Vec::new(),
        dimensions: Vec::new(),
        values: Vec::new(),
        columns: Vec::new(),
        coverage: Coverage::Complete,
        vertex_count: 0,
    };
    let mut seen = HashMap::new();
    let mut selected = HashMap::new();
    let mut previous: Option<f64> = None;
    for (index, cell) in source.cells().enumerate() {
        budget.step()?;
        let value = source.filtration_value(cell);
        let dimension = source.dimension(cell);
        if !value.is_finite() || previous.is_some_and(|p| value < p) {
            return Err(invalid(index, "values must be finite and nondecreasing"));
        }
        previous = Some(value);
        seen.try_reserve(1).map_err(|_| allocation())?;
        if seen.insert(cell, ()).is_some() {
            return Err(invalid(index, "duplicate cell ID"));
        }
        if dimension == 0 {
            input.vertex_count = input
                .vertex_count
                .checked_add(1)
                .ok_or(Error::SizeOverflow {
                    operation: "cell vertices",
                })?;
        }
        if options.max_edge().is_some_and(|t| value > t) {
            input.coverage = Coverage::Through(options.max_edge().unwrap());
            continue;
        }
        if dimension > options.max_homology_dimension().saturating_add(1) {
            continue;
        }
        let mut column = Column::new();
        for (face, coefficient) in source.boundary(cell) {
            budget.step()?;
            if coefficient == 0 {
                continue;
            }
            let &row = selected.get(&face).ok_or(invalid(
                index,
                "boundary cell missing or not earlier in filtration",
            ))?;
            if dimension.checked_sub(1) != Some(input.dimensions[row]) {
                return Err(invalid(index, "boundary dimension must decrease by one"));
            }
            let coefficient =
                i64::from(coefficient).rem_euclid(i64::from(field.characteristic())) as u32;
            column.add_term(row, coefficient, field);
        }
        // Detect invalid chain inputs in the field actually used for this run.
        let mut twice = Column::new();
        for (&row, &coefficient) in column.entries() {
            twice.add_scaled(&input.columns[row], coefficient, field, &mut || {
                budget.step()
            })?;
        }
        if !twice.is_empty() {
            return Err(invalid(
                index,
                "boundary of boundary is nonzero in the selected field",
            ));
        }
        let position = input.cells.len();
        selected.try_reserve(1).map_err(|_| allocation())?;
        input.cells.try_reserve(1).map_err(|_| allocation())?;
        input.dimensions.try_reserve(1).map_err(|_| allocation())?;
        input.values.try_reserve(1).map_err(|_| allocation())?;
        input.columns.try_reserve(1).map_err(|_| allocation())?;
        selected.insert(cell, position);
        input.cells.push(cell);
        input.dimensions.push(dimension);
        input.values.push(crate::canonical_zero(value));
        input.columns.push(column);
    }
    budget.check()?;
    Ok(input)
}

pub(super) fn diagram<C: FilteredComplex>(
    source: &C,
    options: &PersistenceOptions,
    coverage: Option<Coverage>,
    budget: &mut WorkBudget<'_>,
) -> Result<(PersistenceDiagram, usize)> {
    let input = read(source, options, budget)?;
    let reduced = reduction::reduce_pairs(input.columns, options.field(), &mut || budget.step())?;
    let mut intervals = Vec::new();
    for (birth, column) in reduced.reduced.iter().enumerate() {
        budget.step()?;
        if column.is_empty() && input.dimensions[birth] <= options.max_homology_dimension() {
            intervals.try_reserve(1).map_err(|_| allocation())?;
            intervals.push((
                input.dimensions[birth],
                input.values[birth],
                reduced.deaths[birth].map(|d| input.values[d]),
            ));
        }
    }
    Ok((
        assemble_diagram(
            options.max_homology_dimension(),
            coverage.unwrap_or(input.coverage),
            intervals,
        )?,
        input.vertex_count,
    ))
}

pub(super) fn compute<C: FilteredComplex>(
    source: &C,
    options: &PersistenceOptions,
    requests: &[RepresentativeRequest],
    budget: &mut WorkBudget<'_>,
) -> Result<PersistenceResult> {
    if !requests.is_empty() {
        return Err(Error::InvalidParameter {
            parameter: "representatives",
            reason: "generic cell analysis has no simplex vertex labels; use a simplicial source",
        });
    }
    let (diagram, vertex_count) = diagram(source, options, None, budget)?;
    Ok(PersistenceResult {
        diagram,
        representatives: None,
        context: ComputationContext::new(
            options.field(),
            FiltrationKind::SuppliedCells,
            vertex_count,
            options.max_edge(),
            None,
            None,
        ),
    })
}
fn invalid(index: usize, reason: &'static str) -> Error {
    Error::InvalidComplex {
        cell: Some(index),
        reason,
    }
}
fn allocation() -> Error {
    Error::AllocationFailed {
        context: "filtered boundary input",
    }
}
