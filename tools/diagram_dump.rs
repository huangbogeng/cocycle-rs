//! Developer-only adapter for compare_ripser.py; not a library API or installed CLI.

use cocycle::diagram::{Coverage, IntervalEnd};
use cocycle::geometry::DissimilarityView;
use cocycle::persistence::{RipsOptions, rips_from_dissimilarities};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let n: usize = args[0].parse()?;
    let dimension: usize = args[1].parse()?;
    let cutoff = if args[2] == "none" {
        None
    } else {
        Some(args[2].parse()?)
    };
    let values: Vec<f64> = args[3..]
        .iter()
        .map(|value| value.parse())
        .collect::<Result<_, _>>()?;
    let diagram = rips_from_dissimilarities(
        DissimilarityView::new(&values, n)?,
        &RipsOptions::new(dimension, cutoff)?,
    )?;
    match diagram.coverage() {
        Coverage::Complete => println!("complete"),
        Coverage::Through(value) => println!("through {value}"),
    }
    for bar in diagram.intervals() {
        let (kind, endpoint) = match bar.end() {
            IntervalEnd::Finite(death) => ("F", death),
            IntervalEnd::Essential => ("E", 0.0),
            IntervalEnd::RightCensored { through } => ("C", through),
        };
        println!("{} {} {kind} {endpoint}", bar.dimension(), bar.birth());
    }
    Ok(())
}
