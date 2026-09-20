//! Requested persistent cycle bases and their scale-specific dual cocycles.
mod basis;
mod complex;
mod dual;
mod request;
pub(super) use basis::compute;
pub use request::{RepresentativeRequest, RepresentativeSelection};
