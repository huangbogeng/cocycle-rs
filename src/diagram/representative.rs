//! Owned chain and cochain representatives associated with diagram intervals.

/// Which equation a representative satisfies at its query scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepresentativeKind {
    /// A cycle: its simplicial boundary is zero.
    Cycle,
    /// A cocycle: its simplicial coboundary is zero.
    Cocycle,
}
/// A nonzero chain/cochain coefficient on an increasingly oriented simplex.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepresentativeTerm {
    pub(crate) vertices: Vec<usize>,
    pub(crate) coefficient: u32,
}
impl RepresentativeTerm {
    /// Increasing original input vertex indices; no working complex IDs escape.
    pub fn vertices(&self) -> &[usize] {
        &self.vertices
    }
    /// Canonical nonzero residue in `1..characteristic`.
    pub fn coefficient(&self) -> u32 {
        self.coefficient
    }
}
/// One basis representative at a finite, inclusive query scale.
///
/// The interval index addresses this result's sorted diagram, distinguishing
/// duplicate intervals. Identity is local to one computation, not canonical
/// across vertex permutations or algorithms. Finite-interval cycles are chosen
/// to become boundaries at their associated death. Cocycles are the dual basis
/// to these persistent cycles at this scale; no cross-scale identical cochain
/// or shortest-support claim is made.
#[derive(Clone, Debug, PartialEq)]
pub struct Representative {
    pub(crate) request_index: usize,
    pub(crate) interval_index: usize,
    pub(crate) dimension: usize,
    pub(crate) characteristic: u32,
    pub(crate) scale: f64,
    pub(crate) kind: RepresentativeKind,
    pub(crate) terms: Vec<RepresentativeTerm>,
}
impl Representative {
    /// Position in the supplied request slice; repeated requests remain distinct.
    pub fn request_index(&self) -> usize {
        self.request_index
    }
    /// Position in the owning result's diagram interval slice.
    pub fn interval_index(&self) -> usize {
        self.interval_index
    }
    /// Homology/cohomology dimension.
    pub fn dimension(&self) -> usize {
        self.dimension
    }
    /// Prime coefficient characteristic.
    pub fn characteristic(&self) -> u32 {
        self.characteristic
    }
    /// Scale after all simplices with filtration value at most this value entered.
    pub fn scale(&self) -> f64 {
        self.scale
    }
    /// Cycle or cocycle equation satisfied by this representative.
    pub fn kind(&self) -> RepresentativeKind {
        self.kind
    }
    /// Nonzero terms in lexicographic vertex order.
    pub fn terms(&self) -> &[RepresentativeTerm] {
        &self.terms
    }
}
