//! Dimension-independent clique access without combinatorial integer IDs.
//!
//! Vertex tuples have the same decreasing-colex order as the H1 combinatorial
//! IDs, but cannot overflow merely because an absent simplex has a large rank.
use crate::complex::{FilteredSimplicialComplex, Simplex, WeightedGraph};
use crate::geometry::DissimilarityMatrixView;
use crate::{Error, Result};

/// Implementations supply zero-valued vertices and all codimension-one cofaces
/// within their cutoff. Unique extension adds only vertices greater than the
/// last vertex, so dimension traversal emits every clique exactly once.
pub(crate) trait SimplicialAccess {
    fn vertex_count(&self) -> usize;
    fn vertices(&self) -> Result<Vec<Simplex>> {
        vertices(self.vertex_count())
    }
    fn vertex_position(&self, original: usize) -> usize {
        original
    }
    fn visit_cofacets(
        &self,
        simplex: &Simplex,
        unique: bool,
        checkpoint: &mut impl FnMut() -> Result<()>,
        visitor: impl FnMut(Simplex) -> Result<()>,
    ) -> Result<()>;
}

pub(crate) enum CliqueAccess<'a> {
    Dense(DissimilarityMatrixView<'a>, f64),
    Sparse(&'a WeightedGraph, f64),
}
impl SimplicialAccess for CliqueAccess<'_> {
    fn vertex_count(&self) -> usize {
        match self {
            Self::Dense(matrix, _) => matrix.len(),
            Self::Sparse(graph, _) => graph.vertex_count(),
        }
    }
    fn visit_cofacets(
        &self,
        simplex: &Simplex,
        unique: bool,
        checkpoint: &mut impl FnMut() -> Result<()>,
        mut visitor: impl FnMut(Simplex) -> Result<()>,
    ) -> Result<()> {
        let mut candidate = |vertex: usize| -> Result<()> {
            checkpoint()?;
            if (unique && vertex <= *simplex.vertices.last().unwrap())
                || simplex.vertices.binary_search(&vertex).is_ok()
            {
                return Ok(());
            }
            let mut value = simplex.value;
            for &v in &simplex.vertices {
                checkpoint()?;
                let (edge, cutoff) = match self {
                    Self::Dense(matrix, cutoff) => (matrix.get(v, vertex), cutoff),
                    Self::Sparse(graph, cutoff) => (graph.edge_value(v, vertex), cutoff),
                };
                let Some(edge) = edge.filter(|w| w <= cutoff) else {
                    return Ok(());
                };
                value = value.max(edge);
            }
            let mut vertices = simplex.vertices.clone();
            vertices.try_reserve(1).map_err(|_| allocation())?;
            let position = vertices.partition_point(|&v| v < vertex);
            vertices.insert(position, vertex);
            visitor(Simplex { vertices, value })
        };
        match self {
            Self::Dense(matrix, _) => {
                for vertex in (0..matrix.len()).rev() {
                    candidate(vertex)?;
                }
            }
            Self::Sparse(graph, _) => {
                // Every common neighbor must occur in the shortest adjacency list.
                let neighbors = simplex
                    .vertices
                    .iter()
                    .map(|&v| graph.neighbors(v).unwrap())
                    .min_by_key(|neighbors| neighbors.len())
                    .unwrap();
                for neighbor in neighbors.iter().rev() {
                    candidate(neighbor.vertex)?;
                }
            }
        }
        Ok(())
    }
}

pub(crate) struct ExplicitAccess<'a> {
    pub(crate) complex: &'a FilteredSimplicialComplex,
    pub(crate) vertex_count: usize,
    pub(crate) cutoff: f64,
}
impl SimplicialAccess for ExplicitAccess<'_> {
    fn vertices(&self) -> Result<Vec<Simplex>> {
        let mut result = Vec::new();
        result
            .try_reserve_exact(self.vertex_count)
            .map_err(|_| allocation())?;
        result.extend(
            self.complex
                .simplices()
                .iter()
                .filter(|s| s.dimension() == 0)
                .cloned(),
        );
        Ok(result)
    }
    fn vertex_position(&self, original: usize) -> usize {
        // Zero-born vertices precede every positive-dimensional simplex.
        self.complex.find(&[original]).unwrap().index()
    }
    fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    fn visit_cofacets(
        &self,
        simplex: &Simplex,
        unique: bool,
        checkpoint: &mut impl FnMut() -> Result<()>,
        mut visitor: impl FnMut(Simplex) -> Result<()>,
    ) -> Result<()> {
        let id = self
            .complex
            .find(&simplex.vertices)
            .ok_or(Error::InternalInvariant {
                reason: "explicit column missing",
            })?;
        for &cofacet in self.complex.cofacets(id).unwrap() {
            checkpoint()?;
            let cofacet = self.complex.simplex(cofacet).unwrap();
            if cofacet.value <= self.cutoff
                && (!unique || cofacet.vertices[..simplex.vertices.len()] == simplex.vertices)
            {
                visitor(cofacet.clone())?;
            }
        }
        Ok(())
    }
}

pub(crate) fn vertices(n: usize) -> Result<Vec<Simplex>> {
    let mut result = Vec::new();
    result.try_reserve_exact(n).map_err(|_| allocation())?;
    result.extend((0..n).map(|v| Simplex {
        vertices: vec![v],
        value: 0.0,
    }));
    result.sort_unstable();
    Ok(result)
}
pub(crate) fn next_dimension(
    access: &impl SimplicialAccess,
    level: &[Simplex],
    checkpoint: &mut impl FnMut() -> Result<()>,
) -> Result<Vec<Simplex>> {
    let mut result = Vec::new();
    for simplex in level {
        checkpoint()?;
        access.visit_cofacets(simplex, true, checkpoint, |cofacet| {
            result.try_reserve(1).map_err(|_| allocation())?;
            result.push(cofacet);
            Ok(())
        })?;
    }
    result.sort_unstable();
    checkpoint()?;
    Ok(result)
}
fn allocation() -> Error {
    Error::AllocationFailed {
        context: "clique enumeration",
    }
}
