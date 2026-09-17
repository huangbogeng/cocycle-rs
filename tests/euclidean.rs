//! Euclidean distance stability, geometric invariance and independent calls.

use cocycle::Error;
use cocycle::descriptors::betti_curve;
use cocycle::diagram::{Coverage, IntervalEnd};
use cocycle::geometry::{DissimilarityView, PointCloudView};
use cocycle::persistence::{RipsOptions, rips_from_dissimilarities, rips_from_points};

#[test]
fn square_matches_distances_and_survives_release_of_coordinate_storage() {
    let diagram = {
        let points = vec![0., 0., 1., 0., 1., 1., 0., 1.];
        rips_from_points(
            PointCloudView::new(&points, 4, 2).unwrap(),
            &RipsOptions::default(),
        )
        .unwrap()
    };
    let diagonal = 2.0_f64.sqrt();
    let values = [1., diagonal, 1., 1., diagonal, 1.];
    let from_distances = rips_from_dissimilarities(
        DissimilarityView::new(&values, 4).unwrap(),
        &RipsOptions::default(),
    )
    .unwrap();
    assert_eq!(diagram, from_distances);
    assert_eq!(
        betti_curve(&diagram, 0, &[0., 1., diagonal, 2.]).unwrap(),
        [4, 1, 1, 1]
    );
    assert_eq!(
        betti_curve(&diagram, 1, &[0., 1., diagonal, 2.]).unwrap(),
        [0, 1, 0, 0]
    );
    let complete_with_cap = rips_from_dissimilarities(
        DissimilarityView::new(&values, 4).unwrap(),
        &RipsOptions::new(1, Some(2.)).unwrap(),
    )
    .unwrap();
    assert_eq!(complete_with_cap.coverage(), Coverage::Complete);
    assert_eq!(betti_curve(&complete_with_cap, 1, &[3.]).unwrap(), [0]);
}

#[test]
fn stable_norm_handles_large_and_tiny_representable_distances() {
    for scale in [1e154, 1e-200] {
        let coordinates = [0., 0., 3. * scale, 4. * scale];
        let diagram = rips_from_points(
            PointCloudView::new(&coordinates, 2, 2).unwrap(),
            &RipsOptions::new(0, None).unwrap(),
        )
        .unwrap();
        let IntervalEnd::Finite(death) = diagram.intervals()[0].end() else {
            panic!()
        };
        assert!((death / (5. * scale) - 1.).abs() < 1e-14);
    }
    for coordinates in [vec![-f64::MAX, f64::MAX], vec![0., 0., f64::MAX, f64::MAX]] {
        assert!(matches!(
            rips_from_points(
                PointCloudView::new(&coordinates, 2, coordinates.len() / 2).unwrap(),
                &RipsOptions::default()
            ),
            Err(Error::NumericalFailure { .. })
        ));
    }
}

#[test]
fn empty_singleton_and_duplicate_clouds_remain_distinct_inputs() {
    for (coordinates, n, expected) in [(vec![], 0, 0), (vec![0., 0.], 1, 1), (vec![0.; 6], 3, 1)] {
        let diagram = rips_from_points(
            PointCloudView::new(&coordinates, n, 2).unwrap(),
            &RipsOptions::default(),
        )
        .unwrap();
        assert_eq!(diagram.intervals().len(), expected);
        assert_eq!(diagram.coverage(), Coverage::Complete);
    }
}

#[test]
fn translation_rotation_and_concurrent_calls_preserve_diagrams() {
    let original = [0., 0., 3., 0., 3., 4., 0., 4.];
    let transformed: Vec<_> = original
        .as_chunks::<2>()
        .0
        .iter()
        .flat_map(|p| [10. - p[1], 7. + p[0]])
        .collect();
    let options = RipsOptions::default();
    let input = PointCloudView::new(&original, 4, 2).unwrap();
    let expected = rips_from_points(input, &options).unwrap();
    assert_eq!(
        rips_from_points(PointCloudView::new(&transformed, 4, 2).unwrap(), &options).unwrap(),
        expected
    );
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| scope.spawn(|| rips_from_points(input, &options).unwrap()))
            .collect();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    });
}
