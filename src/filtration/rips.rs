//! Implicit F2 Rips 2-skeleton; see docs/reference/mathematics.md section 9.

use std::cmp::Ordering;

use crate::geometry::DissimilarityView;
use crate::{Error, Result};

/// Within one dimension: increasing value, then decreasing combinatorial id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SimplexEntry {
    pub(crate) id: usize,
    pub(crate) value: f64,
}

// Values come only from validated, canonicalized finite distances.
impl Eq for SimplexEntry {}
impl Ord for SimplexEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value
            .total_cmp(&other.value)
            .then(other.id.cmp(&self.id))
    }
}
impl PartialOrd for SimplexEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// O(n) binomial prefixes, rather than an index for every triangle.
pub(crate) struct RipsFiltration<'a> {
    input: DissimilarityView<'a>,
    offsets: Vec<[usize; 2]>,
    cutoff: f64,
}

impl<'a> RipsFiltration<'a> {
    pub(crate) fn new(input: DissimilarityView<'a>, cutoff: f64) -> Result<Self> {
        let n = input.len();
        // Check the largest counts before allocating or constructing any ids.
        choose(n, 3)?;
        let count = n.checked_add(1).ok_or(Error::SizeOverflow {
            operation: "Rips index table length",
        })?;
        let mut offsets = Vec::new();
        offsets
            .try_reserve_exact(count)
            .map_err(|_| Error::AllocationFailed {
                context: "Rips binomial prefixes",
            })?;
        for v in 0..=n {
            offsets.push([choose(v, 2)?, choose(v, 3)?]);
        }
        Ok(Self {
            input,
            offsets,
            cutoff,
        })
    }

    pub(crate) fn edges(&self) -> Result<Vec<SimplexEntry>> {
        let mut edges = Vec::new();
        for b in 1..self.input.len() {
            for a in 0..b {
                let edge = self.edge(a, b);
                if edge.value <= self.cutoff {
                    edges.try_reserve(1).map_err(|_| Error::AllocationFailed {
                        context: "Rips edges",
                    })?;
                    edges.push(edge);
                }
            }
        }
        edges.sort_unstable();
        Ok(edges)
    }

    pub(crate) fn edge_vertices(&self, id: usize) -> [usize; 2] {
        debug_assert!(id < self.offsets[self.input.len()][0]);
        let b = self.offsets.partition_point(|x| x[0] <= id) - 1;
        [id - self.offsets[b][0], b]
    }

    fn triangle_vertices(&self, id: usize) -> [usize; 3] {
        debug_assert!(id < self.offsets[self.input.len()][1]);
        let c = self.offsets.partition_point(|x| x[1] <= id) - 1;
        let [a, b] = self.edge_vertices(id - self.offsets[c][1]);
        [a, b, c]
    }

    fn edge(&self, a: usize, b: usize) -> SimplexEntry {
        SimplexEntry {
            id: self.offsets[b][0] + a,
            value: self.distance(a, b),
        }
    }

    fn distance(&self, a: usize, b: usize) -> f64 {
        let (a, b) = if a < b { (a, b) } else { (b, a) };
        // All callers use distinct vertices below n. The constructor checked
        // the entire condensed shape; max also canonicalizes negative zero.
        debug_assert!(a < b && b < self.input.len());
        self.input.values()[self.offsets[b][0] + a].max(0.0)
    }

    /// Descending triangle ids, NOT filtration order. Among cofacets at the
    /// edge's value, the first is the earliest possible filtration cofacet.
    pub(crate) fn cofacets(&self, edge: SimplexEntry) -> impl Iterator<Item = SimplexEntry> + '_ {
        let [a, b] = self.edge_vertices(edge.id);
        (0..self.input.len()).rev().filter_map(move |v| {
            if v == a || v == b {
                return None;
            }
            let value = edge.value.max(self.distance(a, v)).max(self.distance(b, v));
            if value > self.cutoff {
                return None;
            }
            let id = if v > b {
                self.offsets[v][1] + edge.id
            } else if v > a {
                self.offsets[b][1] + self.offsets[v][0] + a
            } else {
                self.offsets[b][1] + self.offsets[a][0] + v
            };
            Some(SimplexEntry { id, value })
        })
    }

    pub(crate) fn latest_facet(&self, triangle: SimplexEntry) -> SimplexEntry {
        let [a, b, c] = self.triangle_vertices(triangle.id);
        self.edge(a, b).max(self.edge(a, c)).max(self.edge(b, c))
    }
}

/// Divide factors first, so a representable binomial is not rejected merely
/// because an intermediate product would overflow. Only k=2,3 are needed.
fn choose(n: usize, k: usize) -> Result<usize> {
    if n < k {
        return Ok(0);
    }
    let mut factors = [n, n - 1, if k == 3 { n - 2 } else { 1 }];
    for divisor in 2..=k {
        let factor = factors.iter_mut().find(|x| **x % divisor == 0).unwrap();
        *factor /= divisor;
    }
    factors
        .into_iter()
        .try_fold(1_usize, |a, b| a.checked_mul(b))
        .ok_or(Error::SizeOverflow {
            operation: "Rips simplex count",
        })
}

/// At this closed scale the flag complex is a cone, even for nonmetric input.
pub(crate) fn cone_radius(input: DissimilarityView<'_>) -> f64 {
    (0..input.len())
        .map(|v| {
            (0..input.len())
                .map(|u| input.get(u, v).unwrap())
                .fold(0.0_f64, f64::max)
        })
        .reduce(f64::min)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinatorial_ids_and_cofacets_match_independent_vertex_enumeration() {
        for n in 0_usize..=12 {
            let values: Vec<_> = (0..n * n.saturating_sub(1) / 2)
                .map(|i| (i % 7) as f64)
                .collect();
            let input = DissimilarityView::new(&values, n).unwrap();
            for cutoff in [0.0, 2.0, 6.0] {
                let rips = RipsFiltration::new(input, cutoff).unwrap();
                let mut triangles = Vec::new();
                for c in 2..n {
                    for b in 1..c {
                        for a in 0..b {
                            triangles.push([a, b, c]);
                        }
                    }
                }
                for (id, &vertices) in triangles.iter().enumerate() {
                    assert_eq!(rips.triangle_vertices(id), vertices);
                }
                for b in 1..n {
                    for a in 0..b {
                        let edge = rips.edge(a, b);
                        assert_eq!(rips.edge_vertices(edge.id), [a, b]);
                        if edge.value > cutoff {
                            continue;
                        }
                        let expected: Vec<_> = triangles
                            .iter()
                            .enumerate()
                            .rev()
                            .filter(|(_, v)| v.contains(&a) && v.contains(&b))
                            .filter_map(|(id, v)| {
                                let value = input
                                    .get(v[0], v[1])
                                    .unwrap()
                                    .max(input.get(v[0], v[2]).unwrap())
                                    .max(input.get(v[1], v[2]).unwrap());
                                (value <= cutoff).then_some(SimplexEntry { id, value })
                            })
                            .collect();
                        assert_eq!(rips.cofacets(edge).collect::<Vec<_>>(), expected);
                    }
                }
                assert!(rips.edges().unwrap().windows(2).all(|w| w[0] < w[1]));
            }
        }
    }

    #[test]
    fn binomial_counts_detect_overflow_without_intermediate_overflow() {
        assert_eq!(choose(0, 3).unwrap(), 0);
        assert_eq!(choose(128, 3).unwrap(), 341376);
        for k in [2, 3] {
            let (mut lo, mut hi) = (k, usize::MAX);
            while hi - lo > 1 {
                let mid = lo + (hi - lo) / 2;
                if choose(mid, k).is_ok() {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let expected = if k == 2 {
                (lo as u128) * ((lo - 1) as u128) / 2
            } else {
                (lo as u128) * ((lo - 1) as u128) * ((lo - 2) as u128) / 6
            };
            assert_eq!(choose(lo, k).unwrap() as u128, expected);
            assert!(choose(hi, k).is_err());
        }
    }
}
