//! Deterministic order within a simplex dimension.

use std::cmp::Ordering;

/// Within one dimension: increasing value, then decreasing combinatorial id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SimplexEntry {
    pub(crate) id: usize,
    pub(crate) value: f64,
}

// Values come only from validated, canonicalized finite distances.
impl Eq for SimplexEntry {}
impl Ord for SimplexEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        crate::complex::compare_filtration(
            self.value,
            other.value,
            Ordering::Equal,
            other.id.cmp(&self.id),
        )
    }
}
impl PartialOrd for SimplexEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
