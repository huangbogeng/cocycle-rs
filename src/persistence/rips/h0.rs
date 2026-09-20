//! H0-only Rips persistence by sorted edges and component merges.

use super::union_find::UnionFind;
use crate::geometry::DissimilarityView;
use crate::{Error, Result};

/// Independent Kruskal/union-find path; no boundary matrix or simplex representation.
pub(super) fn compute(
    input: DissimilarityView<'_>,
    cutoff: f64,
) -> Result<Vec<(usize, f64, Option<f64>)>> {
    let n = input.len();
    let mut edges = Vec::new();
    for i in 0..n {
        for j in 0..i {
            let weight = input.get(i, j).ok_or(Error::InternalInvariant {
                reason: "H0 distance index out of bounds",
            })?;
            if weight <= cutoff {
                edges.try_reserve(1).map_err(|_| Error::AllocationFailed {
                    context: "H0 edges",
                })?;
                edges.push((weight, j, i));
            }
        }
    }
    edges.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut forest = UnionFind::new(n)?;
    let mut raw = Vec::new();
    raw.try_reserve(n).map_err(|_| Error::AllocationFailed {
        context: "H0 intervals",
    })?;
    for (weight, a, b) in edges {
        if !forest.merge(a, b) {
            continue;
        }
        raw.push((0, 0.0, Some(weight)));
        if forest.components() == 1 {
            break;
        }
    }
    raw.extend((0..forest.components()).map(|_| (0, 0.0, None)));
    Ok(raw)
}
