//! Finite weighted graphs with explicit isolated vertices.

mod adjacency;
use crate::geometry::distance::nonnegative;
use crate::{Error, Result};

/// An undirected weighted edge. Raw fields are validated by [`WeightedGraph::new`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeightedEdge {
    /// Endpoints; input order is immaterial.
    pub vertices: [usize; 2],
    /// Finite nonnegative edge filtration value.
    pub value: f64,
}

/// One neighbor, sorted by vertex index within each graph adjacency slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Neighbor {
    /// Original vertex index.
    pub vertex: usize,
    /// Edge filtration value.
    pub value: f64,
}

/// An owned undirected graph on vertices `0..vertex_count`.
///
/// Vertices have filtration value zero. Missing edges are absent, not implicitly
/// zero. This storage establishes no relationship to distances of an unseen
/// point cloud. Construction normalizes endpoint order and sorts edges
/// lexicographically, retaining isolated vertices and zero-weight edges.
#[derive(Clone, Debug)]
pub struct WeightedGraph {
    edges: Vec<WeightedEdge>,
    offsets: Vec<usize>,
    neighbors: Vec<Neighbor>,
    max_edge: f64,
}

impl WeightedGraph {
    /// Consume and validate edges; construct sorted adjacency in O(n + m log m)
    /// time and O(n + m) storage. The caller's vector is reused for sorted edges.
    ///
    /// # Errors
    /// Rejects out-of-range endpoints, self-loops, repeated undirected edges,
    /// invalid values, arithmetic overflow or a failed fallible reservation.
    pub fn new(vertex_count: usize, mut edges: Vec<WeightedEdge>) -> Result<Self> {
        let mut max_edge: f64 = 0.0;
        for (index, edge) in edges.iter_mut().enumerate() {
            let [a, b] = edge.vertices;
            if a >= vertex_count || b >= vertex_count {
                return Err(Error::InvalidGraph {
                    edge: index,
                    reason: "endpoint outside vertex range",
                });
            }
            if a == b {
                return Err(Error::InvalidGraph {
                    edge: index,
                    reason: "self-loop",
                });
            }
            edge.vertices.sort_unstable();
            edge.value = nonnegative(edge.value, "edge", Some(index))?;
            max_edge = max_edge.max(edge.value);
        }
        edges.sort_unstable_by_key(|e| e.vertices);
        for (index, pair) in edges.windows(2).enumerate() {
            if pair[0].vertices == pair[1].vertices {
                return Err(Error::InvalidGraph {
                    edge: index + 1,
                    reason: "duplicate edge in sorted input",
                });
            }
        }
        let (offsets, neighbors) = adjacency::build(vertex_count, &edges)?;
        Ok(Self {
            edges,
            offsets,
            neighbors,
            max_edge,
        })
    }
    /// Number of vertices, including isolated ones.
    pub fn vertex_count(&self) -> usize {
        self.offsets.len() - 1
    }
    /// Number of undirected edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    /// Edges with increasing endpoints, in lexicographic endpoint order.
    pub fn edges(&self) -> &[WeightedEdge] {
        &self.edges
    }
    /// Sorted neighbors, or `None` for an invalid vertex.
    pub fn neighbors(&self, vertex: usize) -> Option<&[Neighbor]> {
        if vertex >= self.vertex_count() {
            return None;
        }
        Some(&self.neighbors[self.offsets[vertex]..self.offsets[vertex + 1]])
    }
    /// Edge weight, or `None` for a missing edge, diagonal, or invalid index.
    pub fn edge_value(&self, a: usize, b: usize) -> Option<f64> {
        let neighbors = self.neighbors(a)?;
        neighbors
            .binary_search_by_key(&b, |v| v.vertex)
            .ok()
            .map(|i| neighbors[i].value)
    }
    /// Largest stored edge value, or zero when there are no edges.
    pub fn max_edge(&self) -> f64 {
        self.max_edge
    }
}
