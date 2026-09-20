//! Construct and inspect an exact Rips graph, then compute its persistence.
use cocycle::descriptors::betti_curve;
use cocycle::filtration::threshold_rips_from_points;
use cocycle::geometry::PointCloudView;
use cocycle::persistence::{ExecutionLimits, PersistenceOptions, compute_threshold_rips};
fn main() -> cocycle::Result<()> {
    let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
    let graph = threshold_rips_from_points(PointCloudView::new(&coordinates, 4, 2)?, Some(1.))?;
    println!(
        "{} vertices, {} edges; {:?}",
        graph.graph().vertex_count(),
        graph.graph().edge_count(),
        graph.coverage()
    );
    let result = compute_threshold_rips(
        &graph,
        &PersistenceOptions::default(),
        &ExecutionLimits::default(),
    )?;
    println!(
        "H1 Betti curve: {:?}",
        betti_curve(result.diagram(), 1, &[0., 1.])?
    );
    assert_eq!(betti_curve(result.diagram(), 1, &[0., 1.])?, [0, 1]);
    Ok(())
}
