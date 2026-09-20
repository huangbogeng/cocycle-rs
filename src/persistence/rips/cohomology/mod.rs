//! F2 Rips H1 via implicit coboundaries and stored change-of-basis columns.
//! Independently implemented from the invariants in docs/reference/mathematics.md section 9;
//! the explicit boundary reducer remains an independent test oracle.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::union_find::UnionFind;
use crate::filtration::rips::{RipsFiltration, SimplexEntry, cone_radius};
use crate::geometry::DissimilarityView;
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

pub(super) fn compute(input: DissimilarityView<'_>, cutoff: f64) -> Result<RawIntervals> {
    run::<true, true, true, 3>(input, cutoff, &mut Stats::default())
}

// Private compile-time switches let tests independently remove optimizations.
// SHORTCUTS: bit 0 apparent pairs, bit 1 emergent pairs on an original column.
fn run<const IMPLICIT: bool, const CLEAR: bool, const CONE: bool, const SHORTCUTS: u8>(
    input: DissimilarityView<'_>,
    cutoff: f64,
    stats: &mut Stats,
) -> Result<RawIntervals> {
    let stop = if CONE {
        cutoff.min(cone_radius(input))
    } else {
        cutoff
    };
    let rips = RipsFiltration::new(input, stop)?;
    let edges = rips.edges()?;
    #[cfg(test)]
    {
        stats.edges = edges.len();
    }
    // Classify edges in forward filtration order. Clearing and the reverse
    // reduction below must use this same ordering, including ties.
    let mut forest = UnionFind::new(input.len())?;
    let mut cycle_edges = Vec::new();
    cycle_edges
        .try_reserve_exact(edges.len())
        .map_err(|_| allocation("Rips cycle edges"))?;
    let mut raw = Vec::new();
    // H0 contributes exactly n intervals before removing zero bars. Every
    // remaining output comes from one of at most m cycle edges.
    let capacity = input
        .len()
        .checked_add(edges.len())
        .ok_or(Error::SizeOverflow {
            operation: "Rips interval capacity",
        })?;
    raw.try_reserve(capacity)
        .map_err(|_| allocation("Rips intervals"))?;
    for edge in &edges {
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
        if CLEAR && !cycle_edges[j] {
            continue;
        }
        working.clear();
        transform.clear();
        let edge = edges[j];
        let shortcut = find_shortcut::<SHORTCUTS>(&rips, edge, &pivot_owners, stats);
        let pivot = if let Some(pivot) = shortcut {
            Some(pivot)
        } else {
            append_coboundary(&rips, edge, &mut working, stats)?;
            loop {
                let Some(Reverse(pivot)) = pop_parity(&mut working) else {
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
                    append_coboundary(&rips, edges[column.edge.0], &mut working, stats)?;
                    push_heap(&mut transform, column.edge)?;
                    for &k in &column.additions {
                        append_coboundary(&rips, edges[k.0], &mut working, stats)?;
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
            while let Some(k) = pop_parity(&mut transform) {
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
                    append_coboundary(&rips, edge, &mut working, stats)?;
                } else {
                    push_heap(&mut working, Reverse(pivot))?;
                }
                while let Some(Reverse(row)) = pop_parity(&mut working) {
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
            raw.push((1, edge.value, Some(pivot.value)));
        } else if cycle_edges[j] {
            raw.push((1, edge.value, None));
        }
    }
    Ok(raw)
}

/// Search only the original edge column, before any column additions.
fn find_shortcut<const SHORTCUTS: u8>(
    rips: &RipsFiltration<'_>,
    edge: SimplexEntry,
    pivot_owners: &HashMap<usize, ColumnPosition>,
    stats: &mut Stats,
) -> Option<SimplexEntry> {
    if SHORTCUTS == 0 {
        return None;
    }
    // Cofacets have nonincreasing ids. The first with value=birth is
    // the earliest cofacet in the filtration. No later item can beat it.
    for triangle in rips.cofacets(edge) {
        count_cofacet(stats);
        if triangle.value == edge.value {
            let apparent = SHORTCUTS & 1 != 0 && rips.latest_facet(triangle) == edge;
            let emergent = SHORTCUTS & 2 != 0;
            if (apparent || emergent) && !pivot_owners.contains_key(&triangle.id) {
                #[cfg(test)]
                {
                    stats.shortcuts += 1;
                }
                return Some(triangle);
            }
            // Even if occupied, a later equal-valued triangle is NOT
            // the pivot of this original column. Fall back to reduction.
            return None;
        }
    }
    None
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
fn pop_parity<T: Ord + Copy>(heap: &mut BinaryHeap<T>) -> Option<T> {
    while let Some(value) = heap.pop() {
        let mut odd = true;
        while heap.peek() == Some(&value) {
            heap.pop();
            odd = !odd;
        }
        if odd {
            return Some(value);
        }
    }
    None
}

fn count_cofacet(_stats: &mut Stats) {
    #[cfg(test)]
    {
        _stats.cofacets += 1;
    }
}

fn append_coboundary(
    rips: &RipsFiltration<'_>,
    edge: SimplexEntry,
    heap: &mut Coboundary,
    stats: &mut Stats,
) -> Result<()> {
    for row in rips.cofacets(edge) {
        count_cofacet(stats);
        push_heap(heap, Reverse(row))?;
    }
    #[cfg(test)]
    {
        stats.peak_heap = stats.peak_heap.max(heap.len());
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod profiling;
