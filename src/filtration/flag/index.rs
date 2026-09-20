//! Checked O(n) combinatorial index prefixes for edges and triangles.
use crate::{Error, Result};

pub(super) struct SimplexIndex {
    offsets: Vec<[usize; 2]>,
}
impl SimplexIndex {
    pub(super) fn new(n: usize) -> Result<Self> {
        choose(n, 3)?;
        let len = n.checked_add(1).ok_or(Error::SizeOverflow {
            operation: "simplex index table",
        })?;
        let mut offsets = Vec::new();
        offsets
            .try_reserve_exact(len)
            .map_err(|_| Error::AllocationFailed {
                context: "simplex index table",
            })?;
        for v in 0..=n {
            offsets.push([choose(v, 2)?, choose(v, 3)?]);
        }
        Ok(Self { offsets })
    }
    pub(super) fn edge(&self, a: usize, b: usize) -> usize {
        let (a, b) = if a < b { (a, b) } else { (b, a) };
        self.offsets[b][0] + a
    }
    pub(super) fn triangle(&self, a: usize, b: usize, v: usize) -> usize {
        if v > b {
            self.offsets[v][1] + self.edge(a, b)
        } else if v > a {
            self.offsets[b][1] + self.edge(a, v)
        } else {
            self.offsets[b][1] + self.edge(v, a)
        }
    }
    pub(super) fn edge_vertices(&self, id: usize) -> [usize; 2] {
        let b = self.offsets.partition_point(|x| x[0] <= id) - 1;
        [id - self.offsets[b][0], b]
    }
    pub(super) fn triangle_vertices(&self, id: usize) -> [usize; 3] {
        let c = self.offsets.partition_point(|x| x[1] <= id) - 1;
        let [a, b] = self.edge_vertices(id - self.offsets[c][1]);
        [a, b, c]
    }
}

/// Divide factors first, so a representable binomial is not rejected merely
/// because an intermediate product would overflow. Only k=2,3 are needed.
pub(super) fn choose(n: usize, k: usize) -> Result<usize> {
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
