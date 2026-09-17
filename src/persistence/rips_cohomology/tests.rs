use super::*;
use crate::diagram::{Coverage, IntervalEnd};
use crate::persistence::reference::FilteredBoundary;
use crate::persistence::{RipsOptions, finish, range, reference};

fn compare_all(values: &[f64], n: usize, cutoff: Option<f64>) {
    let input = DissimilarityView::new(values, n).unwrap();
    let options = RipsOptions::new(1, cutoff).unwrap();
    let expected = reference::compute(input, &options).unwrap();
    let (cutoff, coverage) = range(input, &options);
    macro_rules! check {
        ($implicit:literal, $clear:literal, $cone:literal, $short:literal) => {{
            let raw = run::<$implicit, $clear, $cone, $short>(input, cutoff, &mut Stats::default())
                .unwrap();
            assert_eq!(
                finish(1, coverage, raw).unwrap(),
                expected,
                "n={n} cutoff={cutoff} implicit={} clear={} cone={} shortcuts={} values={values:?}",
                $implicit,
                $clear,
                $cone,
                $short
            );
        }};
    }
    check!(false, false, false, 0);
    check!(false, true, false, 0);
    check!(true, true, false, 0);
    check!(true, true, true, 0);
    check!(true, true, true, 1);
    check!(true, true, true, 2);
    check!(true, true, true, 3);
}

#[test]
fn every_three_level_four_vertex_filtration_matches_reference_with_each_optimization() {
    for code in 0..3_usize.pow(6) {
        let mut remaining = code;
        let values: Vec<_> = (0..6)
            .map(|_| {
                let value = (remaining % 3) as f64;
                remaining /= 3;
                value
            })
            .collect();
        for cutoff in [None, Some(0.0), Some(1.0), Some(1.5)] {
            compare_all(&values, 4, cutoff);
        }
    }
}

#[test]
fn randomized_f64_nonmetric_filtrations_and_ties_match_each_optimization() {
    let mut state = 173_u64;
    for n in 0_usize..=12 {
        for sample in 0..24 {
            let values: Vec<_> = (0..n * n.saturating_sub(1) / 2)
                .map(|_| {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    if sample % 2 == 0 {
                        ((state >> 32) % 9) as f64 / 4.0
                    } else {
                        ((state >> 11) as f64) / ((1_u64 << 52) as f64)
                    }
                })
                .collect();
            for cutoff in [None, Some(0.0), Some(0.75), Some(1.0), Some(1.8)] {
                compare_all(&values, n, cutoff);
            }
        }
    }
}

#[test]
fn extreme_scales_and_adjacent_f64_values_do_not_quantize_or_overflow() {
    for base in [f64::from_bits(1), 1.0, 1.0e300] {
        let next = f64::from_bits(base.to_bits() + 1);
        let values = [base, next, base, base, next, base];
        for cutoff in [None, Some(base), Some(next)] {
            compare_all(&values, 4, cutoff);
        }
    }
    compare_all(&[-0.0, f64::MAX, 0.0, f64::MAX, f64::MAX, -0.0], 4, None);
}

#[test]
fn cone_stopping_preserves_user_coverage_and_closed_boundary() {
    // Vertex zero is a cone point at 1, while the full input diameter is 5.
    let values = [1., 1., 5., 1., 5., 5.];
    let input = DissimilarityView::new(&values, 4).unwrap();
    assert_eq!(cone_radius(input), 1.0);
    for cutoff in [0.5, 1.0, 2.0, 5.0] {
        compare_all(&values, 4, Some(cutoff));
        let options = RipsOptions::new(1, Some(cutoff)).unwrap();
        let result = crate::persistence::rips_from_dissimilarities(input, &options).unwrap();
        if (1.0..5.0).contains(&cutoff) {
            assert_eq!(result.coverage(), Coverage::Through(cutoff));
            assert_eq!(result.intervals().len(), 4);
            assert!(
                result
                    .intervals()
                    .iter()
                    .any(|bar| bar.end() == IntervalEnd::RightCensored { through: cutoff })
            );
        }
    }
}

#[test]
fn large_cycles_and_disconnected_components_survive_truncation() {
    for n in [16, 24] {
        let values: Vec<_> = (0..n)
            .flat_map(|b| {
                (0..b).map(move |a| {
                    // Two disjoint cycle graphs until scale 3; a complete graph at 4.
                    let half = n / 2;
                    if a / half != b / half {
                        4.0
                    } else if b - a == 1 || b - a == half - 1 {
                        1.0
                    } else {
                        3.0
                    }
                })
            })
            .collect();
        for cutoff in [None, Some(0.0), Some(1.0), Some(2.0), Some(3.0)] {
            compare_all(&values, n, cutoff);
        }
    }
}

#[test]
fn heap_entries_cancel_by_parity_instead_of_set_deduplication() {
    let mut heap = BinaryHeap::from(vec![5, 5, 4, 4, 4, 2, 2]);
    assert_eq!(pop_parity(&mut heap), Some(4));
    assert_eq!(pop_parity(&mut heap), None);
}

struct Matrix(Vec<Vec<usize>>);
impl FilteredBoundary for Matrix {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn dimension(&self, _: usize) -> usize {
        unreachable!()
    }
    fn value(&self, _: usize) -> f64 {
        unreachable!()
    }
    fn write_boundary(&self, j: usize, out: &mut Vec<usize>) -> Result<()> {
        out.clear();
        out.extend_from_slice(&self.0[j]);
        Ok(())
    }
}

#[test]
fn reversed_transpose_pairs_and_unpaired_indices_map_back_to_boundary_reduction() {
    // Enumerate subsets independently of production combinatorial indexing.
    for n in 1_usize..=6 {
        let values: Vec<_> = (0..n * (n - 1) / 2)
            .map(|i| ((i * 13) % 5) as f64)
            .collect();
        let input = DissimilarityView::new(&values, n).unwrap();
        for cutoff in [0.0, 2.0, 4.0] {
            let mut cells: Vec<_> = (1_usize..(1 << n))
                .filter(|mask| mask.count_ones() <= 3)
                .map(|mask| {
                    let v: Vec<_> = (0..n).filter(|i| mask & (1 << i) != 0).collect();
                    let value = v
                        .iter()
                        .flat_map(|&a| v.iter().map(move |&b| input.get(a, b).unwrap()))
                        .fold(0.0_f64, f64::max);
                    (mask, v.len(), value)
                })
                .filter(|x| x.2 <= cutoff)
                .collect();
            // Within a fixed dimension, mask order equals combinatorial order.
            cells.sort_by(|a, b| a.2.total_cmp(&b.2).then(a.1.cmp(&b.1)).then(b.0.cmp(&a.0)));
            let size = cells.len();
            let d = Matrix(
                cells
                    .iter()
                    .map(|&(mask, dim, _)| {
                        cells
                            .iter()
                            .enumerate()
                            .filter_map(|(i, &(face, fdim, _))| {
                                (fdim + 1 == dim && face & mask == face).then_some(i)
                            })
                            .collect()
                    })
                    .collect(),
            );
            let mut c = Matrix(vec![vec![]; size]);
            for (j, rows) in d.0.iter().enumerate() {
                for &i in rows {
                    c.0[size - 1 - i].push(size - 1 - j);
                }
            }
            for rows in &mut c.0 {
                rows.sort_unstable();
            }
            let forward = crate::persistence::reference::reduction::reduce(&d).unwrap();
            let dual = crate::persistence::reference::reduction::reduce(&c).unwrap();
            let mut expected = forward.pairs;
            let mut actual: Vec<_> = dual
                .pairs
                .into_iter()
                .map(|(i, j)| (size - 1 - j, size - 1 - i))
                .collect();
            expected.sort_unstable();
            actual.sort_unstable();
            assert_eq!(actual, expected);
            let mut actual: Vec<_> = dual.unpaired.into_iter().map(|i| size - 1 - i).collect();
            actual.sort_unstable();
            assert_eq!(actual, forward.unpaired);
        }
    }
}

#[test]
fn shortcut_and_reconstruction_paths_are_exercised() {
    let n = 24;
    let values: Vec<_> = (0..n)
        .flat_map(|b| {
            (0..b).map(move |a| {
                let angle = std::f64::consts::PI * (b - a) as f64 / n as f64;
                2.0 * angle.sin()
            })
        })
        .collect();
    let input = DissimilarityView::new(&values, n).unwrap();
    let mut stats = Stats::default();
    let raw = run::<true, true, true, 3>(input, input.diameter(), &mut stats).unwrap();
    let expected = reference::compute(input, &RipsOptions::default()).unwrap();
    assert_eq!(finish(1, Coverage::Complete, raw).unwrap(), expected);
    assert!(
        stats.shortcuts > 0 && stats.column_additions > 0 && stats.stored_entries > 0,
        "{stats:?}"
    );
}
