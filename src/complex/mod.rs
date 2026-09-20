//! Owned validated topology storage, independent of persistence workspaces.

mod graph;
pub use graph::{Neighbor, WeightedEdge, WeightedGraph};

mod simplicial;
pub use simplicial::{BoundaryTerm, FilteredSimplicialComplex, Simplex, SimplexId};

pub(crate) use simplicial::compare_filtration;
