use crate::{Error, Result};

/// Sorted, distinct nonzero row indices of an F2 column.
#[derive(Default)]
pub(super) struct SparseColumn {
    entries: Vec<usize>,
}

impl SparseColumn {
    pub(super) fn new(entries: Vec<usize>) -> Self {
        debug_assert!(entries.windows(2).all(|pair| pair[0] < pair[1]));
        Self { entries }
    }

    pub(super) fn low(&self) -> Option<usize> {
        self.entries.last().copied()
    }

    pub(super) fn into_entries(self) -> Vec<usize> {
        self.entries
    }

    pub(super) fn add_assign(&mut self, other: &Self, scratch: &mut Vec<usize>) -> Result<()> {
        let capacity =
            self.entries
                .len()
                .checked_add(other.entries.len())
                .ok_or(Error::SizeOverflow {
                    operation: "column addition capacity",
                })?;
        scratch.clear();
        scratch
            .try_reserve(capacity)
            .map_err(|_| Error::AllocationFailed {
                context: "column sum",
            })?;
        let (mut a, mut b) = (0, 0);
        while a < self.entries.len() && b < other.entries.len() {
            match self.entries[a].cmp(&other.entries[b]) {
                std::cmp::Ordering::Less => {
                    scratch.push(self.entries[a]);
                    a += 1;
                }
                std::cmp::Ordering::Greater => {
                    scratch.push(other.entries[b]);
                    b += 1;
                }
                std::cmp::Ordering::Equal => {
                    a += 1;
                    b += 1;
                }
            }
        }
        scratch.extend_from_slice(&self.entries[a..]);
        scratch.extend_from_slice(&other.entries[b..]);
        // Keep the old allocation for the next addition, including across columns.
        std::mem::swap(&mut self.entries, scratch);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_matches_independent_bit_vector_xor() {
        let mut scratch = Vec::new();
        for a in 0_u32..32 {
            for b in 0_u32..32 {
                let indices = |bits: u32| (0..5).filter(|i| bits & (1 << i) != 0).collect();
                let mut left = SparseColumn::new(indices(a));
                left.add_assign(&SparseColumn::new(indices(b)), &mut scratch)
                    .unwrap();
                let expected: Vec<_> = indices(a ^ b);
                assert_eq!(left.entries, expected);
                assert_eq!(left.low(), expected.last().copied());
            }
        }
    }
}
