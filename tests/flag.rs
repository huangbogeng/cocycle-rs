//! Public contracts for flag.

use cocycle::Error;
use cocycle::complex::{WeightedEdge, WeightedGraph};
use cocycle::diagram::{Coverage, IntervalEnd};
use cocycle::filtration::FlagFiltration;
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout};
use cocycle::persistence::{
    ExecutionLimits, PersistenceOptions, compute_flag, compute_rips_from_distances,
};
use std::sync::atomic::{AtomicBool, Ordering};

fn cycle_with_isolate() -> FlagFiltration {
    FlagFiltration::new(
        WeightedGraph::new(
            5,
            [[0, 1], [1, 2], [2, 3], [0, 3]]
                .into_iter()
                .map(|vertices| WeightedEdge {
                    vertices,
                    value: 1.,
                })
                .collect(),
        )
        .unwrap(),
    )
}

#[test]
fn cycle_and_isolate_have_essential_classes_only_for_complete_graph() {
    let graph = cycle_with_isolate();
    for q in [0, 1] {
        let result = compute_flag(
            &graph,
            &PersistenceOptions::new(q, None).unwrap(),
            &ExecutionLimits::default(),
        )
        .unwrap();
        let h0: Vec<_> = result
            .diagram()
            .intervals_in_dimension(0)
            .unwrap()
            .collect();
        assert_eq!(h0.len(), 5);
        assert_eq!(
            h0.iter()
                .filter(|i| i.end() == IntervalEnd::Essential)
                .count(),
            2
        );
        if q == 1 {
            let h1: Vec<_> = result
                .diagram()
                .intervals_in_dimension(1)
                .unwrap()
                .collect();
            assert_eq!(h1.len(), 1);
            assert_eq!(h1[0].birth(), 1.);
            assert_eq!(h1[0].end(), IntervalEnd::Essential);
        }
    }
    let result = compute_flag(
        &graph,
        &PersistenceOptions::new(1, Some(0.)).unwrap(),
        &ExecutionLimits::default(),
    )
    .unwrap();
    assert_eq!(result.diagram().coverage(), Coverage::Through(0.));
    assert_eq!(result.diagram().intervals().len(), 5);
}

#[test]
fn control_failures_do_not_leak_state_into_later_computations() {
    let graph = cycle_with_isolate();
    let options = PersistenceOptions::default();
    let expected = compute_flag(&graph, &options, &ExecutionLimits::default()).unwrap();
    for limit in 0..8 {
        assert!(matches!(
            compute_flag(&graph, &options, &ExecutionLimits::new(Some(limit), None)),
            Err(Error::WorkLimitExceeded { .. })
        ));
    }
    let cancel = AtomicBool::new(true);
    assert!(matches!(
        compute_flag(&graph, &options, &ExecutionLimits::new(None, Some(&cancel))),
        Err(Error::Cancelled)
    ));
    cancel.store(false, Ordering::Relaxed);
    assert_eq!(
        compute_flag(
            &graph,
            &options,
            &ExecutionLimits::new(Some(10000), Some(&cancel))
        )
        .unwrap(),
        expected
    );
    assert!(PersistenceOptions::new(2, None).is_ok());
    assert!(PersistenceOptions::new(1, Some(f64::NAN)).is_err());
    let matrix =
        DissimilarityMatrixView::new(&[1., 1., 1.], 3, MatrixLayout::LowerTriangle).unwrap();
    assert!(matches!(
        compute_rips_from_distances(matrix, &options, &ExecutionLimits::new(Some(1), None)),
        Err(Error::WorkLimitExceeded { .. })
    ));
}

#[test]
fn sparse_h1_handles_many_isolated_vertices_with_bounded_work() {
    // A dense fallback would scan ~50 million pairs and fail this work limit.
    let graph = FlagFiltration::new(
        WeightedGraph::new(
            10000,
            vec![WeightedEdge {
                vertices: [9998, 9999],
                value: 2.,
            }],
        )
        .unwrap(),
    );
    let result = compute_flag(
        &graph,
        &PersistenceOptions::default(),
        &ExecutionLimits::new(Some(100), None),
    )
    .unwrap();
    assert_eq!(
        result.diagram().intervals_in_dimension(0).unwrap().count(),
        10000
    );
    assert_eq!(
        result.diagram().intervals_in_dimension(1).unwrap().count(),
        0
    );
}
