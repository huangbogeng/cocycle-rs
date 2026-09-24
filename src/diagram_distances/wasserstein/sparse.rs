//! Primal-dual sparse shortest augmenting paths with two residual layouts.
//!
//! Adapted from Topp; the copyright and complete MIT notice are in the parent
//! module. Both layouts share edge order, capacities, potentials and search.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::{Graph, SparseLayout, Stats, allocation, buffer, finite, numerical, push, sum_size};
use crate::{Error, Result};

#[derive(Clone, Copy, Default)]
struct Residual {
    destination: usize,
    reverse: usize,
    capacity: usize,
    cost: f64,
}

enum Network {
    Vectors(Vec<Vec<Residual>>),
    Arena {
        offsets: Vec<usize>,
        edges: Vec<Residual>,
    },
}

impl Network {
    fn new(nodes: usize, degree: &[usize], layout: SparseLayout) -> Result<Self> {
        match layout {
            SparseLayout::Vectors => Ok(Self::Vectors(buffer(nodes, Vec::new())?)),
            SparseLayout::Arena => {
                let mut offsets = Vec::new();
                offsets
                    .try_reserve_exact(sum_size(degree.len(), 1)?)
                    .map_err(|_| allocation())?;
                let mut total = 0;
                offsets.push(0);
                for &count in degree {
                    total = sum_size(total, count)?;
                    offsets.push(total);
                }
                Ok(Self::Arena {
                    offsets,
                    edges: buffer(total, Residual::default())?,
                })
            }
        }
    }

    fn insert(
        &mut self,
        cursor: &mut [usize],
        source: usize,
        destination: usize,
        cost: f64,
        capacity: usize,
    ) -> Result<()> {
        let forward = Residual {
            destination,
            reverse: cursor[destination],
            capacity,
            cost,
        };
        let backward = Residual {
            destination: source,
            reverse: cursor[source],
            capacity: 0,
            cost: -cost,
        };
        match self {
            Self::Vectors(rows) => {
                push(&mut rows[source], forward)?;
                push(&mut rows[destination], backward)?;
            }
            Self::Arena { offsets, edges } => {
                edges[offsets[source] + cursor[source]] = forward;
                edges[offsets[destination] + cursor[destination]] = backward;
            }
        }
        cursor[source] += 1;
        cursor[destination] += 1;
        Ok(())
    }

    fn edges(&self, node: usize) -> &[Residual] {
        match self {
            Self::Vectors(rows) => &rows[node],
            Self::Arena { offsets, edges } => &edges[offsets[node]..offsets[node + 1]],
        }
    }

    fn edge_mut(&mut self, node: usize, edge: usize) -> &mut Residual {
        match self {
            Self::Vectors(rows) => &mut rows[node][edge],
            Self::Arena { offsets, edges } => &mut edges[offsets[node] + edge],
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct QueueItem {
    distance: f64,
    node: usize,
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance && self.node == other.node
    }
}
impl Eq for QueueItem {}
impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .distance
            .total_cmp(&self.distance)
            .then_with(|| other.node.cmp(&self.node))
    }
}

struct Scratch {
    distance: Vec<f64>,
    previous_node: Vec<usize>,
    previous_edge: Vec<usize>,
    heap: BinaryHeap<QueueItem>,
}

impl Scratch {
    fn new(nodes: usize) -> Result<Self> {
        Ok(Self {
            distance: buffer(nodes, f64::INFINITY)?,
            previous_node: buffer(nodes, usize::MAX)?,
            previous_edge: buffer(nodes, usize::MAX)?,
            heap: BinaryHeap::new(),
        })
    }

    fn reset(&mut self) {
        self.distance.fill(f64::INFINITY);
        self.previous_node.fill(usize::MAX);
        self.previous_edge.fill(usize::MAX);
        self.heap.clear();
    }

    fn enqueue(&mut self, distance: f64, node: usize) -> Result<()> {
        self.heap.try_reserve(1).map_err(|_| allocation())?;
        self.heap.push(QueueItem { distance, node });
        Ok(())
    }
}

pub(super) fn solve(
    graph: &Graph,
    capacities: Option<(&[usize], &[usize])>,
    layout: SparseLayout,
    stats: &mut Stats,
) -> Result<Vec<(usize, usize, usize)>> {
    stats.sparse_solves += 1;
    let (rows, columns) = graph.active()?;
    if rows.is_empty() || columns.is_empty() {
        return Ok(Vec::new());
    }
    let row_capacity = |row: usize| capacities.map_or(1, |(r, _)| r[row]);
    let column_capacity = |column: usize| capacities.map_or(1, |(_, c)| c[column]);
    let row_base = 1;
    let column_base = sum_size(row_base, rows.len())?;
    let sink = sum_size(column_base, columns.len())?;
    let nodes = sum_size(sink, 1)?;
    let mut row_map = buffer(graph.rows, usize::MAX)?;
    let mut column_map = buffer(graph.columns, usize::MAX)?;
    for (i, &row) in rows.iter().enumerate() {
        row_map[row] = i;
    }
    for (i, &column) in columns.iter().enumerate() {
        column_map[column] = i;
    }
    let mut degree = Vec::new();
    if layout == SparseLayout::Arena {
        degree = buffer(nodes, 0_usize)?;
        degree[0] = rows.len();
        degree[sink] = columns.len();
        for &row in &rows {
            let node = row_base + row_map[row];
            degree[node] = 1;
            for edge in graph.edges(row) {
                degree[node] = sum_size(degree[node], 1)?;
                let other = column_base + column_map[edge.column];
                degree[other] = sum_size(degree[other], 1)?;
            }
        }
        for i in 0..columns.len() {
            degree[column_base + i] = sum_size(degree[column_base + i], 1)?;
        }
    }
    let mut network = Network::new(nodes, &degree, layout)?;
    let mut cursor = buffer(nodes, 0)?;
    for (i, &row) in rows.iter().enumerate() {
        network.insert(&mut cursor, 0, row_base + i, 0.0, row_capacity(row))?;
    }
    for (i, &column) in columns.iter().enumerate() {
        network.insert(
            &mut cursor,
            column_base + i,
            sink,
            0.0,
            column_capacity(column),
        )?;
    }
    let mut potential = buffer(nodes, 0.0_f64)?;
    for &row in &rows {
        let node = row_base + row_map[row];
        for edge in graph.edges(row) {
            let other = column_base + column_map[edge.column];
            network.insert(
                &mut cursor,
                node,
                other,
                -edge.saving,
                row_capacity(row).min(column_capacity(edge.column)),
            )?;
            potential[other] = potential[other].min(-edge.saving);
        }
    }
    potential[sink] = potential[column_base..sink]
        .iter()
        .copied()
        .fold(0.0, f64::min);
    // The baseline allocates search buffers per augmentation. Arena mode keeps
    // their capacities, including the heap, throughout this one matching call.
    let mut scratch = Scratch::new(nodes)?;
    if let Network::Arena { edges, .. } = &network {
        // Match the pinned arena experiment's bounded initial heap reservation.
        // Both capacity and reuse belong to this layout ablation, not R0.
        let budget = nodes.checked_mul(8).ok_or(Error::SizeOverflow {
            operation: "Wasserstein arena heap reservation",
        })?;
        scratch
            .heap
            .try_reserve_exact(edges.len().min(budget))
            .map_err(|_| allocation())?;
    }
    let mut iteration = 0;
    loop {
        if iteration != 0 {
            if layout == SparseLayout::Arena {
                scratch.reset();
                stats.scratch_reuses += 1;
            } else {
                scratch = Scratch::new(nodes)?;
            }
        }
        iteration += 1;
        scratch.distance[0] = 0.0;
        scratch.enqueue(0.0, 0)?;
        while let Some(QueueItem { distance, node }) = scratch.heap.pop() {
            if distance != scratch.distance[node] {
                continue;
            }
            for (index, edge) in network.edges(node).iter().enumerate() {
                if edge.capacity == 0 {
                    continue;
                }
                let raw = finite(edge.cost + potential[node] - potential[edge.destination])?;
                let magnitude = edge
                    .cost
                    .abs()
                    .max(potential[node].abs())
                    .max(potential[edge.destination].abs());
                // A negative rounding residue is harmless; a materially invalid
                // dual cannot be repaired by silently treating its edge as free.
                if raw < -64.0 * f64::EPSILON * magnitude {
                    return Err(numerical());
                }
                let candidate = finite(distance + raw.max(0.0))?;
                if candidate >= scratch.distance[edge.destination] {
                    continue;
                }
                scratch.distance[edge.destination] = candidate;
                scratch.previous_node[edge.destination] = node;
                scratch.previous_edge[edge.destination] = index;
                scratch.enqueue(candidate, edge.destination)?;
            }
        }
        if scratch.previous_node[sink] == usize::MAX {
            break;
        }
        let path_cost = finite(scratch.distance[sink] - potential[0] + potential[sink])?;
        if path_cost >= 0.0 {
            break;
        }
        for (node, value) in potential.iter_mut().enumerate() {
            if scratch.distance[node].is_finite() {
                *value = finite(*value + scratch.distance[node])?;
            }
        }
        let mut amount = usize::MAX;
        let mut node = sink;
        let mut hops = 0;
        while node != 0 {
            let parent = scratch.previous_node[node];
            if parent == usize::MAX || hops >= nodes {
                return Err(Error::InternalInvariant {
                    reason: "invalid Wasserstein augmenting path",
                });
            }
            amount = amount.min(network.edges(parent)[scratch.previous_edge[node]].capacity);
            node = parent;
            hops += 1;
        }
        node = sink;
        while node != 0 {
            let parent = scratch.previous_node[node];
            let index = scratch.previous_edge[node];
            let edge = network.edge_mut(parent, index);
            edge.capacity -= amount;
            let reverse = edge.reverse;
            let other = network.edge_mut(node, reverse);
            other.capacity = sum_size(other.capacity, amount)?;
            node = parent;
        }
        stats.augmentations += 1;
    }
    let mut flows = Vec::new();
    for (local, &row) in rows.iter().enumerate() {
        for edge in network.edges(row_base + local) {
            if edge.destination < column_base || edge.destination >= sink {
                continue;
            }
            let count = network.edges(edge.destination)[edge.reverse].capacity;
            if count > 0 {
                push(
                    &mut flows,
                    (row, columns[edge.destination - column_base], count),
                )?;
            }
        }
    }
    Ok(flows)
}
