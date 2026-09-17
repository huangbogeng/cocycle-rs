//! Dependency-free wall-clock benchmarks; see benches/README.md for interpretation.
use std::hint::black_box;
use std::time::Instant;

use cocycle::descriptors::{betti_curve, finite_lifetime_summary};
use cocycle::geometry::{DissimilarityView, PointCloudView};
use cocycle::persistence::{RipsOptions, rips_from_dissimilarities, rips_from_points};

const SAMPLES: usize = 7;

fn selected(name: &str) -> bool {
    std::env::var("COCYCLE_BENCH_CASE").map_or(true, |filter| filter == name)
}

fn measure(name: &str, iterations: usize, mut run: impl FnMut()) {
    // An optional exact-name filter also permits process-level memory measurement.
    if !selected(name) {
        return;
    }
    run();
    let mut milliseconds = [0.0; SAMPLES];
    for elapsed in &mut milliseconds {
        let start = Instant::now();
        for _ in 0..iterations {
            run();
        }
        *elapsed = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    }
    milliseconds.sort_by(f64::total_cmp);
    println!(
        "{name},{iterations},{SAMPLES},{:.6},{:.6},{:.6}",
        milliseconds[0],
        milliseconds[SAMPLES / 2],
        milliseconds[SAMPLES - 1],
    );
}

fn points(n: usize) -> Vec<f64> {
    let mut state = 1729_u64;
    (0..2 * n)
        .map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (state >> 11) as f64 / ((1_u64 << 53) as f64)
        })
        .collect()
}

fn distances(points: &[f64]) -> Vec<f64> {
    let mut result = Vec::new();
    for i in 0..points.len() / 2 {
        for j in 0..i {
            result
                .push((points[2 * i] - points[2 * j]).hypot(points[2 * i + 1] - points[2 * j + 1]));
        }
    }
    result
}

fn main() -> cocycle::Result<()> {
    eprintln!(
        "seed=1729; uniform unit-square points; release profile; times include output destruction"
    );
    println!("case,iterations_per_sample,samples,min_ms,median_ms,max_ms");
    for n in [32, 64, 128] {
        let coordinates = points(n);
        let values = distances(&coordinates);
        let input = DissimilarityView::new(&values, n)?;
        let options = RipsOptions::default();
        measure(&format!("h1_distances_{n}"), 1, || {
            black_box(rips_from_dissimilarities(black_box(input), black_box(&options)).unwrap());
        });
    }

    let coordinates = points(128);
    let cloud = PointCloudView::new(&coordinates, 128, 2)?;
    let options = RipsOptions::default();
    measure("h1_points_128", 1, || {
        black_box(rips_from_points(black_box(cloud), black_box(&options)).unwrap());
    });
    let cutoff = RipsOptions::new(1, Some(0.2))?;
    measure("h1_points_128_cutoff_0.2", 3, || {
        black_box(rips_from_points(black_box(cloud), black_box(&cutoff)).unwrap());
    });

    let tied = vec![1.0; 128 * 127 / 2];
    let tied = DissimilarityView::new(&tied, 128)?;
    measure("h1_equal_distances_128", 1, || {
        black_box(rips_from_dissimilarities(black_box(tied), black_box(&options)).unwrap());
    });

    let larger = points(1024);
    let cloud = PointCloudView::new(&larger, 1024, 2)?;
    let h0 = RipsOptions::new(0, None)?;
    measure("h0_points_1024", 1, || {
        black_box(rips_from_points(black_box(cloud), black_box(&h0)).unwrap());
    });

    if selected("descriptors_h1_128") {
        let diagram = rips_from_points(PointCloudView::new(&coordinates, 128, 2)?, &options)?;
        let grid: Vec<_> = (0..101).map(|i| f64::from(i) / 100.0).collect();
        measure("descriptors_h1_128", 1000, || {
            black_box(finite_lifetime_summary(black_box(&diagram), 1).unwrap());
            black_box(betti_curve(black_box(&diagram), 1, black_box(&grid)).unwrap());
        });
    }
    Ok(())
}
