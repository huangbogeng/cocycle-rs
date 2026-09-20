//! Public contracts for matrix.

use cocycle::Error;
use cocycle::geometry::{DissimilarityMatrixView as Matrix, DissimilarityView, MatrixLayout::*};
use cocycle::persistence::{ExecutionLimits, PersistenceOptions, compute_rips_from_distances};

#[test]
fn all_layouts_borrow_and_compute_the_same_filtration() {
    let lower = [1., 4., 2., 3., 5., 6.];
    let upper = [1., 4., 3., 2., 5., 6.];
    let square = [
        0., 1., 4., 3., 1., 0., 2., 5., 4., 2., 0., 6., 3., 5., 6., 0.,
    ];
    let original = DissimilarityView::new(&lower, 4).unwrap();
    let baseline = compute_rips_from_distances(
        original.into(),
        &PersistenceOptions::default(),
        &ExecutionLimits::default(),
    )
    .unwrap();
    for (values, layout) in [
        (&lower[..], LowerTriangle),
        (&upper[..], UpperTriangle),
        (&square[..], Square),
    ] {
        let matrix = Matrix::new(values, 4, layout).unwrap();
        assert_eq!(matrix.values().as_ptr(), values.as_ptr());
        for a in 0..4 {
            for b in 0..4 {
                assert_eq!(matrix.get(a, b), original.get(a, b));
            }
        }
        assert_eq!(matrix.get(4, 0), None);
        assert_eq!(
            compute_rips_from_distances(
                matrix,
                &PersistenceOptions::default(),
                &ExecutionLimits::default()
            )
            .unwrap(),
            baseline
        );
    }
    assert_eq!(original.values(), &lower);
}

#[test]
fn shapes_structure_and_numeric_errors_are_checked() {
    assert!(matches!(
        Matrix::new(&[], usize::MAX, Square),
        Err(Error::SizeOverflow { .. })
    ));
    assert!(Matrix::new(&[1.], 3, LowerTriangle).is_err());
    for values in [[1., 2., 2., 0.], [0., 2., 3., 0.]] {
        assert!(matches!(
            Matrix::new(&values, 2, Square),
            Err(Error::InvalidMatrix { .. })
        ));
    }
    for value in [f64::NAN, f64::INFINITY, -1.] {
        assert!(Matrix::new(&[value], 2, UpperTriangle).is_err());
    }
    let values = [-0., -0., 0., 0.];
    let view = Matrix::new(&values, 2, Square).unwrap();
    assert_eq!(view.get(0, 1).unwrap().to_bits(), 0);
    assert_eq!(view.values()[0].to_bits(), (-0.0f64).to_bits());
    for n in [0, 1] {
        assert_eq!(Matrix::new(&[], n, UpperTriangle).unwrap().len(), n);
    }
    assert!(Matrix::new(&[], 1, Square).is_err());
}
