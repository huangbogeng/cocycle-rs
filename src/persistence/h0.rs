use crate::geometry::DissimilarityView;
use crate::{Error, Result};

/// Independent Kruskal/union-find path; no boundary matrix or simplex representation.
pub(super) fn compute(
    input: DissimilarityView<'_>,
    cutoff: f64,
) -> Result<Vec<(usize, f64, Option<f64>)>> {
    let n = input.len();
    let mut edges = Vec::new();
    for i in 0..n {
        for j in 0..i {
            let weight = input.get(i, j).ok_or(Error::InternalInvariant {
                reason: "H0 distance index out of bounds",
            })?;
            if weight <= cutoff {
                edges.try_reserve(1).map_err(|_| Error::AllocationFailed {
                    context: "H0 edges",
                })?;
                edges.push((weight, j, i));
            }
        }
    }
    edges.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut forest = UnionFind::new(n)?;
    let mut raw = Vec::new();
    raw.try_reserve(n).map_err(|_| Error::AllocationFailed {
        context: "H0 intervals",
    })?;
    for (weight, a, b) in edges {
        if !forest.merge(a, b) {
            continue;
        }
        raw.push((0, 0.0, Some(weight)));
        if forest.components == 1 {
            break;
        }
    }
    raw.extend((0..forest.components).map(|_| (0, 0.0, None)));
    Ok(raw)
}

/// The H1 caller continues scanning after connectivity to identify cycle edges.
pub(super) struct UnionFind {
    parents: Vec<usize>,
    sizes: Vec<usize>,
    pub(super) components: usize,
}

impl UnionFind {
    pub(super) fn new(n: usize) -> Result<Self> {
        let mut parents = Vec::new();
        let mut sizes = Vec::new();
        parents
            .try_reserve_exact(n)
            .map_err(|_| Error::AllocationFailed {
                context: "H0 parents",
            })?;
        sizes
            .try_reserve_exact(n)
            .map_err(|_| Error::AllocationFailed {
                context: "H0 sizes",
            })?;
        parents.extend(0..n);
        sizes.resize(n, 1);
        Ok(Self {
            parents,
            sizes,
            components: n,
        })
    }

    pub(super) fn merge(&mut self, a: usize, b: usize) -> bool {
        let (mut a, mut b) = (root(&mut self.parents, a), root(&mut self.parents, b));
        if a == b {
            return false;
        }
        if self.sizes[a] < self.sizes[b] {
            std::mem::swap(&mut a, &mut b);
        }
        self.parents[b] = a;
        self.sizes[a] += self.sizes[b];
        self.components -= 1;
        true
    }
}

fn root(parents: &mut [usize], mut vertex: usize) -> usize {
    while parents[vertex] != vertex {
        parents[vertex] = parents[parents[vertex]];
        vertex = parents[vertex];
    }
    vertex
}
