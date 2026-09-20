//! Validated coefficient fields; private sparse arithmetic used by persistence.
mod field;
pub use field::PrimeField;
pub(crate) mod column;
pub(crate) mod reduction;
