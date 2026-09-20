//! Statistics and curves computed from an existing persistence diagram.
//!
//! These functions never rerun a persistence algorithm. Lifetime statistics use
//! only finite intervals; Betti curves also count essential and censored classes.

mod betti_curve;
mod lifetime_statistics;

pub use betti_curve::betti_curve;
pub use lifetime_statistics::{LifetimeSummary, finite_lifetime_summary};
