//! Owned persistence intervals and their computation range.
//!
//! These types validate interval and range consistency, not whether a diagram
//! was actually produced by a particular dataset or algorithm. Dimensions are
//! general nonnegative integers and scales may be negative. Rips-specific facts
//! such as zero-dimensional births at zero belong to the Rips computation.

mod interval;
mod persistence_diagram;

pub use interval::{IntervalEnd, PersistenceInterval};
pub use persistence_diagram::{Coverage, PersistenceDiagram};
