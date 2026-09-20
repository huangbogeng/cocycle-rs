//! Oriented incidence of a frozen simplicial complex.
use super::SimplexId;

/// One codimension-one boundary term with the increasing-vertex orientation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundaryTerm {
    /// Face in the same complex.
    pub face: SimplexId,
    /// `(-1)^i` for omission of vertex position `i`; either +1 or -1.
    pub coefficient: i8,
}
