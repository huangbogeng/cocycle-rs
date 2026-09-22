//! F2 Rips H1 via implicit coboundaries and stored change-of-basis columns.
//! Independently implemented from the invariants in docs/reference/mathematics.md section 9;
//! the explicit boundary reducer remains an independent test oracle.

pub(super) mod dimensions;

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::union_find::UnionFind;
#[cfg(test)]
use crate::filtration::flag::DenseFlag;
use crate::filtration::flag::{FlagAccess, SimplexEntry};
#[cfg(test)]
use crate::filtration::rips::cone_radius;
#[cfg(test)]
use crate::geometry::DissimilarityView;
use crate::persistence::execution::WorkBudget;
use crate::{Error, Result};

type RawIntervals = Vec<(usize, f64, Option<f64>)>;
type Coboundary = BinaryHeap<Reverse<SimplexEntry>>;

/// Position in the forward-ordered edge array, not a combinatorial simplex ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct EdgePosition(usize);

/// Position in the stored transformation-column array.
#[derive(Clone, Copy)]
struct ColumnPosition(usize);

struct TransformColumn {
    // The diagonal entry of V is implicit. Other entries index later edges in
    // the forward filtration (earlier columns in the reverse computation).
    edge: EdgePosition,
    additions: Vec<EdgePosition>,
    #[cfg(test)]
    reduced: Vec<SimplexEntry>,
}

/// Instrumentation is compiled out of production builds.
#[derive(Default, Debug)]
struct Stats {
    #[cfg(test)]
    edges: usize,
    #[cfg(test)]
    cofacets: usize,
    #[cfg(test)]
    initial_candidates: usize,
    #[cfg(test)]
    reconstruction_candidates: usize,
    #[cfg(test)]
    column_additions: usize,
    #[cfg(test)]
    shortcuts: usize,
    #[cfg(test)]
    peak_heap: usize,
    #[cfg(test)]
    stored_entries: usize,
    #[cfg(test)]
    largest_transform: usize,
    #[cfg(test)]
    peak_transform_heap: usize,
}

pub(super) fn compute(rips: &impl FlagAccess, budget: &mut WorkBudget<'_>) -> Result<RawIntervals> {
    run_access::<true, true, 3>(rips, &mut Stats::default(), budget)
}

// Retain independent optimization configurations for the dense test oracle.
#[cfg(test)]
fn run<const IMPLICIT: bool, const CLEAR: bool, const CONE: bool, const SHORTCUTS: u8>(
    input: DissimilarityView<'_>,
    cutoff: f64,
    stats: &mut Stats,
) -> Result<RawIntervals> {
    let mut budget = WorkBudget::new(&crate::persistence::ExecutionLimits::default())?;
    let stop = if CONE {
        cutoff.min(cone_radius(input.into(), &mut || budget.step())?)
    } else {
        cutoff
    };
    run_access::<IMPLICIT, CLEAR, SHORTCUTS>(
        &DenseFlag::new(input.into(), stop)?,
        stats,
        &mut budget,
    )
}

fn run_access<const IMPLICIT: bool, const CLEAR: bool, const SHORTCUTS: u8>(
    rips: &impl FlagAccess,
    stats: &mut Stats,
    budget: &mut WorkBudget<'_>,
) -> Result<RawIntervals> {
    let edges = rips.edges(&mut || budget.step())?;
    budget.check()?;
    #[cfg(test)]
    {
        stats.edges = edges.len();
    }
    // Classify edges in forward filtration order. Clearing and the reverse
    // reduction below must use this same ordering, including ties.
    let mut forest = UnionFind::new(rips.vertex_count())?;
    let mut cycle_edges = Vec::new();
    cycle_edges
        .try_reserve_exact(edges.len())
        .map_err(|_| allocation("Rips cycle edges"))?;
    let mut raw = Vec::new();
    // H0 contributes exactly n intervals before removing zero bars. Grow for
    // actual H1 output rather than reserving space for zero-lifetime pairs.
    raw.try_reserve(rips.vertex_count())
        .map_err(|_| allocation("Rips intervals"))?;
    for edge in &edges {
        budget.step()?;
        let [a, b] = rips.edge_vertices(edge.id);
        let merged = forest.merge(a, b);
        cycle_edges.push(!merged);
        if merged {
            raw.push((0, 0.0, Some(edge.value)));
        }
    }
    raw.extend((0..forest.components()).map(|_| (0, 0.0, None)));

    // Map triangle ids to stored transformation columns; edge and
    // additions in those columns are positions in edges, not simplex ids.
    let mut pivot_owners: HashMap<usize, ColumnPosition> = HashMap::new();
    let mut columns: Vec<TransformColumn> = Vec::new();
    let mut working = Coboundary::new();
    let mut transform = BinaryHeap::new();
    for j in (0..edges.len()).rev() {
        budget.step()?;
        if CLEAR && !cycle_edges[j] {
            continue;
        }
        working.clear();
        transform.clear();
        let edge = edges[j];
        let shortcut = initialize_coboundary::<SHORTCUTS>(
            rips,
            edge,
            &pivot_owners,
            &mut working,
            stats,
            budget,
        )?;
        let pivot = if let Some(pivot) = shortcut {
            Some(pivot)
        } else {
            loop {
                budget.step()?;
                let Some(Reverse(pivot)) = pop_parity(&mut working, budget)? else {
                    break None;
                };
                let Some(&owner) = pivot_owners.get(&pivot.id) else {
                    break Some(pivot);
                };
                // Put the pivot back: adding the owner's column cancels it.
                push_heap(&mut working, Reverse(pivot))?;
                let column = &columns[owner.0];
                #[cfg(test)]
                {
                    stats.column_additions += 1;
                }
                if IMPLICIT {
                    append_coboundary(rips, edges[column.edge.0], &mut working, stats, budget)?;
                    push_heap(&mut transform, column.edge)?;
                    for &k in &column.additions {
                        append_coboundary(rips, edges[k.0], &mut working, stats, budget)?;
                        push_heap(&mut transform, k)?;
                    }
                } else {
                    #[cfg(test)]
                    for &row in &column.reduced {
                        push_heap(&mut working, Reverse(row))?;
                    }
                }
                #[cfg(test)]
                {
                    stats.peak_heap = stats.peak_heap.max(working.len());
                    stats.peak_transform_heap = stats.peak_transform_heap.max(transform.len());
                }
            }
        };
        if let Some(pivot) = pivot {
            if !cycle_edges[j] {
                return Err(Error::InternalInvariant {
                    reason: "H0 death edge paired in H1",
                });
            }
            let mut additions = Vec::new();
            while let Some(k) = pop_parity(&mut transform, budget)? {
                additions
                    .try_reserve(1)
                    .map_err(|_| allocation("Rips transform column"))?;
                additions.push(k);
            }
            #[cfg(test)]
            let mut reduced = Vec::new();
            #[cfg(test)]
            if !IMPLICIT {
                if shortcut.is_some() {
                    append_coboundary(rips, edge, &mut working, stats, budget)?;
                } else {
                    push_heap(&mut working, Reverse(pivot))?;
                }
                while let Some(Reverse(row)) = pop_parity(&mut working, budget)? {
                    reduced.push(row);
                }
            }
            #[cfg(test)]
            {
                stats.stored_entries += additions.len() + reduced.len();
                stats.largest_transform = stats.largest_transform.max(additions.len());
            }
            pivot_owners
                .try_reserve(1)
                .map_err(|_| allocation("Rips pivot owners"))?;
            columns
                .try_reserve(1)
                .map_err(|_| allocation("Rips transform columns"))?;
            pivot_owners.insert(pivot.id, ColumnPosition(columns.len()));
            columns.push(TransformColumn {
                edge: EdgePosition(j),
                additions,
                #[cfg(test)]
                reduced,
            });
            // Keep the pivot and transformation even when this public bar is
            // empty: later columns can still need them for cancellation.
            if edge.value != pivot.value {
                raw.try_reserve(1)
                    .map_err(|_| allocation("Rips intervals"))?;
                raw.push((1, edge.value, Some(pivot.value)));
            }
        } else if cycle_edges[j] {
            raw.try_reserve(1)
                .map_err(|_| allocation("Rips intervals"))?;
            raw.push((1, edge.value, None));
        }
    }
    Ok(raw)
}

/// Initialize the original column in one scan, stopping at a valid shortcut.
fn initialize_coboundary<const SHORTCUTS: u8>(
    rips: &impl FlagAccess,
    edge: SimplexEntry,
    pivot_owners: &HashMap<usize, ColumnPosition>,
    working: &mut Coboundary,
    stats: &mut Stats,
    budget: &mut WorkBudget<'_>,
) -> Result<Option<SimplexEntry>> {
    // Recover the existing heap allocation as an unsorted scratch buffer.
    let mut rows = std::mem::take(working).into_vec();
    rows.clear();
    let mut found = None;
    let mut check_shortcut = SHORTCUTS != 0;
    #[cfg(test)]
    let mut candidates = 0;
    // Both access implementations visit triangles in decreasing combinatorial
    // ID order. The first equal-valued cofacet is the original column pivot.
    rips.visit_cofacets(
        edge,
        &mut || {
            #[cfg(test)]
            {
                candidates += 1;
            }
            budget.step()
        },
        |triangle| {
            count_cofacet(stats);
            if check_shortcut && triangle.value == edge.value {
                // An occupied first equal-valued cofacet requires ordinary
                // reduction; never try a later equal-valued cofacet instead.
                check_shortcut = false;
                let eligible = SHORTCUTS & 2 != 0
                    || (SHORTCUTS & 1 != 0 && rips.latest_facet(triangle) == edge);
                if eligible && !pivot_owners.contains_key(&triangle.id) {
                    #[cfg(test)]
                    {
                        stats.shortcuts += 1;
                    }
                    found = Some(triangle);
                    return Ok(false);
                }
            }
            rows.try_reserve(1)
                .map_err(|_| allocation("Rips working heap"))?;
            rows.push(Reverse(triangle));
            Ok(true)
        },
    )?;
    #[cfg(test)]
    {
        stats.initial_candidates += candidates;
    }
    if found.is_some() {
        rows.clear();
    }
    // Heap construction, like sorting, is checked at its phase boundaries.
    budget.check()?;
    *working = BinaryHeap::from(rows);
    budget.check()?;
    #[cfg(test)]
    {
        stats.peak_heap = stats.peak_heap.max(working.len());
    }
    Ok(found)
}

fn allocation(context: &'static str) -> Error {
    Error::AllocationFailed { context }
}

fn push_heap<T: Ord>(heap: &mut BinaryHeap<T>, value: T) -> Result<()> {
    heap.try_reserve(1)
        .map_err(|_| allocation("Rips working heap"))?;
    heap.push(value);
    Ok(())
}

/// F2 cancellation, including repeated entries introduced by multiple additions.
fn pop_parity<T: Ord + Copy>(
    heap: &mut BinaryHeap<T>,
    budget: &mut WorkBudget<'_>,
) -> Result<Option<T>> {
    while !heap.is_empty() {
        budget.step()?;
        // Nonempty heap established above; count before removing the entry.
        let value = heap.pop().unwrap();
        let mut odd = true;
        while heap.peek() == Some(&value) {
            budget.step()?;
            heap.pop();
            odd = !odd;
        }
        if odd {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn count_cofacet(_stats: &mut Stats) {
    #[cfg(test)]
    {
        _stats.cofacets += 1;
    }
}

fn append_coboundary(
    rips: &impl FlagAccess,
    edge: SimplexEntry,
    heap: &mut Coboundary,
    stats: &mut Stats,
    budget: &mut WorkBudget<'_>,
) -> Result<()> {
    #[cfg(test)]
    let mut candidates = 0;
    rips.visit_cofacets(
        edge,
        &mut || {
            #[cfg(test)]
            {
                candidates += 1;
            }
            budget.step()
        },
        |row| {
            count_cofacet(stats);
            push_heap(heap, Reverse(row))?;
            Ok(true)
        },
    )?;
    #[cfg(test)]
    {
        stats.reconstruction_candidates += candidates;
        stats.peak_heap = stats.peak_heap.max(heap.len());
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod profiling;
