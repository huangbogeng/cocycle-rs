//! Approximate a point cloud with owned sampling provenance and requested bases.
use cocycle::algebra::PrimeField;
use cocycle::filtration::{SparseRipsOptions, sparse_rips_from_points};
use cocycle::geometry::{MetricPolicy, PointCloudView};
use cocycle::persistence::{
    ExecutionLimits, PersistenceOptions, RepresentativeRequest, RepresentativeSelection,
    compute_sparse_rips_with_representatives,
};
fn main() -> cocycle::Result<()> {
    let points = [0., 0., 1., 0., 1., 1., 0., 1., 0., 0.];
    let input = PointCloudView::new(&points, 5, 2)?;
    let construction =
        sparse_rips_from_points(input, &SparseRipsOptions::new(0.5, MetricPolicy::Check)?)?;
    let options = PersistenceOptions::new(1, None)?.with_field(PrimeField::new(3)?);
    let requests = [RepresentativeRequest::new(
        1,
        1.,
        RepresentativeSelection::Both,
    )?];
    let result = compute_sparse_rips_with_representatives(
        &construction,
        &options,
        &requests,
        &ExecutionLimits::default(),
    )?;
    let metadata = result.context().approximation().unwrap();
    println!(
        "original vertices: {}; retained: {:?}",
        metadata.permutation().len(),
        metadata.retained_vertices()
    );
    println!("conditional ideal-arithmetic bound: {:?}", metadata.bound());
    println!("approximation coverage: {:?}", result.diagram().coverage());
    for interval in result.diagram().intervals() {
        println!("{interval:?}");
    }
    for representative in result.representatives().unwrap() {
        println!("{representative:?}");
    }
    Ok(())
}
