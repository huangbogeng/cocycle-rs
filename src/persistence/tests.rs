use super::*;
use crate::geometry::DissimilarityView;

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

#[test]
fn every_small_weighted_graph_matches_independent_explicit_reduction() {
    use crate::complex::{WeightedEdge, WeightedGraph};
    use crate::filtration::FlagFiltration;
    let pairs = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];
    for mut code in 0..3usize.pow(6) {
        let mut edges = Vec::new();
        for vertices in pairs {
            let state = code % 3;
            code /= 3;
            if state > 0 {
                edges.push(WeightedEdge {
                    vertices,
                    value: (state - 1) as f64,
                });
            }
        }
        let filtration = FlagFiltration::new(WeightedGraph::new(5, edges).unwrap());
        for q in [0, 1] {
            for cutoff in [None, Some(0.), Some(0.5), Some(1.)] {
                let options = PersistenceOptions::new(q, cutoff).unwrap();
                let actual =
                    compute_flag(&filtration, &options, &ExecutionLimits::default()).unwrap();
                let expected = reference::compute_graph(
                    5,
                    filtration.graph().edges(),
                    q,
                    cutoff.unwrap_or(1.),
                    actual.diagram().coverage(),
                )
                .unwrap();
                assert_eq!(
                    actual.diagram(),
                    &expected,
                    "edges={:?}, q={q}, cutoff={cutoff:?}",
                    filtration.graph().edges()
                );
            }
        }
    }
}
