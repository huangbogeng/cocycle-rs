//! Inspect an octahedral Rips sphere and compute its H2 persistence over F2.
use cocycle::diagram::IntervalEnd;
use cocycle::filtration::threshold_rips_from_distances;
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout};
use cocycle::persistence::{ExecutionLimits, PersistenceOptions, compute_expanded_rips};

fn main() -> cocycle::Result<()> {
    // Three pairs of opposite vertices. Other pairs have distance one.
    let values: Vec<_> = (0..6)
        .flat_map(|b| (0..b).map(move |a| if a / 2 == b / 2 { 2. } else { 1. }))
        .collect();
    let input = DissimilarityMatrixView::new(&values, 6, MatrixLayout::LowerTriangle)?;
    let graph = threshold_rips_from_distances(input, None)?;
    // Tetrahedra (dimension 3) are needed to determine deaths in H2.
    let expansion = graph.expand(3)?;
    let complex = expansion.complex();
    let triangle = complex.find(&[0, 2, 4]).unwrap();
    println!(
        "{} simplices; triangle boundary: {:?}",
        complex.len(),
        complex.boundary(triangle).unwrap()
    );
    let result = compute_expanded_rips(
        &expansion,
        &PersistenceOptions::new(2, None)?,
        &ExecutionLimits::default(),
    )?;
    let sphere = result.diagram().intervals_in_dimension(2)?.next().unwrap();
    assert_eq!(sphere.birth(), 1.);
    assert_eq!(sphere.end(), IntervalEnd::Finite(2.));
    println!(
        "H2 sphere: born at {}, ends at {:?}",
        sphere.birth(),
        sphere.end()
    );
    Ok(())
}
