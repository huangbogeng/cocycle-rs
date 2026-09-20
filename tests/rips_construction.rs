//! Public contracts for rips construction.

use cocycle::Error;
use cocycle::diagram::{Coverage, FiltrationKind, IntervalEnd};
use cocycle::filtration::{
    FlagFiltration, threshold_rips_from_distances, threshold_rips_from_points,
    threshold_rips_with_distance,
};
use cocycle::geometry::{DissimilarityMatrixView as Matrix, MatrixLayout, PointCloudView};
use cocycle::persistence::{
    ExecutionLimits, PersistenceOptions, compute_flag, compute_rips_from_points,
    compute_threshold_rips,
};

#[test]
fn construction_equivalence_duplicate_points_and_owned_results() {
    let source = vec![0., 0., 1., 0., 1., 1., 0., 1., 0., 0.];
    let points = PointCloudView::new(&source, 5, 2).unwrap();
    let graph = threshold_rips_from_points(points, Some(1.)).unwrap();
    let objects = [[0f64, 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]];
    let mut calls = Vec::new();
    let custom = threshold_rips_with_distance(&objects, Some(1.), |a, b| {
        calls.push((*a, *b));
        Ok((a[0] - b[0]).hypot(a[1] - b[1]))
    })
    .unwrap();
    assert_eq!(calls.len(), 10);
    assert_eq!(graph.graph().edges(), custom.graph().edges());
    assert_eq!(graph.graph().vertex_count(), 5);
    assert_eq!(graph.graph().edge_value(0, 4), Some(0.));
    let options = PersistenceOptions::new(1, Some(1.)).unwrap();
    let result = compute_threshold_rips(&graph, &options, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        result,
        compute_rips_from_points(points, &options, &ExecutionLimits::default()).unwrap()
    );
    drop(source);
    drop(graph);
    assert_eq!(result.context().vertex_count(), 5);
    assert_eq!(
        result.context().filtration_kind(),
        FiltrationKind::RipsEuclidean
    );
    assert_eq!(result.diagram().coverage(), Coverage::Through(1.));
}

#[test]
fn original_range_is_not_inferred_from_the_largest_retained_edge() {
    let matrix = Matrix::new(&[1., 2., 1., 1., 2., 1.], 4, MatrixLayout::LowerTriangle).unwrap();
    let threshold = threshold_rips_from_distances(matrix, Some(1.5)).unwrap();
    assert_eq!(threshold.graph().max_edge(), 1.);
    let limits = ExecutionLimits::default();
    let result =
        compute_threshold_rips(&threshold, &PersistenceOptions::default(), &limits).unwrap();
    assert_eq!(result.diagram().coverage(), Coverage::Through(1.5));
    assert!(
        result
            .diagram()
            .intervals_in_dimension(1)
            .unwrap()
            .any(|i| matches!(i.end(), IntervalEnd::RightCensored { through: 1.5 }))
    );
    assert!(matches!(
        compute_threshold_rips(
            &threshold,
            &PersistenceOptions::new(1, Some(2.)).unwrap(),
            &limits
        ),
        Err(Error::IncompleteFiltration { .. })
    ));
    let supplied = FlagFiltration::new(threshold.graph().clone());
    let result = compute_flag(&supplied, &PersistenceOptions::default(), &limits).unwrap();
    assert_eq!(result.diagram().coverage(), Coverage::Complete);
    assert!(
        result
            .diagram()
            .intervals_in_dimension(1)
            .unwrap()
            .any(|i| i.end() == IntervalEnd::Essential)
    );
    assert_eq!(
        result.context().filtration_kind(),
        FiltrationKind::SuppliedFlag
    );
}

#[test]
fn callbacks_validate_every_pair_even_outside_the_cutoff() {
    let mut calls = Vec::new();
    threshold_rips_with_distance(&[0, 1, 2], Some(0.), |a, b| {
        calls.push((*a, *b));
        Ok(10.)
    })
    .unwrap();
    assert_eq!(calls, vec![(0, 1), (0, 2), (1, 2)]);
    for invalid in [-1., f64::NAN, f64::INFINITY] {
        assert!(threshold_rips_with_distance(&[0, 1], Some(0.), |_, _| Ok(invalid)).is_err());
    }
    assert!(matches!(
        threshold_rips_with_distance(&[0, 1], None, |_, _| Err(Error::Cancelled)),
        Err(Error::Cancelled)
    ));
    assert!(
        threshold_rips_with_distance(&[0, 1], Some(-1.), |_, _| panic!(
            "invalid cutoff must precede callbacks"
        ))
        .is_err()
    );
}

#[test]
fn empty_singleton_numeric_limits_and_complete_cutoffs() {
    for n in [0, 1] {
        let input = Matrix::new(&[], n, MatrixLayout::LowerTriangle).unwrap();
        let graph = threshold_rips_from_distances(input, Some(0.)).unwrap();
        assert_eq!(graph.graph().vertex_count(), n);
        assert_eq!(graph.coverage(), Coverage::Complete);
    }
    let input = Matrix::new(&[f64::MAX], 2, MatrixLayout::UpperTriangle).unwrap();
    assert_eq!(
        threshold_rips_from_distances(input, Some(f64::MAX))
            .unwrap()
            .coverage(),
        Coverage::Complete
    );
    let coords = [f64::MAX, -f64::MAX];
    assert!(
        threshold_rips_from_points(PointCloudView::new(&coords, 2, 1).unwrap(), Some(0.)).is_err()
    );
}
