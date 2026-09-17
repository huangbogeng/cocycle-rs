//! Hand calculations, independent F2 ranks and invariance tests for Rips persistence.

use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram, PersistenceInterval};
use cocycle::geometry::DissimilarityView;
use cocycle::persistence::{RipsOptions, rips_from_dissimilarities};

fn compute(values: &[f64], n: usize, dimension: usize, cutoff: Option<f64>) -> PersistenceDiagram {
    rips_from_dissimilarities(
        DissimilarityView::new(values, n).unwrap(),
        &RipsOptions::new(dimension, cutoff).unwrap(),
    )
    .unwrap()
}

fn finite(dimension: usize, birth: f64, death: f64) -> PersistenceInterval {
    PersistenceInterval::new(dimension, birth, IntervalEnd::Finite(death)).unwrap()
}

fn alive(dimension: usize, birth: f64, cutoff: Option<f64>) -> PersistenceInterval {
    let end = cutoff.map_or(IntervalEnd::Essential, |through| {
        IntervalEnd::RightCensored { through }
    });
    PersistenceInterval::new(dimension, birth, end).unwrap()
}

#[test]
fn empty_singleton_pair_duplicates_and_filled_triangle() {
    let cases = [
        (0, vec![], vec![]),
        (1, vec![], vec![alive(0, 0.0, None)]),
        (2, vec![2.0], vec![finite(0, 0.0, 2.0), alive(0, 0.0, None)]),
        (2, vec![0.0], vec![alive(0, 0.0, None)]),
        (
            3,
            vec![1.0; 3],
            vec![
                finite(0, 0.0, 1.0),
                finite(0, 0.0, 1.0),
                alive(0, 0.0, None),
            ],
        ),
    ];
    for (n, values, expected) in cases {
        let diagram = compute(&values, n, 1, None);
        assert_eq!(diagram.coverage(), Coverage::Complete);
        assert_eq!(diagram.intervals(), expected);
    }
}

#[test]
fn square_loop_dies_at_the_diagonal_and_is_censored_at_a_smaller_cutoff() {
    let diagonal = 2.0_f64.sqrt();
    let values = [1., diagonal, 1., 1., diagonal, 1.];
    let mut full = vec![finite(0, 0., 1.); 3];
    full.extend([alive(0, 0., None), finite(1, 1., diagonal)]);
    assert_eq!(compute(&values, 4, 1, None).intervals(), full);
    let mut partial = vec![finite(0, 0., 1.); 3];
    partial.extend([alive(0, 0., Some(1.)), alive(1, 1., Some(1.))]);
    assert_eq!(compute(&values, 4, 1, Some(1.)).intervals(), partial);
    let isolated = compute(&values, 4, 1, Some(0.5));
    assert_eq!(isolated.intervals(), vec![alive(0, 0., Some(0.5)); 4]);
    for cutoff in [diagonal, 2.0] {
        let result = compute(&values, 4, 1, Some(cutoff));
        assert_eq!(result.coverage(), Coverage::Complete);
        assert_eq!(result.intervals(), full);
    }
}

#[test]
fn top_skeleton_homology_is_not_exported() {
    let diagram = compute(&[1.; 6], 4, 1, None);
    assert_eq!(diagram.intervals_in_dimension(1).unwrap().count(), 0);
    assert!(diagram.intervals().iter().all(|bar| bar.dimension() <= 1));
    assert_eq!(diagram.intervals().len(), 4);
}

#[test]
fn zero_cutoff_merges_duplicate_vertices_without_zero_length_bars() {
    let diagram = compute(&[0., 1., 1.], 3, 1, Some(0.));
    assert_eq!(diagram.intervals(), vec![alive(0, 0., Some(0.)); 2]);
}

#[test]
fn complete_bipartite_filtrations_preserve_quadratic_interval_multiplicity() {
    // At t=1 this is K_(a,b), with no triangles and E-V+1 independent cycles.
    // At t=2 it becomes a full simplex, killing all cycles simultaneously.
    for (a, b) in [(1, 1), (2, 3), (4, 4), (8, 8), (16, 16)] {
        let n = a + b;
        let values: Vec<_> = (0..n)
            .flat_map(|i| (0..i).map(move |j| if (i < a) != (j < a) { 1.0 } else { 2.0 }))
            .collect();
        for cutoff in [None, Some(1.0)] {
            let complete = cutoff.is_none() || n == 2;
            let mut expected = vec![finite(0, 0.0, 1.0); n - 1];
            expected.push(alive(0, 0.0, if complete { None } else { cutoff }));
            let h1 = if complete {
                finite(1, 1.0, 2.0)
            } else {
                alive(1, 1.0, cutoff)
            };
            expected.extend(vec![h1; (a - 1) * (b - 1)]);
            let diagram = compute(&values, n, 1, cutoff);
            assert_eq!(diagram.intervals(), expected);
            assert_eq!(
                diagram.coverage(),
                if complete {
                    Coverage::Complete
                } else {
                    Coverage::Through(1.0)
                }
            );
        }
    }
}

fn betti(diagram: &PersistenceDiagram, dimension: usize, t: f64) -> usize {
    diagram
        .intervals_in_dimension(dimension)
        .unwrap()
        .filter(|bar| {
            bar.birth() <= t
                && match bar.end() {
                    IntervalEnd::Finite(death) => t < death,
                    IntervalEnd::Essential => true,
                    IntervalEnd::RightCensored { through } => t <= through,
                }
        })
        .count()
}

// Independent dense F2 rank oracle. Enumerate vertex subsets, not production simplices.
fn rank(mut columns: Vec<u64>) -> usize {
    let mut rank = 0;
    for row in (0..64).rev() {
        if let Some(pivot) = (rank..columns.len()).find(|&j| columns[j] & (1_u64 << row) != 0) {
            columns.swap(rank, pivot);
            for j in rank + 1..columns.len() {
                if columns[j] & (1_u64 << row) != 0 {
                    columns[j] ^= columns[rank];
                }
            }
            rank += 1;
        }
    }
    rank
}

fn oracle(input: DissimilarityView<'_>, t: f64) -> [usize; 2] {
    let mut cells: [Vec<usize>; 3] = Default::default();
    for mask in 1_usize..(1 << input.len()) {
        let size = mask.count_ones() as usize;
        if size > 3 {
            continue;
        }
        let vertices: Vec<_> = (0..input.len()).filter(|i| mask & (1 << i) != 0).collect();
        if vertices
            .iter()
            .all(|&i| vertices.iter().all(|&j| input.get(i, j).unwrap() <= t))
        {
            cells[size - 1].push(mask);
        }
    }
    let boundary_rank = |dimension: usize| {
        let columns = cells[dimension]
            .iter()
            .map(|&mask| {
                let mut column = 0_u64;
                for vertex in 0..input.len() {
                    if mask & (1 << vertex) != 0 {
                        let face = mask ^ (1 << vertex);
                        let row = cells[dimension - 1]
                            .iter()
                            .position(|&other| other == face)
                            .unwrap();
                        column |= 1 << row;
                    }
                }
                column
            })
            .collect();
        rank(columns)
    };
    let rank1 = boundary_rank(1);
    let rank2 = boundary_rank(2);
    [cells[0].len() - rank1, cells[1].len() - rank1 - rank2]
}

#[test]
fn diagrams_match_independent_ranks_and_truncation_on_small_nonmetric_inputs() {
    let mut state = 713_u64;
    for n in 2..=6 {
        for _ in 0..20 {
            let values: Vec<_> = (0..n * (n - 1) / 2)
                .map(|_| {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    ((state >> 32) % 5) as f64
                })
                .collect();
            let input = DissimilarityView::new(&values, n).unwrap();
            let full = compute(&values, n, 1, None);
            for cutoff in [0., 1., 2., 3., 4.] {
                let partial = compute(&values, n, 1, Some(cutoff));
                for t in [0., 0.5, 1., 1.5, 2., 2.5, 3., 3.5, 4.] {
                    let expected = oracle(input, t);
                    for (dimension, &expected) in expected.iter().enumerate() {
                        assert_eq!(betti(&full, dimension, t), expected);
                        if t <= cutoff {
                            assert_eq!(betti(&partial, dimension, t), expected);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn vertex_permutations_and_scaling_preserve_the_expected_diagram() {
    let values = [1., 3., 2., 2., 4., 1.];
    let input = DissimilarityView::new(&values, 4).unwrap();
    let expected = compute(&values, 4, 1, None);
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let order = [a, b, c, d];
                    if (0..4).any(|i| (i + 1..4).any(|j| order[i] == order[j])) {
                        continue;
                    }
                    let mut permuted = Vec::new();
                    for i in 0..4 {
                        for j in 0..i {
                            permuted.push(input.get(order[i], order[j]).unwrap());
                        }
                    }
                    assert_eq!(compute(&permuted, 4, 1, None), expected);
                }
            }
        }
    }
    let scaled: Vec<_> = values.iter().map(|d| 3.0 * d).collect();
    let expected_bars: Vec<_> = expected
        .intervals()
        .iter()
        .map(|bar| match bar.end() {
            IntervalEnd::Finite(death) => finite(bar.dimension(), 3. * bar.birth(), 3. * death),
            IntervalEnd::Essential => alive(bar.dimension(), 3. * bar.birth(), None),
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(compute(&scaled, 4, 1, None).intervals(), expected_bars);
}

// Exhaustive partial matching, including the diagonal, independent of PH reduction.
fn matches_within(
    a: &[(f64, f64)],
    b: &[(f64, f64)],
    epsilon: f64,
    index: usize,
    used: usize,
) -> bool {
    if index == a.len() {
        return b
            .iter()
            .enumerate()
            .all(|(j, &(birth, death))| used & (1 << j) != 0 || (death - birth) / 2.0 <= epsilon);
    }
    let (birth, death) = a[index];
    if (death - birth) / 2.0 <= epsilon && matches_within(a, b, epsilon, index + 1, used) {
        return true;
    }
    b.iter()
        .enumerate()
        .any(|(j, &(other_birth, other_death))| {
            used & (1 << j) == 0
                && (birth - other_birth).abs().max((death - other_death).abs()) <= epsilon
                && matches_within(a, b, epsilon, index + 1, used | (1 << j))
        })
}

#[test]
fn bounded_distance_perturbations_obey_the_diagram_stability_bound() {
    let finite_bars = |diagram: &PersistenceDiagram, dimension| {
        diagram
            .intervals_in_dimension(dimension)
            .unwrap()
            .filter_map(|bar| match bar.end() {
                IntervalEnd::Finite(death) => Some((bar.birth(), death)),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let mut state = 43_u64;
    for n in 2..=5 {
        for _ in 0..20 {
            let mut base = Vec::new();
            let mut perturbed = Vec::new();
            for _ in 0..n * (n - 1) / 2 {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let value = ((state >> 32) % 5) as f64;
                let delta = if state & 1 == 0 { -0.125 } else { 0.125 };
                base.push(value);
                perturbed.push((value + delta).max(0.0));
            }
            let a = compute(&base, n, 1, None);
            let b = compute(&perturbed, n, 1, None);
            for dimension in 0..=1 {
                assert!(matches_within(
                    &finite_bars(&a, dimension),
                    &finite_bars(&b, dimension),
                    0.125,
                    0,
                    0
                ));
            }
            // Both complete, nonempty inputs have the same essential H0 interval.
            for diagram in [&a, &b] {
                let essential: Vec<_> = diagram
                    .intervals()
                    .iter()
                    .filter(|bar| bar.end() == IntervalEnd::Essential)
                    .copied()
                    .collect();
                assert_eq!(essential, vec![alive(0, 0., None)]);
            }
        }
    }
}
