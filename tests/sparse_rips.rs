//! Sparse approximation contracts and blocker-aware integration.
use cocycle::Error;
use cocycle::algebra::PrimeField;
use cocycle::diagram::{ApproximationTarget, Coverage, IntervalEnd};
use cocycle::filtration::{
    FlagFiltration, SparseRipsOptions, sparse_rips_from_distances, sparse_rips_from_points,
    sparse_rips_with_distance,
};
use cocycle::geometry::{
    DissimilarityMatrixView, MatrixLayout, MetricPolicy, MetricValidation, PointCloudView,
    validate_metric,
};
use cocycle::persistence::{
    ExecutionLimits, PersistenceOptions, RepresentativeRequest, RepresentativeSelection,
    compute_expanded_sparse_rips, compute_sparse_rips, compute_sparse_rips_with_representatives,
};
use std::sync::atomic::AtomicBool;

fn matrix(values: &[f64], n: usize) -> DissimilarityMatrixView<'_> {
    DissimilarityMatrixView::new(values, n, MatrixLayout::LowerTriangle).unwrap()
}
fn options(epsilon: f64) -> SparseRipsOptions {
    SparseRipsOptions::new(epsilon, MetricPolicy::Check).unwrap()
}
fn line(points: &[f64], options: &SparseRipsOptions) -> cocycle::filtration::SparseRips {
    sparse_rips_with_distance(points, options, |a, b| Ok((a - b).abs())).unwrap()
}

#[test]
fn deterministic_sampling_preserves_zero_duplicates_and_subsample_provenance() {
    let input = line(&[0., 0., 2., 10., 5.], &options(0.5));
    let metadata = input.approximation();
    assert_eq!(metadata.permutation(), &[0, 3, 4, 2, 1]);
    assert_eq!(
        metadata.insertion_radii(),
        &[None, Some(10.), Some(5.), Some(2.), Some(0.)]
    );
    assert_eq!(metadata.retained_vertices(), &[0, 2, 3, 4]);
    assert_eq!(metadata.metric_validation(), MetricValidation::Checked);
    assert_eq!(
        metadata.bound().unwrap().target(),
        ApproximationTarget::OriginalInput
    );
    assert_eq!(metadata.bound().unwrap().factor(), 2.);
    let input = line(
        &[0., 0., 2., 10., 5.],
        &options(0.5).with_min_insertion_radius(3.).unwrap(),
    );
    assert_eq!(input.approximation().retained_vertices(), &[0, 3, 4]);
    assert_eq!(input.approximation().covering_radius(), 2.);
    assert_eq!(
        input.approximation().bound().unwrap().target(),
        ApproximationTarget::RetainedSubset
    );
    // Equality at the insertion threshold retains the point.
    assert_eq!(
        line(
            &[0., 2., 10.],
            &options(0.5).with_min_insertion_radius(2.).unwrap()
        )
        .graph()
        .vertex_count(),
        3
    );
    let tied = line(&[0., 2., -2., 1., -1.], &options(0.5));
    assert_eq!(tied.approximation().permutation(), &[0, 1, 2, 3, 4]);
    let started = line(&[0., 2., -2.], &options(0.5).with_start_vertex(2));
    assert_eq!(started.approximation().permutation(), &[2, 1, 0]);
}

#[test]
fn metric_evidence_is_explicit_and_rounding_does_not_hide_violations() {
    let bad = matrix(&[1., 3., 1.], 3);
    assert!(matches!(
        validate_metric(bad, MetricPolicy::Check),
        Err(Error::InvalidMetric { .. })
    ));
    assert!(sparse_rips_from_distances(bad, &options(0.5)).is_err());
    for policy in [MetricPolicy::Unchecked, MetricPolicy::Assume] {
        let input =
            sparse_rips_from_distances(bad, &SparseRipsOptions::new(0.5, policy).unwrap()).unwrap();
        assert_eq!(
            input.approximation().bound().is_some(),
            policy == MetricPolicy::Assume
        );
    }
    // 1 + next_down(1) rounds UP to 2, but the exact stored values sum below 2.
    assert!(
        validate_metric(
            matrix(&[1., 2., 1.0_f64.next_down()], 3),
            MetricPolicy::Check
        )
        .is_err()
    );
    assert!(validate_metric(matrix(&[f64::MAX; 3], 3), MetricPolicy::Check).is_ok());
    assert!(
        validate_metric(
            matrix(
                &[f64::from_bits(1), f64::from_bits(3), f64::from_bits(1)],
                3
            ),
            MetricPolicy::Check
        )
        .is_err()
    );
    assert!(validate_metric(matrix(&[0., 1., 2.], 3), MetricPolicy::Check).is_err());
}

fn blocker_input() -> cocycle::filtration::SparseRips {
    let points = [
        (68_i32, 71_i32),
        (87, 26),
        (42, 69),
        (15, 91),
        (81, 8),
        (39, 52),
        (10, 64),
        (60, 81),
    ];
    sparse_rips_with_distance(&points, &options(0.5), |a, b| {
        Ok(((a.0 - b.0).abs() + (a.1 - b.1).abs()) as f64)
    })
    .unwrap()
}
#[test]
fn blocker_excludes_a_real_metric_clique_and_preserves_face_closure() {
    let input = blocker_input();
    let expanded = input.expand(7).unwrap();
    let bare = FlagFiltration::new(input.graph().clone())
        .expand(7)
        .unwrap();
    assert!(bare.find(&[1, 2, 3]).is_some());
    assert!(expanded.complex().find(&[1, 2, 3]).is_none());
    assert!(expanded.complex().len() < bare.len());
    assert!(expanded.is_dimension_complete());
    // Independent subset enumeration from publicly exposed graph and radii.
    let ids = input.approximation().retained_vertices();
    let mut expected = Vec::new();
    for mask in 1usize..1 << ids.len() {
        let local: Vec<_> = (0..ids.len()).filter(|v| mask & (1 << v) != 0).collect();
        let vertices: Vec<_> = local.iter().map(|&i| ids[i]).collect();
        let mut value = 0.0_f64;
        let mut clique = true;
        for (i, &a) in local.iter().enumerate() {
            for &b in &local[..i] {
                match input.graph().edge_value(a, b) {
                    Some(w) => value = value.max(w),
                    None => clique = false,
                }
            }
        }
        let allowed = vertices.iter().all(|v| {
            let i = input
                .approximation()
                .permutation()
                .iter()
                .position(|x| x == v)
                .unwrap();
            input.approximation().insertion_radii()[i].is_none_or(|r| r >= value / 8.)
        });
        if clique && allowed {
            expected.push((vertices, value));
        }
    }
    let mut actual: Vec<_> = expanded
        .complex()
        .simplices()
        .iter()
        .map(|s| (s.vertices().to_vec(), s.value()))
        .collect();
    actual.sort_by(|a, b| a.0.cmp(&b.0));
    expected.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(actual, expected);
    for simplex in expanded.complex().simplices() {
        if simplex.dimension() == 0 {
            continue;
        }
        for i in 0..simplex.vertices().len() {
            let mut face = simplex.vertices().to_vec();
            face.remove(i);
            let id = expanded.complex().find(&face).unwrap();
            assert!(expanded.complex().simplex(id).unwrap().value() <= simplex.value());
        }
    }
}

#[test]
fn arbitrary_dimensions_fields_and_representatives_agree_on_blocked_topology() {
    for input in [
        blocker_input(),
        line(&[0., 0., 2., 10., 5.], &options(0.5).with_start_vertex(3)),
    ] {
        let expanded = input.expand(7).unwrap();
        for p in [2, 3, 5, 251, 4_294_967_291] {
            for q in 0..=4 {
                let opts = PersistenceOptions::new(q, None)
                    .unwrap()
                    .with_field(PrimeField::new(p).unwrap());
                let plain =
                    compute_sparse_rips(&input, &opts, &ExecutionLimits::default()).unwrap();
                assert_eq!(
                    plain.diagram(),
                    compute_expanded_sparse_rips(&expanded, &opts, &ExecutionLimits::default())
                        .unwrap()
                        .diagram()
                );
                let queries: Vec<_> = (0..=q)
                    .map(|d| {
                        RepresentativeRequest::new(d, 25., RepresentativeSelection::Both).unwrap()
                    })
                    .collect();
                let represented = compute_sparse_rips_with_representatives(
                    &input,
                    &opts,
                    &queries,
                    &ExecutionLimits::default(),
                )
                .unwrap();
                assert_eq!(plain.diagram(), represented.diagram());
                assert_eq!(plain.context().approximation(), Some(input.approximation()));
                for rep in represented.representatives().unwrap() {
                    for term in rep.terms() {
                        assert!(
                            term.vertices()
                                .iter()
                                .all(|v| input.approximation().retained_vertices().contains(v))
                        );
                        assert!(expanded.complex().find(term.vertices()).is_some());
                    }
                }
            }
        }
    }
}

#[test]
fn coverage_and_dimension_are_independent_and_no_bound_for_large_epsilon() {
    let input = line(
        &[0., 1., 3.],
        &options(0.5).with_max_scale(Some(1.)).unwrap(),
    );
    assert_eq!(input.coverage(), Coverage::Through(1.));
    let opts = PersistenceOptions::new(1, None).unwrap();
    let result = compute_sparse_rips(&input, &opts, &ExecutionLimits::default()).unwrap();
    assert!(
        result
            .diagram()
            .intervals()
            .iter()
            .any(|i| matches!(i.end(), IntervalEnd::RightCensored { through: 1. }))
    );
    assert!(
        compute_sparse_rips(
            &input,
            &PersistenceOptions::new(1, Some(2.)).unwrap(),
            &ExecutionLimits::default()
        )
        .is_err()
    );
    let full = line(&[0., 1., 3.], &options(0.5));
    let vertices = full.expand(0).unwrap();
    assert_eq!(vertices.complex().len(), 3);
    assert!(!vertices.is_dimension_complete());
    assert!(matches!(
        compute_expanded_sparse_rips(&vertices, &opts, &ExecutionLimits::default()),
        Err(Error::InsufficientSkeleton { .. })
    ));
    for epsilon in [1., 1.5, 2.] {
        let input = line(&[0., 1., 3.], &options(epsilon));
        assert!(input.approximation().bound().is_none());
        let bare = FlagFiltration::new(input.graph().clone())
            .expand(4)
            .unwrap();
        assert_eq!(
            input.expand(4).unwrap().complex().simplices(),
            bare.simplices()
        );
    }
}

#[test]
fn empty_singleton_duplicate_inputs_and_execution_limits() {
    for n in 0..=4 {
        let input = line(&vec![0.; n], &options(0.5));
        assert_eq!(input.graph().vertex_count(), usize::from(n > 0));
        assert_eq!(input.approximation().permutation().len(), n);
        let expanded = input.expand(0).unwrap();
        assert!(expanded.is_dimension_complete());
        let opts = PersistenceOptions::new(4, None).unwrap();
        let result =
            compute_expanded_sparse_rips(&expanded, &opts, &ExecutionLimits::default()).unwrap();
        assert_eq!(result.diagram().intervals().len(), usize::from(n > 0));
    }
    let input = blocker_input();
    let opts = PersistenceOptions::new(2, None).unwrap();
    assert!(matches!(
        compute_sparse_rips(&input, &opts, &ExecutionLimits::new(Some(1), None)),
        Err(Error::WorkLimitExceeded { .. })
    ));
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        compute_sparse_rips(&input, &opts, &ExecutionLimits::new(None, Some(&cancelled)))
            .unwrap_err(),
        Error::Cancelled
    );
}

#[test]
fn input_paths_cache_callbacks_once_and_keep_matrix_layouts_equivalent() {
    let coords = [0., 0., 3., 0., 0., 4.];
    let options = options(0.5);
    let points =
        sparse_rips_from_points(PointCloudView::new(&coords, 3, 2).unwrap(), &options).unwrap();
    let lower = sparse_rips_from_distances(matrix(&[3., 4., 5.], 3), &options).unwrap();
    let upper = sparse_rips_from_distances(
        DissimilarityMatrixView::new(&[3., 4., 5.], 3, MatrixLayout::UpperTriangle).unwrap(),
        &options,
    )
    .unwrap();
    let square = sparse_rips_from_distances(
        DissimilarityMatrixView::new(
            &[0., 3., 4., 3., 0., 5., 4., 5., 0.],
            3,
            MatrixLayout::Square,
        )
        .unwrap(),
        &options,
    )
    .unwrap();
    for input in [points, upper, square] {
        assert_eq!(input.approximation(), lower.approximation());
        assert_eq!(input.graph().edges(), lower.graph().edges());
    }
    let mut calls = Vec::new();
    let callback = sparse_rips_with_distance(&[0, 1, 2], &options, |&a, &b| {
        calls.push((a, b));
        Ok(lower_distance(a, b))
    })
    .unwrap();
    assert_eq!(calls, vec![(0, 1), (0, 2), (1, 2)]);
    assert_eq!(callback.graph().edges(), lower.graph().edges());
    fn lower_distance(a: usize, b: usize) -> f64 {
        [[0., 3., 4.], [3., 0., 5.], [4., 5., 0.]][a][b]
    }
}

#[test]
fn invalid_parameters_and_unrepresentable_arithmetic_fail_explicitly() {
    for epsilon in [0., -1., f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        assert!(SparseRipsOptions::new(epsilon, MetricPolicy::Assume).is_err());
    }
    assert!(options(0.5).with_min_insertion_radius(-1.).is_err());
    assert!(options(0.5).with_max_scale(Some(f64::NAN)).is_err());
    assert!(
        sparse_rips_from_distances(matrix(&[], 0), &options(0.5).with_start_vertex(0)).is_err()
    );
    assert!(
        sparse_rips_from_distances(matrix(&[1.], 2), &options(0.5).with_start_vertex(2)).is_err()
    );
    assert!(matches!(
        sparse_rips_from_distances(matrix(&[f64::MAX], 2), &options(0.5)),
        Err(Error::NumericalFailure { .. })
    ));
    let source = [0., 1., 2.];
    assert!(
        sparse_rips_with_distance(
            &source,
            &options(0.5).with_max_scale(Some(0.)).unwrap(),
            |_, _| Ok(f64::INFINITY)
        )
        .is_err()
    );
}

#[test]
fn noncontiguous_h1_representatives_are_closed_dual_and_owned() {
    use cocycle::diagram::{FiltrationKind, RepresentativeKind};
    use cocycle::persistence::compute_expanded_sparse_rips_with_representatives;
    use std::collections::BTreeMap;
    // Duplicating alternating points produces retained original IDs 0,2,4,6.
    let points = [
        (0_i32, 0_i32),
        (0, 0),
        (1, 0),
        (1, 0),
        (1, 1),
        (1, 1),
        (0, 1),
    ];
    let input = sparse_rips_with_distance(&points, &options(0.5), |a, b| {
        Ok(((a.0 - b.0).abs() + (a.1 - b.1).abs()) as f64)
    })
    .unwrap();
    assert_eq!(input.approximation().retained_vertices(), &[0, 2, 4, 6]);
    let opts = PersistenceOptions::new(1, None)
        .unwrap()
        .with_field(PrimeField::new(3).unwrap());
    let requests = [RepresentativeRequest::new(1, 1., RepresentativeSelection::Both).unwrap()];
    let limits = ExecutionLimits::default();
    let result =
        compute_sparse_rips_with_representatives(&input, &opts, &requests, &limits).unwrap();
    let expanded = input.expand(2).unwrap();
    let explicit =
        compute_expanded_sparse_rips_with_representatives(&expanded, &opts, &requests, &limits)
            .unwrap();
    assert_eq!(result, explicit);
    drop(expanded);
    drop(input);
    assert_eq!(
        result.context().filtration_kind(),
        FiltrationKind::SparseRipsCustom
    );
    assert_eq!(result.context().vertex_count(), 4);
    let h1: Vec<_> = result
        .diagram()
        .intervals()
        .iter()
        .filter(|i| i.dimension() == 1)
        .collect();
    assert_eq!(h1.len(), 1);
    assert_eq!(h1[0].birth(), 1.);
    assert_eq!(h1[0].end(), IntervalEnd::Finite(2.));
    let reps = result.representatives().unwrap();
    let cycle = reps
        .iter()
        .find(|r| r.kind() == RepresentativeKind::Cycle)
        .unwrap();
    let cocycle = reps
        .iter()
        .find(|r| r.kind() == RepresentativeKind::Cocycle)
        .unwrap();
    let mut boundary = BTreeMap::<usize, i64>::new();
    for term in cycle.terms() {
        let vertices = term.vertices();
        *boundary.entry(vertices[0]).or_default() -= i64::from(term.coefficient());
        *boundary.entry(vertices[1]).or_default() += i64::from(term.coefficient());
        assert!(vertices.iter().all(|v| [0, 2, 4, 6].contains(v)));
    }
    assert!(boundary.values().all(|v| v.rem_euclid(3) == 0));
    // No 2-simplices exist at scale 1, so this nonzero cycle is not a boundary.
    assert!(!cycle.terms().is_empty());
    let evaluation: u64 = cycle
        .terms()
        .iter()
        .map(|chain| {
            let value = cocycle
                .terms()
                .iter()
                .find(|cochain| cochain.vertices() == chain.vertices())
                .map_or(0, |cochain| cochain.coefficient());
            u64::from(chain.coefficient()) * u64::from(value)
        })
        .sum();
    assert_eq!(evaluation % 3, 1);
}
