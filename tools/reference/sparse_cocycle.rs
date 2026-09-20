//! Sparse Rips native worker: topology, implicit/explicit persistence and basis parity.
use cocycle::algebra::PrimeField;
use cocycle::diagram::{Coverage, IntervalEnd};
use cocycle::filtration::{SparseRipsOptions, sparse_rips_from_distances};
use cocycle::geometry::{DissimilarityMatrixView, MatrixLayout, MetricPolicy};
use cocycle::persistence::{
    ExecutionLimits, PersistenceOptions, RepresentativeRequest, RepresentativeSelection,
    compute_expanded_sparse_rips, compute_sparse_rips, compute_sparse_rips_with_representatives,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err("usage: sparse_cocycle FIXTURE EPSILON MIN_RADIUS START DIMENSION".into());
    }
    let file = std::fs::read_to_string(&args[1])?;
    let mut words = file.split_whitespace();
    if words.next() != Some("dense") {
        return Err("dense fixture required".into());
    }
    let n: usize = words.next().ok_or("n")?.parse()?;
    let q: usize = words.next().ok_or("q")?.parse()?;
    let cutoff = words.next().ok_or("cutoff")?;
    let cutoff = if cutoff == "none" {
        None
    } else {
        Some(cutoff.parse()?)
    };
    let count: usize = words.next().ok_or("count")?.parse()?;
    let characteristic: u32 = words.next().ok_or("field")?.parse()?;
    let values: Vec<f64> = words.map(str::parse).collect::<Result<_, _>>()?;
    if values.len() != count {
        return Err("count".into());
    }
    let mut options = SparseRipsOptions::new(args[2].parse()?, MetricPolicy::Check)?
        .with_min_insertion_radius(args[3].parse()?)?
        .with_max_scale(cutoff)?;
    if n > 0 {
        options = options.with_start_vertex(args[4].parse()?);
    }
    let input = sparse_rips_from_distances(
        DissimilarityMatrixView::new(&values, n, MatrixLayout::LowerTriangle)?,
        &options,
    )?;
    let dimension: usize = args[5].parse()?;
    let expanded = input.expand(dimension)?;
    let compute = dimension > q || dimension >= n;
    let result = if compute {
        let opts = PersistenceOptions::new(q, None)?.with_field(PrimeField::new(characteristic)?);
        let limits = ExecutionLimits::default();
        let implicit = compute_sparse_rips(&input, &opts, &limits)?;
        if implicit.diagram() != compute_expanded_sparse_rips(&expanded, &opts, &limits)?.diagram()
        {
            return Err("explicit disagreement".into());
        }
        let scale = match input.coverage() {
            Coverage::Through(t) => t.min(25.),
            Coverage::Complete => 25.,
        };
        let requests: Vec<_> = (0..=q)
            .map(|d| RepresentativeRequest::new(d, scale, RepresentativeSelection::Both))
            .collect::<Result<_, _>>()?;
        if implicit.diagram()
            != compute_sparse_rips_with_representatives(&input, &opts, &requests, &limits)?
                .diagram()
        {
            return Err("representative disagreement".into());
        }
        Some(implicit)
    } else {
        None
    };
    print!("{{\"status\":\"ok\",\"characteristic\":{characteristic},\"simplices\":[");
    for (i, simplex) in expanded.complex().simplices().iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!("[{:?},{:?}]", simplex.vertices(), simplex.value());
    }
    print!("],\"intervals\":[");
    if let Some(result) = &result {
        for (i, interval) in result.diagram().intervals().iter().enumerate() {
            if i > 0 {
                print!(",");
            }
            print!("[{},{:?},", interval.dimension(), interval.birth());
            match interval.end() {
                IntervalEnd::Finite(d) => print!("{d:?}"),
                _ => print!("null"),
            };
            print!("]");
        }
    }
    print!("],\"coverage\":");
    match input.coverage() {
        Coverage::Complete => print!("null"),
        Coverage::Through(t) => print!("{t:?}"),
    };
    println!("}}");
    print!(
        "{{\"permutation\":{:?},\"radii\":[",
        input.approximation().permutation()
    );
    for (i, radius) in input.approximation().insertion_radii().iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        match radius {
            Some(r) => print!("{r:?}"),
            None => print!("null"),
        };
    }
    println!("]}}");
    Ok(())
}
