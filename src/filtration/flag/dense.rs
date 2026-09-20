//! Dense distance access, without materializing triangles or copying inputs.
use super::index::SimplexIndex;
use super::{FlagAccess, SimplexEntry};
use crate::geometry::DissimilarityMatrixView;
use crate::{Error, Result};

pub(crate) struct DenseFlag<'a> {
    input: DissimilarityMatrixView<'a>,
    index: SimplexIndex,
    cutoff: f64,
}
impl<'a> DenseFlag<'a> {
    pub(crate) fn new(input: DissimilarityMatrixView<'a>, cutoff: f64) -> Result<Self> {
        Ok(Self {
            index: SimplexIndex::new(input.len())?,
            input,
            cutoff,
        })
    }
    fn edge(&self, a: usize, b: usize) -> SimplexEntry {
        SimplexEntry {
            id: self.index.edge(a, b),
            value: self.input.get(a, b).unwrap(),
        }
    }
}
impl FlagAccess for DenseFlag<'_> {
    fn vertex_count(&self) -> usize {
        self.input.len()
    }
    fn edges(&self, checkpoint: &mut impl FnMut() -> Result<()>) -> Result<Vec<SimplexEntry>> {
        let mut edges = Vec::new();
        for b in 0..self.input.len() {
            for a in 0..b {
                checkpoint()?;
                let edge = self.edge(a, b);
                if edge.value <= self.cutoff {
                    edges.try_reserve(1).map_err(|_| Error::AllocationFailed {
                        context: "flag edges",
                    })?;
                    edges.push(edge);
                }
            }
        }
        edges.sort_unstable();
        Ok(edges)
    }
    fn edge_vertices(&self, id: usize) -> [usize; 2] {
        self.index.edge_vertices(id)
    }
    fn latest_facet(&self, triangle: SimplexEntry) -> SimplexEntry {
        let [a, b, c] = self.index.triangle_vertices(triangle.id);
        self.edge(a, b).max(self.edge(a, c)).max(self.edge(b, c))
    }
    fn visit_cofacets(
        &self,
        edge: SimplexEntry,
        checkpoint: &mut impl FnMut() -> Result<()>,
        mut visitor: impl FnMut(SimplexEntry) -> Result<bool>,
    ) -> Result<()> {
        let [a, b] = self.edge_vertices(edge.id);
        for v in (0..self.input.len()).rev() {
            checkpoint()?;
            if v == a || v == b {
                continue;
            }
            let value = edge
                .value
                .max(self.input.get(a, v).unwrap())
                .max(self.input.get(b, v).unwrap());
            if value <= self.cutoff
                && !visitor(SimplexEntry {
                    id: self.index.triangle(a, b, v),
                    value,
                })?
            {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::index::choose;
    use super::*;
    use crate::geometry::DissimilarityView;

    #[test]
    fn combinatorial_ids_and_cofacets_match_independent_vertex_enumeration() {
        for n in 0_usize..=12 {
            let values: Vec<_> = (0..n * n.saturating_sub(1) / 2)
                .map(|i| (i % 7) as f64)
                .collect();
            let input = DissimilarityView::new(&values, n).unwrap();
            for cutoff in [0.0, 2.0, 6.0] {
                let rips = DenseFlag::new(input.into(), cutoff).unwrap();
                let mut triangles = Vec::new();
                for c in 2..n {
                    for b in 1..c {
                        for a in 0..b {
                            triangles.push([a, b, c]);
                        }
                    }
                }
                for (id, &vertices) in triangles.iter().enumerate() {
                    assert_eq!(rips.index.triangle_vertices(id), vertices);
                }
                for b in 1..n {
                    for a in 0..b {
                        let edge = rips.edge(a, b);
                        assert_eq!(rips.index.edge_vertices(edge.id), [a, b]);
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
                        assert_eq!(
                            {
                                let mut rows = Vec::new();
                                rips.visit_cofacets(edge, &mut || Ok(()), |row| {
                                    rows.push(row);
                                    Ok(true)
                                })
                                .unwrap();
                                rows
                            },
                            expected
                        );
                    }
                }
                assert!(
                    rips.edges(&mut || Ok(()))
                        .unwrap()
                        .windows(2)
                        .all(|w| w[0] < w[1])
                );
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
