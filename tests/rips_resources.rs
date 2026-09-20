//! Resource failures are local to a call and do not corrupt reusable inputs.
use cocycle::algebra::PrimeField;
use cocycle::diagram::PersistenceResult;
use cocycle::filtration::{
    FlagFiltration, SparseRipsOptions, sparse_rips_from_distances, threshold_rips_from_distances,
};
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout, MetricPolicy};
use cocycle::persistence::*;
use cocycle::{Error, Result};
use std::sync::atomic::{AtomicBool, Ordering};

type Computation<'a> = Box<dyn Fn(&ExecutionLimits<'_>) -> Result<PersistenceResult> + 'a>;

#[test]
fn all_rips_paths_recover_after_budget_and_cancellation_failures() {
    let values: Vec<_> = (0..8)
        .flat_map(|b| (0..b).map(move |a| if a / 2 == b / 2 { 2. } else { 1. }))
        .collect();
    let view = DissimilarityMatrixView::new(&values, 8, MatrixLayout::LowerTriangle).unwrap();
    let exact = threshold_rips_from_distances(view, None).unwrap();
    let expanded = exact.expand(4).unwrap();
    let flag = FlagFiltration::new(exact.graph().clone());
    let sparse = sparse_rips_from_distances(
        view,
        &SparseRipsOptions::new(0.5, MetricPolicy::Check).unwrap(),
    )
    .unwrap();
    let sparse_expanded = sparse.expand(4).unwrap();
    let cancelled = AtomicBool::new(false);
    for p in [2, 3] {
        let options = PersistenceOptions::new(3, None)
            .unwrap()
            .with_field(PrimeField::new(p).unwrap());
        let requests = [RepresentativeRequest::new(3, 1., RepresentativeSelection::Both).unwrap()];
        let paths: Vec<Computation<'_>> = vec![
            Box::new(|limits| compute_rips_from_distances(view, &options, limits)),
            Box::new(|limits| compute_threshold_rips(&exact, &options, limits)),
            Box::new(|limits| compute_flag(&flag, &options, limits)),
            Box::new(|limits| compute_expanded_rips(&expanded, &options, limits)),
            Box::new(|limits| compute_sparse_rips(&sparse, &options, limits)),
            Box::new(|limits| compute_expanded_sparse_rips(&sparse_expanded, &options, limits)),
            Box::new(|limits| {
                compute_rips_from_distances_with_representatives(view, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_threshold_rips_with_representatives(&exact, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_flag_with_representatives(&flag, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_expanded_rips_with_representatives(&expanded, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_sparse_rips_with_representatives(&sparse, &options, &requests, limits)
            }),
            Box::new(|limits| {
                compute_expanded_sparse_rips_with_representatives(
                    &sparse_expanded,
                    &options,
                    &requests,
                    limits,
                )
            }),
        ];
        for compute in paths {
            let expected = compute(&ExecutionLimits::default()).unwrap();
            for limit in [0, 7, 31] {
                assert_eq!(
                    compute(&ExecutionLimits::new(Some(limit), None)).unwrap_err(),
                    Error::WorkLimitExceeded { limit }
                );
            }
            cancelled.store(true, Ordering::Relaxed);
            assert_eq!(
                compute(&ExecutionLimits::new(None, Some(&cancelled))).unwrap_err(),
                Error::Cancelled
            );
            assert!(cancelled.load(Ordering::Relaxed));
            cancelled.store(false, Ordering::Relaxed);
            assert_eq!(
                compute(&ExecutionLimits::new(None, Some(&cancelled))).unwrap(),
                expected
            );
        }
    }
}

#[test]
fn simultaneous_prime_field_calls_share_only_immutable_input() {
    let values = [1., 2., 1., 1., 2., 1.];
    let view = DissimilarityMatrixView::new(&values, 4, MatrixLayout::LowerTriangle).unwrap();
    let input = threshold_rips_from_distances(view, None).unwrap();
    std::thread::scope(|scope| {
        let handles: Vec<_> = [2, 3, 5, 251]
            .into_iter()
            .map(|p| {
                let input = &input;
                scope.spawn(move || {
                    let options = PersistenceOptions::new(1, None)
                        .unwrap()
                        .with_field(PrimeField::new(p).unwrap());
                    let expected =
                        compute_threshold_rips(input, &options, &ExecutionLimits::default())
                            .unwrap();
                    for _ in 0..4 {
                        assert!(
                            compute_threshold_rips(
                                input,
                                &options,
                                &ExecutionLimits::new(Some(1), None)
                            )
                            .is_err()
                        );
                        assert_eq!(
                            compute_threshold_rips(input, &options, &ExecutionLimits::default())
                                .unwrap(),
                            expected
                        );
                    }
                    expected.into_diagram()
                })
            })
            .collect();
        let diagrams: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(diagrams.windows(2).all(|pair| pair[0] == pair[1]));
    });
}
