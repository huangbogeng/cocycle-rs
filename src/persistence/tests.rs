use super::*;

#[test]
fn union_find_matches_reference_reduction_with_ties_and_cutoffs() {
    let mut state = 19_u64;
    for n in 0_usize..=8 {
        for _ in 0..20 {
            let values: Vec<_> = (0..n * n.saturating_sub(1) / 2)
                .map(|_| {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    ((state >> 32) % 5) as f64
                })
                .collect();
            let input = DissimilarityView::new(&values, n).unwrap();
            for cutoff in [None, Some(0.0), Some(1.0), Some(3.0), Some(4.0)] {
                let options = RipsOptions::new(0, cutoff).unwrap();
                assert_eq!(
                    rips_from_dissimilarities(input, &options).unwrap(),
                    reference::compute(input, &options).unwrap()
                );
            }
        }
    }
}
