//! F2 Rips H1 via implicit coboundaries and stored change-of-basis columns.
//! Independently implemented from the invariants in mathematics.md section 9;
//! the explicit boundary reducer remains an independent test oracle.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::h0::UnionFind;
use crate::filtration::rips::{Entry, Rips, cone_radius};
use crate::geometry::DissimilarityView;
use crate::{Error, Result};

type RawIntervals = Vec<(usize, f64, Option<f64>)>;
type Coboundary = BinaryHeap<Reverse<Entry>>;

struct Column {
    // The diagonal entry of V is implicit. Other entries index later edges in
    // the forward filtration (earlier columns in the reverse computation).
    edge: usize,
    additions: Vec<usize>,
    #[cfg(test)]
    reduced: Vec<Entry>,
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
    let rips = Rips::new(input, stop)?;
    let edges = rips.edges()?;
    #[cfg(test)]
    {
        stats.edges = edges.len();
    }
    let mut forest = UnionFind::new(input.len())?;
    let mut cycle = Vec::new();
    cycle
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
        cycle.push(!merged);
        if merged {
            raw.push((0, 0.0, Some(edge.value)));
        }
    }
    raw.extend((0..forest.components).map(|_| (0, 0.0, None)));

    let mut owners: HashMap<usize, usize> = HashMap::new();
    let mut columns: Vec<Column> = Vec::new();
    let mut working = Coboundary::new();
    let mut transform = BinaryHeap::new();
    for j in (0..edges.len()).rev() {
        if CLEAR && !cycle[j] {
            continue;
        }
        working.clear();
        transform.clear();
        let edge = edges[j];
        let mut shortcut = None;
        if SHORTCUTS != 0 {
            // Cofacets have nonincreasing ids. The first with value=birth is
            // the earliest cofacet in the filtration. No later item can beat it.
            for triangle in rips.cofacets(edge) {
                count_cofacet(stats);
                if triangle.value == edge.value {
                    let apparent = SHORTCUTS & 1 != 0 && rips.latest_facet(triangle) == edge;
                    let emergent = SHORTCUTS & 2 != 0;
                    if (apparent || emergent) && !owners.contains_key(&triangle.id) {
                        shortcut = Some(triangle);
                        #[cfg(test)]
                        {
                            stats.shortcuts += 1;
                        }
                    }
                    // Even if occupied, a later equal-valued triangle is NOT
                    // the pivot of this original column. Fall back to reduction.
                    break;
                }
            }
        }
        let pivot = if let Some(pivot) = shortcut {
            Some(pivot)
        } else {
            append_coboundary(&rips, edge, &mut working, stats)?;
            loop {
                let Some(Reverse(pivot)) = pop_parity(&mut working) else {
                    break None;
                };
                let Some(&owner) = owners.get(&pivot.id) else {
                    break Some(pivot);
                };
                // Put the pivot back: adding the owner's column cancels it.
                push_heap(&mut working, Reverse(pivot))?;
                let column = &columns[owner];
                #[cfg(test)]
                {
                    stats.column_additions += 1;
                }
                if IMPLICIT {
                    append_coboundary(&rips, edges[column.edge], &mut working, stats)?;
                    push_heap(&mut transform, column.edge)?;
                    for &k in &column.additions {
                        append_coboundary(&rips, edges[k], &mut working, stats)?;
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
            if !cycle[j] {
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
            owners
                .try_reserve(1)
                .map_err(|_| allocation("Rips pivot owners"))?;
            columns
                .try_reserve(1)
                .map_err(|_| allocation("Rips transform columns"))?;
            owners.insert(pivot.id, columns.len());
            columns.push(Column {
                edge: j,
                additions,
                #[cfg(test)]
                reduced,
            });
            raw.push((1, edge.value, Some(pivot.value)));
        } else if cycle[j] {
            raw.push((1, edge.value, None));
        }
    }
    Ok(raw)
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
    rips: &Rips<'_>,
    edge: Entry,
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
