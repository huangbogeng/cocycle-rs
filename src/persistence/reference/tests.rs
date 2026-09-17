use super::complex::Simplex;
use super::*;
use crate::geometry::DissimilarityView;
use explicit::{ExplicitFiltration, FilteredSimplex};

#[test]
fn closed_square_filtration_has_expected_skeletons() {
    let values = [1.0, 2.0_f64.sqrt(), 1.0, 1.0, 2.0_f64.sqrt(), 1.0];
    let input = DissimilarityView::new(&values, 4).unwrap();
    assert_eq!(rips::build(input, 1, 0.5).unwrap().len(), 4);
    assert_eq!(rips::build(input, 1, 1.0).unwrap().len(), 8);
    assert_eq!(rips::build(input, 1, input.diameter()).unwrap().len(), 14);
    assert_eq!(rips::build(input, 0, input.diameter()).unwrap().len(), 10);
}

#[test]
fn boundaries_are_ordered_and_square_to_zero() {
    let input = DissimilarityView::new(&[1.0; 10], 5).unwrap();
    let filtration = rips::build(input, 1, 1.0).unwrap();
    let mut boundary = vec![usize::MAX];
    let mut face_boundary = Vec::new();
    for j in 0..filtration.len() {
        filtration.write_boundary(j, &mut boundary).unwrap();
        assert!(boundary.windows(2).all(|pair| pair[0] < pair[1]));
        let mut twice = vec![false; filtration.len()];
        for &i in &boundary {
            assert!(i < j);
            assert_eq!(filtration.dimension(i) + 1, filtration.dimension(j));
            assert!(filtration.value(i) <= filtration.value(j));
            filtration.write_boundary(i, &mut face_boundary).unwrap();
            for &k in &face_boundary {
                twice[k] ^= true;
            }
        }
        assert!(twice.iter().all(|&entry| !entry));
    }
}

#[test]
fn invalid_explicit_filtrations_are_rejected() {
    let vertex = |i, value| FilteredSimplex {
        simplex: Simplex::Vertex(i),
        value,
    };
    let edge = FilteredSimplex {
        simplex: Simplex::Edge([0, 1]),
        value: 1.0,
    };
    for cells in [
        vec![edge],
        vec![vertex(0, 0.0), vertex(0, 0.0)],
        vec![vertex(0, 0.0), vertex(1, 2.0), edge],
        vec![vertex(0, f64::NAN)],
        vec![FilteredSimplex {
            simplex: Simplex::Edge([1, 0]),
            value: 0.0,
        }],
    ] {
        assert!(ExplicitFiltration::new(cells).is_err());
    }
}
