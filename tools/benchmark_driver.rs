//! Private benchmark process. Binary fixtures are written by benchmark_gudhi.py.
use std::hint::black_box;
use std::io::{self, Read};
use std::time::Instant;

use cocycle::diagram::{Coverage, IntervalEnd, PersistenceDiagram};
use cocycle::geometry::{DissimilarityView, PointCloudView};
use cocycle::persistence::{RipsOptions, rips_from_dissimilarities, rips_from_points};

struct Input {
    mode: u64,
    n: usize,
    d: usize,
    options: RipsOptions,
    values: Vec<f64>,
}

fn read_input(path: &str) -> Result<Input, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    if bytes.len() < 48 || &bytes[..8] != b"COCYCLE1" || !(bytes.len() - 48).is_multiple_of(8) {
        return Err(io::Error::other("invalid benchmark fixture").into());
    }
    let integer = |i| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let mode = integer(8);
    if mode > 1 {
        return Err(io::Error::other("invalid input mode").into());
    }
    let cutoff = f64::from_le_bytes(bytes[40..48].try_into()?);
    Ok(Input {
        mode,
        n: integer(16).try_into()?,
        d: integer(24).try_into()?,
        options: RipsOptions::new(
            integer(32).try_into()?,
            (!cutoff.is_nan()).then_some(cutoff),
        )?,
        values: bytes[48..]
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()))
            .collect(),
    })
}

impl Input {
    fn compute(&self) -> cocycle::Result<PersistenceDiagram> {
        if self.mode == 0 {
            rips_from_dissimilarities(DissimilarityView::new(&self.values, self.n)?, &self.options)
        } else {
            rips_from_points(
                PointCloudView::new(&self.values, self.n, self.d)?,
                &self.options,
            )
        }
    }
}

fn rss(field: &str) -> Option<u64> {
    let mut text = String::new();
    std::fs::File::open("/proc/self/status")
        .ok()?
        .read_to_string(&mut text)
        .ok()?;
    text.lines().find_map(|line| {
        let tail = line.strip_prefix(field)?;
        tail.split_whitespace().next()?.parse().ok()
    })
}

fn json_option(value: Option<u64>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err(io::Error::other("usage: benchmark_driver FIXTURE SAMPLES ITERATIONS").into());
    }
    let input = read_input(&args[0])?;
    let samples: usize = args[1].parse()?;
    let iterations: usize = args[2].parse()?;
    if samples == 0 || iterations == 0 {
        return Err(io::Error::other("samples and iterations must be positive").into());
    }
    let rss_before = rss("VmRSS:");
    let hwm_before = rss("VmHWM:");
    // One untimed warmup also supplies the diagram checked by the controller.
    let diagram = input.compute()?;
    let mut times = Vec::with_capacity(samples);
    for _ in 0..samples {
        let start = Instant::now();
        for _ in 0..iterations {
            drop(black_box(input.compute()?));
        }
        times.push(start.elapsed().as_secs_f64() * 1000.0 / iterations as f64);
    }
    let peak = rss("VmHWM:");
    print!(
        "{{\"samples_ms\":{times:?},\"rss_before_kib\":{},\"hwm_before_kib\":{},\"peak_rss_kib\":{},",
        json_option(rss_before),
        json_option(hwm_before),
        json_option(peak),
    );
    match diagram.coverage() {
        Coverage::Complete => print!("\"coverage\":[\"complete\",null],"),
        Coverage::Through(t) => print!("\"coverage\":[\"through\",{t}],"),
    }
    print!("\"intervals\":[");
    for (i, bar) in diagram.intervals().iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        let (kind, endpoint) = match bar.end() {
            IntervalEnd::Finite(d) => ("F", d),
            IntervalEnd::Essential => ("E", 0.0),
            IntervalEnd::RightCensored { through } => ("C", through),
        };
        print!(
            "[{},{},\"{kind}\",{endpoint}]",
            bar.dimension(),
            bar.birth()
        );
    }
    println!("]}}");
    Ok(())
}
