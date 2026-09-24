//! Scalar diagonal-saving matching, adapted from Topp 1.0.1.
//!
//! Reference: Topp `ffa1da051ca7ac5e313c74cc9fb92a2bcb20c234`,
//! `src/wasserstein.cpp`. Candidate, storage and matcher routing retain the
//! reference thresholds. SIMD and parallel candidate loops use scalar loops.
//!
//! Copyright (c) 2026 Topp contributors
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

use crate::{Error, Result};

mod direct;
mod sparse;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Metric {
    W1,
    W2,
}

/// These switches are consumed by the standalone experimental worker, not the
/// public library API. Forced sparse mode isolates the residual-network change.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Options {
    pub(crate) sparse: SparseLayout,
    pub(crate) force_sparse: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum SparseLayout {
    #[default]
    Vectors,
    // Constructed by the standalone ablation worker and private tests.
    #[allow(dead_code)]
    Arena,
}

#[derive(Debug, Default)]
pub(crate) struct Stats {
    pub(crate) candidate_pairs: usize,
    pub(crate) positive_edges: usize,
    pub(crate) dense_solves: usize,
    pub(crate) sparse_solves: usize,
    pub(crate) augmentations: usize,
    pub(crate) components: usize,
    pub(crate) tiny_components: usize,
    pub(crate) duplicate_groups: usize,
    pub(crate) greedy_certificates: usize,
    pub(crate) scratch_reuses: usize,
    pub(crate) direct_cost_fallbacks: usize,
}

fn allocation() -> Error {
    Error::AllocationFailed {
        context: "Wasserstein workspace",
    }
}

fn numerical() -> Error {
    Error::NumericalFailure {
        context: "Wasserstein distance",
    }
}

fn finite(value: f64) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(numerical())
    }
}

fn restore_scale(value: f64, scale: f64) -> Result<f64> {
    let restored = finite(value * scale)?;
    if value > 0.0 && restored == 0.0 {
        return Err(numerical());
    }
    Ok(restored)
}

fn sum_size(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or(Error::SizeOverflow {
        operation: "Wasserstein workspace size",
    })
}

fn product(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or(Error::SizeOverflow {
        operation: "Wasserstein pair count",
    })
}

fn buffer<T: Clone>(length: usize, value: T) -> Result<Vec<T>> {
    let mut result = Vec::new();
    result.try_reserve_exact(length).map_err(|_| allocation())?;
    result.resize(length, value);
    Ok(result)
}

fn push<T>(target: &mut Vec<T>, value: T) -> Result<()> {
    target.try_reserve(1).map_err(|_| allocation())?;
    target.push(value);
    Ok(())
}

#[derive(Clone, Copy)]
struct Point {
    coordinates: [f64; 2],
    midpoint: f64,
    half: f64,
}

fn prepare(input: &[[f64; 2]], scale: f64) -> Result<Vec<Point>> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(input.len())
        .map_err(|_| allocation())?;
    for &[birth, death] in input {
        let b = birth / scale;
        let d = death / scale;
        // Scaling by a power of two must be reversible: no implicit quantization
        // or collapsed intervals are accepted when the exponent range is wide.
        if !b.is_finite() || !d.is_finite() || b * scale != birth || d * scale != death || b >= d {
            return Err(numerical());
        }
        let half = (d - b) * 0.5;
        if !half.is_finite() || half <= 0.0 {
            return Err(numerical());
        }
        result.push(Point {
            coordinates: [b, d],
            midpoint: b * 0.5 + d * 0.5,
            half,
        });
    }
    Ok(result)
}

fn power_scale(first: &[[f64; 2]], second: &[[f64; 2]]) -> f64 {
    let largest = first
        .iter()
        .chain(second)
        .flatten()
        .fold(0.0_f64, |a, b| a.max(b.abs()));
    if largest == 0.0 || (2.0_f64.powi(-200)..=2.0_f64.powi(200)).contains(&largest) {
        return 1.0;
    }
    let exponent = ((largest.to_bits() >> 52) & 0x7ff) as i32 - 1023;
    // A normal scaling factor also scales the smallest subnormal up to 2^-52.
    2.0_f64.powi(exponent.max(-1022))
}

fn square(value: f64) -> Result<f64> {
    let squared = finite(value * value)?;
    if value != 0.0 && squared == 0.0 {
        return Err(numerical());
    }
    Ok(squared)
}

fn diagonal_power(point: Point, metric: Metric) -> Result<f64> {
    match metric {
        Metric::W1 => Ok(point.half),
        Metric::W2 => finite(2.0 * square(point.half)?),
    }
}

fn cross_power(first: Point, second: Point, metric: Metric) -> Result<f64> {
    let b = finite(first.coordinates[0] - second.coordinates[0])?.abs();
    let d = finite(first.coordinates[1] - second.coordinates[1])?.abs();
    match metric {
        Metric::W1 => Ok(b.max(d)),
        Metric::W2 => finite(square(b)? + square(d)?),
    }
}

fn saving(first: Point, second: Point, metric: Metric) -> Result<(f64, bool)> {
    // Rounded midpoints are only a search index, never the cost authority:
    // adjacent endpoint values need not have a representable midpoint.
    let baseline = finite(diagonal_power(first, metric)? + diagonal_power(second, metric)?)?;
    let cross = cross_power(first, second, metric)?;
    // In this regime, subtracting a small cross cost from the diagonal baseline
    // cannot retain enough significant bits to rank almost equal matchings.
    // Reconstructing the final cost is insufficient: choose the matching itself
    // using original costs. This is a numerical guard, not a changed tolerance
    // or a heuristic substitute for the normal saving-graph solver.
    let direct_cost_required = cross > 0.0 && cross <= 64.0 * f64::EPSILON * baseline;
    Ok((finite(baseline - cross)?, direct_cost_required))
}

#[derive(Clone, Copy, Debug)]
struct Edge {
    column: usize,
    saving: f64,
}

enum Storage {
    Dense(Vec<f64>),
    Csr {
        offsets: Vec<usize>,
        edges: Vec<Edge>,
    },
}

struct Graph {
    rows: usize,
    columns: usize,
    edge_count: usize,
    storage: Storage,
    direct_cost_required: bool,
}

enum Edges<'a> {
    Dense(std::iter::Enumerate<std::slice::Iter<'a, f64>>),
    Csr(std::iter::Copied<std::slice::Iter<'a, Edge>>),
}

impl Iterator for Edges<'_> {
    type Item = Edge;
    fn next(&mut self) -> Option<Edge> {
        match self {
            Self::Dense(iter) => {
                iter.find_map(|(column, &saving)| (saving > 0.0).then_some(Edge { column, saving }))
            }
            Self::Csr(iter) => iter.next(),
        }
    }
}

impl Graph {
    fn edges(&self, row: usize) -> Edges<'_> {
        match &self.storage {
            Storage::Dense(values) => Edges::Dense(
                values[row * self.columns..(row + 1) * self.columns]
                    .iter()
                    .enumerate(),
            ),
            Storage::Csr { offsets, edges } => {
                Edges::Csr(edges[offsets[row]..offsets[row + 1]].iter().copied())
            }
        }
    }

    fn from_candidates(rows: Vec<Vec<Edge>>, columns: usize, force_csr: bool) -> Result<Self> {
        let row_count = rows.len();
        let pairs = product(row_count, columns)?;
        let edge_count = rows.iter().try_fold(0, |n, row| sum_size(n, row.len()))?;
        let density = if pairs == 0 {
            0.0
        } else {
            edge_count as f64 / pairs as f64
        };
        let storage = if !force_csr && (pairs <= 1024 || density >= 0.15) {
            let mut values = buffer(pairs, 0.0)?;
            for (row, edges) in rows.iter().enumerate() {
                for edge in edges {
                    values[row * columns + edge.column] = edge.saving;
                }
            }
            Storage::Dense(values)
        } else {
            let mut offsets = Vec::new();
            offsets
                .try_reserve_exact(sum_size(row_count, 1)?)
                .map_err(|_| allocation())?;
            let mut edges = Vec::new();
            edges
                .try_reserve_exact(edge_count)
                .map_err(|_| allocation())?;
            offsets.push(0);
            for row in rows {
                edges.extend(row);
                offsets.push(edges.len());
            }
            Storage::Csr { offsets, edges }
        };
        Ok(Self {
            rows: row_count,
            columns,
            edge_count,
            storage,
            direct_cost_required: false,
        })
    }

    fn density(&self) -> f64 {
        if self.rows == 0 || self.columns == 0 {
            0.0
        } else {
            self.edge_count as f64 / (self.rows * self.columns) as f64
        }
    }

    fn active(&self) -> Result<(Vec<usize>, Vec<usize>)> {
        let mut rows = Vec::new();
        let mut used = buffer(self.columns, false)?;
        for row in 0..self.rows {
            let mut active = false;
            for edge in self.edges(row) {
                active = true;
                used[edge.column] = true;
            }
            if active {
                push(&mut rows, row)?;
            }
        }
        let mut columns = Vec::new();
        for (column, present) in used.into_iter().enumerate() {
            if present {
                push(&mut columns, column)?;
            }
        }
        Ok((rows, columns))
    }
}

fn generate(first: &[Point], second: &[Point], metric: Metric, stats: &mut Stats) -> Result<Graph> {
    let pairs = product(first.len(), second.len())?;
    let mut candidates = buffer(first.len(), Vec::new())?;
    let mut direct_cost_required = false;
    let mut order = Vec::new();
    order
        .try_reserve_exact(second.len())
        .map_err(|_| allocation())?;
    order.extend(0..second.len());
    order.sort_unstable_by(|&a, &b| {
        second[a]
            .midpoint
            .total_cmp(&second[b].midpoint)
            .then(a.cmp(&b))
    });
    let maximum_half = second.iter().fold(0.0_f64, |a, b| a.max(b.half));
    let midpoint_error = |point: Point| {
        (point.midpoint.next_up() - point.midpoint).max(point.midpoint - point.midpoint.next_down())
    };
    let second_error = second
        .iter()
        .copied()
        .map(midpoint_error)
        .fold(0.0, f64::max);
    let radius = |point: Point| match metric {
        Metric::W1 => (2.0 * point.half).next_up(),
        Metric::W2 => ((2.0 * point.half).next_up() * maximum_half)
            .next_up()
            .sqrt()
            .next_up(),
    };
    let window = |point: Point| {
        let r = (radius(point) + midpoint_error(point)).next_up() + second_error;
        let lower = (point.midpoint - r).next_down();
        let upper = (point.midpoint + r).next_up();
        let start = order.partition_point(|&i| second[i].midpoint < lower);
        let stop = order.partition_point(|&i| second[i].midpoint <= upper);
        start..stop
    };
    let sampled = first.len().min(4);
    let mut window_count = 0;
    for sample in 0..sampled {
        window_count = sum_size(
            window_count,
            window(first[sample * first.len() / sampled]).len(),
        )?;
    }
    let window_density = if sampled == 0 || second.is_empty() {
        0.0
    } else {
        window_count as f64 / product(sampled, second.len())? as f64
    };
    // SIMD/parallel reference routes intentionally use the scalar equivalent.
    let sweep = pairs > 1024 && window_density < 0.75;
    for (row, &point) in first.iter().enumerate() {
        if sweep {
            for &column in &order[window(point)] {
                stats.candidate_pairs += 1;
                let (value, direct) = saving(point, second[column], metric)?;
                direct_cost_required |= direct;
                if value > 0.0 {
                    push(
                        &mut candidates[row],
                        Edge {
                            column,
                            saving: value,
                        },
                    )?;
                }
            }
        } else {
            for (column, &other) in second.iter().enumerate() {
                stats.candidate_pairs += 1;
                let (value, direct) = saving(point, other, metric)?;
                direct_cost_required |= direct;
                if value > 0.0 {
                    push(
                        &mut candidates[row],
                        Edge {
                            column,
                            saving: value,
                        },
                    )?;
                }
            }
        }
        stats.positive_edges += candidates[row].len();
    }
    let mut graph = Graph::from_candidates(candidates, second.len(), false)?;
    graph.direct_cost_required = direct_cost_required;
    Ok(graph)
}

fn certified_greedy(graph: &Graph, stats: &mut Stats) -> Result<Option<Vec<Option<usize>>>> {
    if graph.edge_count == 0 {
        return Ok(None);
    }
    let mut edges = Vec::new();
    edges
        .try_reserve_exact(graph.edge_count)
        .map_err(|_| allocation())?;
    let mut row_max = buffer(graph.rows, 0.0_f64)?;
    let mut column_max = buffer(graph.columns, 0.0_f64)?;
    for (row, maximum) in row_max.iter_mut().enumerate() {
        for edge in graph.edges(row) {
            *maximum = maximum.max(edge.saving);
            column_max[edge.column] = column_max[edge.column].max(edge.saving);
            edges.push((row, edge));
        }
    }
    edges.sort_unstable_by(|a, b| {
        b.1.saving
            .total_cmp(&a.1.saving)
            .then(a.0.cmp(&b.0))
            .then(a.1.column.cmp(&b.1.column))
    });
    let mut matching = buffer(graph.rows, None)?;
    let mut column_match = buffer(graph.columns, None)?;
    let mut weight = buffer(graph.rows, 0.0)?;
    for (row, edge) in edges {
        if matching[row].is_none() && column_match[edge.column].is_none() {
            matching[row] = Some(edge.column);
            column_match[edge.column] = Some(row);
            weight[row] = edge.saving;
        }
    }
    let row_bound = row_max.iter().zip(&weight).all(|(a, b)| a == b);
    let column_bound = column_max
        .iter()
        .zip(&column_match)
        .all(|(&a, &row)| a == 0.0 || row.is_some_and(|r| weight[r] == a));
    if row_bound || column_bound {
        stats.greedy_certificates += 1;
        Ok(Some(matching))
    } else {
        Ok(None)
    }
}

fn dense_sap(graph: &Graph, row_reduction: bool, stats: &mut Stats) -> Result<Vec<Option<usize>>> {
    stats.dense_solves += 1;
    let (rows, columns) = graph.active()?;
    let mut matching = buffer(graph.rows, None)?;
    if rows.is_empty() || columns.is_empty() {
        return Ok(matching);
    }
    let mut lookup;
    let values = match &graph.storage {
        Storage::Dense(values) => values,
        Storage::Csr { .. } => {
            lookup = buffer(product(graph.rows, graph.columns)?, 0.0)?;
            for row in 0..graph.rows {
                for edge in graph.edges(row) {
                    lookup[row * graph.columns + edge.column] = edge.saving;
                }
            }
            &lookup
        }
    };
    let transposed = rows.len() > columns.len();
    let short = rows.len().min(columns.len());
    let long = rows.len().max(columns.len());
    let cost = |r: usize, c: usize| {
        let (row, column) = if transposed {
            (rows[c], columns[r])
        } else {
            (rows[r], columns[c])
        };
        -values[row * graph.columns + column]
    };
    let short_len = sum_size(short, 1)?;
    let long_len = sum_size(long, 1)?;
    let mut u = buffer(short_len, 0.0)?;
    let mut v = buffer(long_len, 0.0)?;
    let mut minimum = buffer(long_len, 0.0)?;
    let mut matched = buffer(long_len, 0)?;
    let mut predecessor = buffer(long_len, 0)?;
    let mut used = buffer(long_len, false)?;
    let mut initially_matched = buffer(short_len, false)?;
    if row_reduction {
        for row in 1..=short {
            let mut best = f64::INFINITY;
            let mut column = 0;
            for c in 1..=long {
                let value = cost(row - 1, c - 1);
                if value < best {
                    best = value;
                    column = c;
                }
            }
            u[row] = best;
            if matched[column] == 0 {
                matched[column] = row;
                initially_matched[row] = true;
            }
        }
    }
    for (row, &already_matched) in initially_matched.iter().enumerate().skip(1) {
        if already_matched {
            continue;
        }
        matched[0] = row;
        let mut column = 0;
        minimum.fill(f64::INFINITY);
        used.fill(false);
        loop {
            used[column] = true;
            let current = matched[column];
            let mut delta = f64::INFINITY;
            let mut next = 0;
            for c in 1..=long {
                if used[c] {
                    continue;
                }
                let reduced = finite(cost(current - 1, c - 1) - u[current] - v[c])?;
                if reduced < minimum[c] {
                    minimum[c] = reduced;
                    predecessor[c] = column;
                }
                if minimum[c] < delta {
                    delta = minimum[c];
                    next = c;
                }
            }
            finite(delta)?;
            for c in 0..=long {
                if used[c] {
                    u[matched[c]] = finite(u[matched[c]] + delta)?;
                    v[c] = finite(v[c] - delta)?;
                } else {
                    minimum[c] -= delta;
                }
            }
            column = next;
            if matched[column] == 0 {
                break;
            }
        }
        loop {
            let previous = predecessor[column];
            matched[column] = matched[previous];
            column = previous;
            if column == 0 {
                break;
            }
        }
        stats.augmentations += 1;
    }
    for column in 1..=long {
        let row = matched[column];
        if row == 0 {
            continue;
        }
        let (r, c) = if transposed {
            (rows[column - 1], columns[row - 1])
        } else {
            (rows[row - 1], columns[column - 1])
        };
        if values[r * graph.columns + c] > 0.0 {
            matching[r] = Some(c);
        }
    }
    Ok(matching)
}

fn solve_graph(
    graph: &Graph,
    dense: Option<bool>,
    greedy: bool,
    metric: Metric,
    options: Options,
    stats: &mut Stats,
) -> Result<Vec<Option<usize>>> {
    if greedy
        && (graph.rows.max(graph.columns) <= 32 || graph.density() <= 0.05)
        && let Some(matching) = certified_greedy(graph, stats)?
    {
        return Ok(matching);
    }
    let use_dense = dense.unwrap_or(graph.rows.max(graph.columns) <= 24 || graph.density() >= 0.15);
    if use_dense {
        dense_sap(
            graph,
            dense.is_none() && metric == Metric::W1 && graph.rows.max(graph.columns) >= 512,
            stats,
        )
    } else {
        let flows = sparse::solve(graph, None, options.sparse, stats)?;
        let mut matching = buffer(graph.rows, None)?;
        for (row, column, amount) in flows {
            if amount != 1 {
                return Err(Error::InternalInvariant {
                    reason: "unit Wasserstein matching has non-unit flow",
                });
            }
            matching[row] = Some(column);
        }
        Ok(matching)
    }
}

struct Component {
    rows: Vec<usize>,
    columns: Vec<usize>,
}

fn components(graph: &Graph) -> Result<Vec<Component>> {
    let mut reverse = buffer(graph.columns, Vec::new())?;
    for row in 0..graph.rows {
        for edge in graph.edges(row) {
            push(&mut reverse[edge.column], row)?;
        }
    }
    let mut row_seen = buffer(graph.rows, false)?;
    let mut col_seen = buffer(graph.columns, false)?;
    let mut result = Vec::new();
    let mut queue = Vec::new();
    queue
        .try_reserve_exact(sum_size(graph.rows, graph.columns)?)
        .map_err(|_| allocation())?;
    for start in 0..graph.rows {
        if row_seen[start] || graph.edges(start).next().is_none() {
            continue;
        }
        let mut component = Component {
            rows: Vec::new(),
            columns: Vec::new(),
        };
        queue.clear();
        queue.push((true, start));
        row_seen[start] = true;
        let mut next = 0;
        while next < queue.len() {
            let (is_row, index) = queue[next];
            next += 1;
            if is_row {
                push(&mut component.rows, index)?;
                for edge in graph.edges(index) {
                    if !col_seen[edge.column] {
                        col_seen[edge.column] = true;
                        queue.push((false, edge.column));
                    }
                }
            } else {
                push(&mut component.columns, index)?;
                for &row in &reverse[index] {
                    if !row_seen[row] {
                        row_seen[row] = true;
                        queue.push((true, row));
                    }
                }
            }
        }
        push(&mut result, component)?;
    }
    Ok(result)
}

fn tiny(graph: &Graph) -> Result<Vec<Option<usize>>> {
    fn visit(
        graph: &Graph,
        row: usize,
        value: f64,
        current: &mut [Option<usize>],
        used: &mut [bool],
        best: &mut (f64, Vec<Option<usize>>),
    ) {
        if row == graph.rows {
            if value > best.0 {
                best.0 = value;
                best.1.copy_from_slice(current);
            }
            return;
        }
        visit(graph, row + 1, value, current, used, best);
        for edge in graph.edges(row) {
            if used[edge.column] {
                continue;
            }
            used[edge.column] = true;
            current[row] = Some(edge.column);
            visit(graph, row + 1, value + edge.saving, current, used, best);
            used[edge.column] = false;
            current[row] = None;
        }
    }
    let mut best = (0.0, buffer(graph.rows, None)?);
    visit(
        graph,
        0,
        0.0,
        &mut buffer(graph.rows, None)?,
        &mut buffer(graph.columns, false)?,
        &mut best,
    );
    Ok(best.1)
}

fn solve_components(
    graph: &Graph,
    metric: Metric,
    options: Options,
    stats: &mut Stats,
) -> Result<Vec<Option<usize>>> {
    let mut matching = buffer(graph.rows, None)?;
    let mut column_map = buffer(graph.columns, 0)?;
    for component in components(graph)? {
        stats.components += 1;
        if component.rows.len() == 1 || component.columns.len() == 1 {
            let mut best = 0.0;
            let mut pair = None;
            for &row in &component.rows {
                for edge in graph.edges(row) {
                    if edge.saving > best {
                        best = edge.saving;
                        pair = Some((row, edge.column));
                    }
                }
            }
            if let Some((row, column)) = pair {
                matching[row] = Some(column);
            }
            stats.tiny_components += 1;
            continue;
        }
        for (local, &column) in component.columns.iter().enumerate() {
            column_map[column] = local;
        }
        let mut candidates = buffer(component.rows.len(), Vec::new())?;
        for (local, &row) in component.rows.iter().enumerate() {
            for edge in graph.edges(row) {
                push(
                    &mut candidates[local],
                    Edge {
                        column: column_map[edge.column],
                        saving: edge.saving,
                    },
                )?;
            }
        }
        let local = Graph::from_candidates(candidates, component.columns.len(), true)?;
        let local_matching = if sum_size(local.rows, local.columns)? <= 8 {
            stats.tiny_components += 1;
            tiny(&local)?
        } else {
            let dense = local.rows.max(local.columns) <= 24 || local.density() >= 0.20;
            solve_graph(&local, Some(dense), !dense, metric, options, stats)?
        };
        for (row, column) in local_matching.into_iter().enumerate() {
            if let Some(column) = column {
                matching[component.rows[row]] = Some(component.columns[column]);
            }
        }
    }
    Ok(matching)
}

fn groups(points: &[Point]) -> Result<(Vec<Point>, Vec<usize>)> {
    let mut order = Vec::new();
    order
        .try_reserve_exact(points.len())
        .map_err(|_| allocation())?;
    order.extend(0..points.len());
    order.sort_unstable_by(|&a, &b| {
        points[a].coordinates[0]
            .total_cmp(&points[b].coordinates[0])
            .then(points[a].coordinates[1].total_cmp(&points[b].coordinates[1]))
    });
    let mut unique: Vec<Point> = Vec::new();
    let mut multiplicity: Vec<usize> = Vec::new();
    for index in order {
        if unique
            .last()
            .is_some_and(|p| p.coordinates == points[index].coordinates)
        {
            let last = multiplicity.len() - 1;
            multiplicity[last] += 1;
        } else {
            push(&mut unique, points[index])?;
            push(&mut multiplicity, 1)?;
        }
    }
    Ok((unique, multiplicity))
}

fn accumulate(sum: &mut f64, correction: &mut f64, cost: f64) -> Result<()> {
    // Kahan accumulation of original nonnegative costs avoids baseline-minus-
    // savings cancellation, especially for almost equal diagrams in W2.
    let adjusted = cost - *correction;
    let next = finite(*sum + adjusted)?;
    *correction = (next - *sum) - adjusted;
    *sum = next;
    Ok(())
}

fn from_flows(
    first: &[Point],
    second: &[Point],
    row_capacity: &[usize],
    column_capacity: &[usize],
    flows: &[(usize, usize, usize)],
    metric: Metric,
) -> Result<f64> {
    let mut row_used = buffer(first.len(), 0)?;
    let mut col_used = buffer(second.len(), 0)?;
    let mut total = 0.0;
    let mut correction = 0.0;
    for &(row, column, count) in flows {
        row_used[row] = sum_size(row_used[row], count)?;
        col_used[column] = sum_size(col_used[column], count)?;
        accumulate(
            &mut total,
            &mut correction,
            finite(cross_power(first[row], second[column], metric)? * count as f64)?,
        )?;
    }
    for (row, &point) in first.iter().enumerate() {
        let remaining =
            row_capacity[row]
                .checked_sub(row_used[row])
                .ok_or(Error::InternalInvariant {
                    reason: "Wasserstein row capacity exceeded",
                })?;
        if remaining > 0 {
            accumulate(
                &mut total,
                &mut correction,
                finite(diagonal_power(point, metric)? * remaining as f64)?,
            )?;
        }
    }
    for (column, &point) in second.iter().enumerate() {
        let remaining = column_capacity[column]
            .checked_sub(col_used[column])
            .ok_or(Error::InternalInvariant {
                reason: "Wasserstein column capacity exceeded",
            })?;
        if remaining > 0 {
            accumulate(
                &mut total,
                &mut correction,
                finite(diagonal_power(point, metric)? * remaining as f64)?,
            )?;
        }
    }
    Ok(if metric == Metric::W2 {
        total.sqrt()
    } else {
        total
    })
}

pub(crate) fn distance(first: &[[f64; 2]], second: &[[f64; 2]], metric: Metric) -> Result<f64> {
    distance_with_options(
        first,
        second,
        metric,
        Options::default(),
        &mut Stats::default(),
    )
}

pub(crate) fn distance_with_options(
    first: &[[f64; 2]],
    second: &[[f64; 2]],
    metric: Metric,
    options: Options,
    stats: &mut Stats,
) -> Result<f64> {
    let scale = power_scale(first, second);
    let first = prepare(first, scale)?;
    let second = prepare(second, scale)?;
    let original_pairs = product(first.len(), second.len())?;
    if !options.force_sparse {
        let (first_groups, rows) = groups(&first)?;
        let (second_groups, columns) = groups(&second)?;
        stats.duplicate_groups = sum_size(first_groups.len(), second_groups.len())?;
        let removed = first_groups.len() != first.len() || second_groups.len() != second.len();
        if removed && product(first_groups.len(), second_groups.len())? <= original_pairs / 16 {
            let mut candidates = buffer(first_groups.len(), Vec::new())?;
            let mut direct_cost_required = false;
            for (row, &point) in first_groups.iter().enumerate() {
                for (column, &other) in second_groups.iter().enumerate() {
                    stats.candidate_pairs += 1;
                    let (value, direct) = saving(point, other, metric)?;
                    direct_cost_required |= direct;
                    if value > 0.0 {
                        push(
                            &mut candidates[row],
                            Edge {
                                column,
                                saving: value,
                            },
                        )?;
                        stats.positive_edges += 1;
                    }
                }
            }
            if direct_cost_required {
                let matching = direct::matching(&first, &second, metric, stats)?;
                return restore_scale(from_matching(&first, &second, matching, metric)?, scale);
            }
            let graph = Graph::from_candidates(candidates, second_groups.len(), true)?;
            let flows = sparse::solve(&graph, Some((&rows, &columns)), options.sparse, stats)?;
            return restore_scale(
                from_flows(
                    &first_groups,
                    &second_groups,
                    &rows,
                    &columns,
                    &flows,
                    metric,
                )?,
                scale,
            );
        }
    }
    let graph = generate(&first, &second, metric, stats)?;
    let matching = if graph.direct_cost_required {
        direct::matching(&first, &second, metric, stats)?
    } else if options.force_sparse {
        solve_graph(&graph, Some(false), false, metric, options, stats)?
    } else if graph.density() >= 0.15 {
        solve_graph(&graph, None, true, metric, options, stats)?
    } else {
        solve_components(&graph, metric, options, stats)?
    };
    restore_scale(from_matching(&first, &second, matching, metric)?, scale)
}

fn from_matching(
    first: &[Point],
    second: &[Point],
    matching: Vec<Option<usize>>,
    metric: Metric,
) -> Result<f64> {
    let mut flows = Vec::new();
    flows
        .try_reserve_exact(first.len().min(second.len()))
        .map_err(|_| allocation())?;
    for (row, column) in matching.into_iter().enumerate() {
        if let Some(column) = column {
            flows.push((row, column, 1));
        }
    }
    from_flows(
        first,
        second,
        &buffer(first.len(), 1)?,
        &buffer(second.len(), 1)?,
        &flows,
        metric,
    )
}
