//! Compact sorted adjacency construction; no dense matrix or reducer state.
use super::{Neighbor, WeightedEdge};
use crate::{Error, Result};

pub(super) fn build(n: usize, edges: &[WeightedEdge]) -> Result<(Vec<usize>, Vec<Neighbor>)> {
    let count = n.checked_add(1).ok_or(Error::SizeOverflow {
        operation: "graph offsets",
    })?;
    let entries = edges.len().checked_mul(2).ok_or(Error::SizeOverflow {
        operation: "graph adjacency",
    })?;
    let mut offsets = Vec::new();
    offsets
        .try_reserve_exact(count)
        .map_err(|_| Error::AllocationFailed {
            context: "graph offsets",
        })?;
    offsets.resize(count, 0usize);
    for edge in edges {
        for v in edge.vertices {
            offsets[v + 1] += 1;
        }
    }
    for v in 0..n {
        offsets[v + 1] += offsets[v];
    }
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(n)
        .map_err(|_| Error::AllocationFailed {
            context: "adjacency positions",
        })?;
    positions.extend_from_slice(&offsets[..n]);
    let mut neighbors = Vec::new();
    neighbors
        .try_reserve_exact(entries)
        .map_err(|_| Error::AllocationFailed {
            context: "graph neighbors",
        })?;
    neighbors.resize(
        entries,
        Neighbor {
            vertex: 0,
            value: 0.0,
        },
    );
    for edge in edges {
        let [a, b] = edge.vertices;
        for (v, u) in [(a, b), (b, a)] {
            neighbors[positions[v]] = Neighbor {
                vertex: u,
                value: edge.value,
            };
            positions[v] += 1;
        }
    }
    for v in 0..n {
        neighbors[offsets[v]..offsets[v + 1]].sort_unstable_by_key(|x| x.vertex);
    }
    Ok((offsets, neighbors))
}
