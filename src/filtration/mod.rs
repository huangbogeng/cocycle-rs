//! Filtration construction, provenance and shared implicit access.
//!
//! Exact threshold Rips and supplied flag filtrations differ in what missing
//! edges mean. Construction never performs persistence reduction.

pub(crate) mod flag;
pub(crate) mod rips;
pub use flag::FlagFiltration;
pub use rips::{
    RipsExpansion, RipsInputKind, ThresholdRips, threshold_rips_from_distances,
    threshold_rips_from_points, threshold_rips_with_distance,
};

pub use rips::approximation::{
    SparseRips, SparseRipsExpansion, SparseRipsOptions, sparse_rips_from_distances,
    sparse_rips_from_points, sparse_rips_with_distance,
};
