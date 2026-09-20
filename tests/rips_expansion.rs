//! Explicit topology contracts and dimension-generic F2 persistence.
use cocycle::{
    Error,
    complex::{WeightedEdge, WeightedGraph},
    diagram::{Coverage, IntervalEnd, PersistenceDiagram},
    filtration::{FlagFiltration, threshold_rips_from_distances},
    geometry::{DissimilarityMatrixView, MatrixLayout},
    persistence::{
        ExecutionLimits, PersistenceOptions, compute_expanded_rips, compute_flag,
        compute_rips_from_distances, compute_threshold_rips,
    },
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::atomic::AtomicBool;

fn matrix(values: &[f64], n: usize) -> DissimilarityMatrixView<'_> {
    DissimilarityMatrixView::new(values, n, MatrixLayout::LowerTriangle).unwrap()
}
fn cross_polytope(pairs: usize) -> Vec<f64> {
    (0..2 * pairs)
        .flat_map(|b| (0..b).map(move |a| if a / 2 == b / 2 { 2. } else { 1. }))
        .collect()
}
fn bars(diagram: &PersistenceDiagram) -> Vec<(usize, f64, Option<f64>)> {
    let mut bars: Vec<_> = diagram
        .intervals()
        .iter()
        .map(|i| {
            (
                i.dimension(),
                i.birth(),
                match i.end() {
                    IntervalEnd::Finite(d) => Some(d),
                    _ => None,
                },
            )
        })
        .collect();
    bars.sort_by(|a, b| {
        a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)).then(
            a.2.unwrap_or(f64::INFINITY)
                .total_cmp(&b.2.unwrap_or(f64::INFINITY)),
        )
    });
    bars
}
#[test]
fn high_dimensional_spheres_die_when_opposite_edges_enter() {
    for pairs in 3..=5 {
        let values = cross_polytope(pairs);
        let input = matrix(&values, 2 * pairs);
        for cutoff in [None, Some(1.)] {
            let source = threshold_rips_from_distances(input, cutoff).unwrap();
            let expansion = source.expand(pairs).unwrap();
            let options = PersistenceOptions::new(pairs, cutoff).unwrap();
            let limits = ExecutionLimits::default();
            let dense = compute_rips_from_distances(input, &options, &limits).unwrap();
            let sparse = compute_threshold_rips(&source, &options, &limits).unwrap();
            let explicit = compute_expanded_rips(&expansion, &options, &limits);
            // A filled simplex's pairs-skeleton cannot certify H_pairs.
            if cutoff.is_none() {
                assert!(matches!(explicit, Err(Error::InsufficientSkeleton { .. })));
                let expansion = source.expand(pairs + 1).unwrap();
                assert_eq!(
                    compute_expanded_rips(&expansion, &options, &limits)
                        .unwrap()
                        .diagram(),
                    dense.diagram()
                );
            } else {
                assert!(expansion.is_dimension_complete());
                assert_eq!(explicit.unwrap().diagram(), dense.diagram());
            }
            assert_eq!(sparse.diagram(), dense.diagram());
            let sphere: Vec<_> = dense
                .diagram()
                .intervals_in_dimension(pairs - 1)
                .unwrap()
                .collect();
            assert_eq!(sphere.len(), 1);
            assert_eq!(sphere[0].birth(), 1.);
            assert_eq!(
                sphere[0].end(),
                if cutoff.is_none() {
                    IntervalEnd::Finite(2.)
                } else {
                    IntervalEnd::RightCensored { through: 1. }
                }
            );
            let flag = FlagFiltration::new(source.graph().clone());
            let supplied = compute_flag(
                &flag,
                &PersistenceOptions::new(pairs, None).unwrap(),
                &limits,
            )
            .unwrap();
            if cutoff.is_some() {
                assert_eq!(
                    supplied
                        .diagram()
                        .intervals_in_dimension(pairs - 1)
                        .unwrap()
                        .next()
                        .unwrap()
                        .end(),
                    IntervalEnd::Essential
                );
            }
        }
    }
}
#[test]
fn incidence_is_closed_oriented_and_ordered() {
    let input = threshold_rips_from_distances(matrix(&[1.; 15], 6), None).unwrap();
    let expanded = input.expand(usize::MAX).unwrap();
    assert!(expanded.is_dimension_complete());
    let complex = expanded.complex();
    assert_eq!(complex.len(), 63);
    assert_eq!(complex.dimension(), Some(5));
    assert!(complex.find(&[]).is_none());
    assert!(complex.find(&[1, 0]).is_none());
    assert!(complex.find(&[0, 0]).is_none());
    assert!(complex.find(&[6]).is_none());
    for (i, simplex) in complex.simplices().iter().enumerate() {
        let id = complex.find(simplex.vertices()).unwrap();
        assert_eq!(id.index(), i);
        assert_eq!(complex.simplex(id).unwrap(), simplex);
        assert!(simplex.vertices().windows(2).all(|w| w[0] < w[1]));
        let mut second_boundary = BTreeMap::new();
        for term in complex.boundary(id).unwrap() {
            assert!(term.face.index() < i);
            let face = complex.simplex(term.face).unwrap();
            assert!(face.value() <= simplex.value());
            assert!(complex.cofacets(term.face).unwrap().contains(&id));
            for next in complex.boundary(term.face).unwrap() {
                *second_boundary.entry(next.face).or_insert(0_i32) +=
                    i32::from(term.coefficient) * i32::from(next.coefficient);
            }
        }
        assert!(
            second_boundary
                .values()
                .all(|&coefficient| coefficient == 0)
        );
    }
    // Equal-value, equal-dimension simplices use decreasing combinatorial rank.
    let edges: Vec<_> = complex
        .simplices()
        .iter()
        .filter(|s| s.dimension() == 1)
        .map(|s| s.vertices())
        .collect();
    assert_eq!(edges[0], [4, 5]);
    assert_eq!(edges.last().unwrap(), &&[0, 1][..]);
    let zero = input.expand(0).unwrap();
    assert_eq!(zero.complex().len(), 6);
    assert!(!zero.is_dimension_complete());
}
#[test]
fn expansion_rejects_missing_dimensions_and_unknown_scales() {
    let values = cross_polytope(3);
    let source = threshold_rips_from_distances(matrix(&values, 6), None).unwrap();
    let skeleton = source.expand(2).unwrap();
    let options = PersistenceOptions::new(2, None).unwrap();
    assert!(matches!(
        compute_expanded_rips(&skeleton, &options, &ExecutionLimits::default()),
        Err(Error::InsufficientSkeleton { .. })
    ));
    let threshold = threshold_rips_from_distances(matrix(&values, 6), Some(1.))
        .unwrap()
        .expand(2)
        .unwrap();
    assert!(threshold.is_dimension_complete());
    assert_eq!(threshold.coverage(), Coverage::Through(1.));
    assert!(matches!(
        compute_expanded_rips(
            &threshold,
            &PersistenceOptions::new(2, Some(2.)).unwrap(),
            &ExecutionLimits::default()
        ),
        Err(Error::IncompleteFiltration { .. })
    ));
    // Owning expansion remains usable after source and distance data are gone.
    drop(source);
    drop(values);
    assert!(compute_expanded_rips(&threshold, &options, &ExecutionLimits::default()).is_ok());
}
#[test]
fn arbitrary_requests_empty_inputs_and_control_failures() {
    for n in 0..=1 {
        let source = threshold_rips_from_distances(matrix(&[], n), None).unwrap();
        let options = PersistenceOptions::new(usize::MAX, None).unwrap();
        let expanded = source.expand(0).unwrap();
        assert!(expanded.is_dimension_complete());
        let result =
            compute_expanded_rips(&expanded, &options, &ExecutionLimits::default()).unwrap();
        assert_eq!(result.diagram().intervals().len(), n);
        assert_eq!(
            result.diagram(),
            compute_rips_from_distances(matrix(&[], n), &options, &ExecutionLimits::default())
                .unwrap()
                .diagram()
        );
    }
    let values = cross_polytope(4);
    let source = threshold_rips_from_distances(matrix(&values, 8), None).unwrap();
    let expanded = source.expand(4).unwrap();
    let options = PersistenceOptions::new(3, None).unwrap();
    for limit in [0, 10, 100, 1000] {
        let limits = ExecutionLimits::new(Some(limit), None);
        assert!(matches!(
            compute_expanded_rips(&expanded, &options, &limits),
            Err(Error::WorkLimitExceeded { .. })
        ));
        assert!(matches!(
            compute_threshold_rips(&source, &options, &limits),
            Err(Error::WorkLimitExceeded { .. })
        ));
    }
    let cancel = AtomicBool::new(true);
    assert!(matches!(
        compute_expanded_rips(
            &expanded,
            &options,
            &ExecutionLimits::new(None, Some(&cancel))
        ),
        Err(Error::Cancelled)
    ));
    assert!(compute_expanded_rips(&expanded, &options, &ExecutionLimits::default()).is_ok());
}

// Independent oracle: enumerate all vertex subsets by bitmask, then reduce the
// ordinary boundary matrix forward. Shares neither clique access nor coboundary
// reduction/clearing with production, and retains the full tiny matrix.
fn boundary_oracle(
    n: usize,
    edges: &[(usize, usize, f64)],
    q: usize,
) -> Vec<(usize, f64, Option<f64>)> {
    let weights: HashMap<_, _> = edges.iter().map(|&(a, b, w)| ((a, b), w)).collect();
    let mut simplices = Vec::new();
    for mask in 1_usize..1 << n {
        if mask.count_ones() as usize > q + 2 {
            continue;
        }
        let mut value = 0_f64;
        let mut valid = true;
        for b in 0..n {
            for a in 0..b {
                if mask & (1 << a) != 0 && mask & (1 << b) != 0 {
                    if let Some(&w) = weights.get(&(a, b)) {
                        value = value.max(w);
                    } else {
                        valid = false;
                    }
                }
            }
        }
        if valid {
            simplices.push((mask, value));
        }
    }
    simplices.sort_by(|a, b| {
        a.1.total_cmp(&b.1)
            .then(a.0.count_ones().cmp(&b.0.count_ones()))
            .then(b.0.cmp(&a.0))
    });
    let positions: HashMap<_, _> = simplices
        .iter()
        .enumerate()
        .map(|(i, &(mask, _))| (mask, i))
        .collect();
    let mut reduced: Vec<BTreeSet<usize>> = Vec::new();
    let mut owners = HashMap::new();
    let mut births = BTreeSet::new();
    let mut raw = Vec::new();
    for (j, &(mask, value)) in simplices.iter().enumerate() {
        let mut column = BTreeSet::new();
        if mask.count_ones() > 1 {
            for v in 0..n {
                if mask & (1 << v) != 0 {
                    column.insert(positions[&(mask ^ (1 << v))]);
                }
            }
        }
        while let Some(&pivot) = column.last() {
            if let Some(&owner) = owners.get(&pivot) {
                column = column
                    .symmetric_difference(&reduced[owner])
                    .copied()
                    .collect();
            } else {
                owners.insert(pivot, j);
                births.remove(&pivot);
                let (birth_mask, birth) = simplices[pivot];
                if value != birth {
                    raw.push((birth_mask.count_ones() as usize - 1, birth, Some(value)));
                }
                break;
            }
        }
        if column.is_empty() {
            births.insert(j);
        }
        reduced.push(column);
    }
    for birth in births {
        let (mask, value) = simplices[birth];
        raw.push((mask.count_ones() as usize - 1, value, None));
    }
    raw.retain(|r| r.0 <= q);
    raw.sort_by(|a, b| {
        a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)).then(
            a.2.unwrap_or(f64::INFINITY)
                .total_cmp(&b.2.unwrap_or(f64::INFINITY)),
        )
    });
    raw
}
#[test]
fn high_dimensions_match_independent_boundary_reduction() {
    let mut seed = 20260920_u64;
    for sample in 0..96 {
        let n = 6 + sample % 3;
        let q = 2 + sample % 3;
        let mut values = Vec::new();
        let mut edges = Vec::new();
        for b in 0..n {
            for a in 0..b {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let value = ((seed >> 32) % 5) as f64;
                values.push(value);
                if value <= 3. {
                    edges.push((a, b, value));
                }
            }
        }
        let graph = FlagFiltration::new(
            WeightedGraph::new(
                n,
                edges
                    .iter()
                    .map(|&(a, b, value)| WeightedEdge {
                        vertices: [a, b],
                        value,
                    })
                    .collect(),
            )
            .unwrap(),
        );
        let expected = boundary_oracle(n, &edges, q);
        let limits = ExecutionLimits::default();
        let options = PersistenceOptions::new(q, Some(3.)).unwrap();
        let input = matrix(&values, n);
        let source = threshold_rips_from_distances(input, Some(3.)).unwrap();
        let expanded = source.expand(q + 1).unwrap();
        for result in [
            compute_flag(&graph, &options, &limits).unwrap(),
            compute_rips_from_distances(input, &options, &limits).unwrap(),
            compute_threshold_rips(&source, &options, &limits).unwrap(),
            compute_expanded_rips(&expanded, &options, &limits).unwrap(),
        ] {
            assert_eq!(
                bars(result.diagram()),
                expected,
                "sample={sample} n={n} q={q} values={values:?}"
            );
        }
    }
}
#[test]
fn sparse_high_dimensions_do_not_densify() {
    let input = FlagFiltration::new(
        WeightedGraph::new(
            10000,
            vec![WeightedEdge {
                vertices: [9998, 9999],
                value: 1.,
            }],
        )
        .unwrap(),
    );
    let result = compute_flag(
        &input,
        &PersistenceOptions::new(4, None).unwrap(),
        &ExecutionLimits::new(Some(20000), None),
    )
    .unwrap();
    assert_eq!(result.diagram().intervals().len(), 10000);
}
