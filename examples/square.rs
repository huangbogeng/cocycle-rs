//! Rips H0/H1 for a unit square, followed by reusable diagram descriptors.

use cocycle::Result;
use cocycle::descriptors::{betti_curve, finite_lifetime_summary};
use cocycle::geometry::PointCloudView;
use cocycle::persistence::{RipsOptions, rips_from_points};

fn main() -> Result<()> {
    let coordinates = [0., 0., 1., 0., 1., 1., 0., 1.];
    let input = PointCloudView::new(&coordinates, 4, 2)?;
    let diagram = rips_from_points(input, &RipsOptions::default())?;
    println!("Intervals: {:?}", diagram.intervals());
    println!("H1 summary: {:?}", finite_lifetime_summary(&diagram, 1)?);
    println!(
        "H1 Betti curve at [0, 1, sqrt(2), 2]: {:?}",
        betti_curve(&diagram, 1, &[0., 1., 2.0_f64.sqrt(), 2.])?
    );
    Ok(())
}
