//! Hereditary insertion-radius constraint on candidate simplices.

pub(super) fn factor(epsilon: f64) -> f64 {
    if epsilon < 1.0 {
        epsilon * (1.0 - epsilon) / 2.0
    } else {
        0.0
    }
}
pub(super) fn allows(value: f64, factor: f64, radii: impl Iterator<Item = Option<f64>>) -> bool {
    // All products are finite: factor <= 1/8, value is a validated edge value.
    // Keep this multiplication order identical for edges and higher simplices.
    let threshold = value * factor;
    radii
        .into_iter()
        .all(|radius| radius.is_none_or(|r| r >= threshold))
}
