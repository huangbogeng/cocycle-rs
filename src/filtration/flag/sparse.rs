//! Sparse cofacets by reverse sorted neighbor intersection.
use super::index::SimplexIndex;
use super::{FlagAccess, SimplexEntry};
use crate::complex::WeightedGraph;
use crate::{Error, Result};

pub(crate) struct SparseFlag<'a> {
    graph: &'a WeightedGraph,
    index: SimplexIndex,
    cutoff: f64,
}
impl<'a> SparseFlag<'a> {
    pub(crate) fn new(graph: &'a WeightedGraph, cutoff: f64) -> Result<Self> {
        Ok(Self {
            graph,
            index: SimplexIndex::new(graph.vertex_count())?,
            cutoff,
        })
    }
    fn edge(&self, a: usize, b: usize) -> SimplexEntry {
        // Called only for facets of triangles produced by neighbor intersection.
        SimplexEntry {
            id: self.index.edge(a, b),
            value: self.graph.edge_value(a, b).unwrap(),
        }
    }
}
impl FlagAccess for SparseFlag<'_> {
    fn vertex_count(&self) -> usize {
        self.graph.vertex_count()
    }
    fn edges(&self, checkpoint: &mut impl FnMut() -> Result<()>) -> Result<Vec<SimplexEntry>> {
        let mut edges = Vec::new();
        for edge in self.graph.edges() {
            checkpoint()?;
            if edge.value <= self.cutoff {
                edges.try_reserve(1).map_err(|_| Error::AllocationFailed {
                    context: "sparse flag edges",
                })?;
                edges.push(SimplexEntry {
                    id: self.index.edge(edge.vertices[0], edge.vertices[1]),
                    value: edge.value,
                });
            }
        }
        edges.sort_unstable();
        Ok(edges)
    }
    fn edge_vertices(&self, id: usize) -> [usize; 2] {
        self.index.edge_vertices(id)
    }
    fn latest_facet(&self, triangle: SimplexEntry) -> SimplexEntry {
        let [a, b, c] = self.index.triangle_vertices(triangle.id);
        self.edge(a, b).max(self.edge(a, c)).max(self.edge(b, c))
    }
    fn visit_cofacets(
        &self,
        edge: SimplexEntry,
        checkpoint: &mut impl FnMut() -> Result<()>,
        mut visitor: impl FnMut(SimplexEntry) -> Result<bool>,
    ) -> Result<()> {
        let [a, b] = self.edge_vertices(edge.id);
        let left = self.graph.neighbors(a).unwrap();
        let right = self.graph.neighbors(b).unwrap();
        let (mut i, mut j) = (left.len(), right.len());
        while i > 0 && j > 0 {
            checkpoint()?;
            let (x, y) = (left[i - 1], right[j - 1]);
            match x.vertex.cmp(&y.vertex) {
                std::cmp::Ordering::Greater => i -= 1,
                std::cmp::Ordering::Less => j -= 1,
                std::cmp::Ordering::Equal => {
                    i -= 1;
                    j -= 1;
                    let value = edge.value.max(x.value).max(y.value);
                    if value <= self.cutoff
                        && !visitor(SimplexEntry {
                            id: self.index.triangle(a, b, x.vertex),
                            value,
                        })?
                    {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
