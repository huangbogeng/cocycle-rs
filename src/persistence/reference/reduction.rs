use super::FilteredBoundary;
use super::column::SparseColumn;
use crate::{Error, Result};

pub(in crate::persistence) struct Reduction {
    pub(in crate::persistence) pairs: Vec<(usize, usize)>,
    pub(in crate::persistence) unpaired: Vec<usize>,
}

/// Standard left-to-right reduction; see mathematics.md section 4 and B21, Prop. 3.1.
pub(in crate::persistence) fn reduce(input: &impl FilteredBoundary) -> Result<Reduction> {
    let n = input.len();
    let mut columns: Vec<SparseColumn> = Vec::new();
    let mut owners: Vec<Option<usize>> = Vec::new();
    let mut pairs = Vec::new();
    let mut unpaired = Vec::new();
    columns
        .try_reserve(n)
        .map_err(|_| Error::AllocationFailed {
            context: "reduced columns",
        })?;
    owners.try_reserve(n).map_err(|_| Error::AllocationFailed {
        context: "pivot owners",
    })?;
    owners.resize(n, None);
    let mut boundary = Vec::new();
    let mut scratch = Vec::new();
    for j in 0..n {
        input.write_boundary(j, &mut boundary)?;
        let mut column = SparseColumn::new(std::mem::take(&mut boundary));
        while let Some(i) = column.low() {
            let Some(owner) = owners[i] else {
                break;
            };
            column.add_assign(&columns[owner], &mut scratch)?;
        }
        if let Some(i) = column.low() {
            owners[i] = Some(j);
            pairs.try_reserve(1).map_err(|_| Error::AllocationFailed {
                context: "persistence pairs",
            })?;
            pairs.push((i, j));
            columns.push(column);
        } else {
            // Preserve the zero-column marker, but reuse its allocation. In a
            // dense 2-skeleton most triangles reduce to zero; retaining their
            // unused buffers would dominate storage without helping reduction.
            boundary = column.into_entries();
            columns.push(SparseColumn::default());
        }
    }
    // An initially empty column can later be paired: decide only after all columns.
    for (i, column) in columns.iter().enumerate() {
        if column.low().is_none() && owners[i].is_none() {
            unpaired
                .try_reserve(1)
                .map_err(|_| Error::AllocationFailed {
                    context: "unpaired births",
                })?;
            unpaired.push(i);
        }
    }
    Ok(Reduction { pairs, unpaired })
}

#[cfg(test)]
mod tests {
    use super::*;

    // No simplex or coordinate access: a hand-built filtered chain complex (V09).
    struct Triangle;
    impl FilteredBoundary for Triangle {
        fn len(&self) -> usize {
            7
        }
        fn dimension(&self, i: usize) -> usize {
            [0, 0, 0, 1, 1, 1, 2][i]
        }
        fn value(&self, i: usize) -> f64 {
            [0., 0., 0., 1., 1., 1., 2.][i]
        }
        fn write_boundary(&self, i: usize, out: &mut Vec<usize>) -> Result<()> {
            let columns: [&[usize]; 7] = [&[], &[], &[], &[0, 1], &[0, 2], &[1, 2], &[3, 4, 5]];
            out.clear();
            out.extend_from_slice(columns[i]);
            Ok(())
        }
    }

    #[test]
    fn hand_filtration_pairs_a_loop_with_a_later_face() {
        let input = Triangle;
        let result = reduce(&input).unwrap();
        assert_eq!(result.unpaired, vec![0]);
        assert_eq!(result.pairs, vec![(1, 3), (2, 4), (5, 6)]);
        assert_eq!((input.value(5), input.value(6)), (1.0, 2.0));
        for (i, j) in result.pairs {
            assert!(i < j);
            assert_eq!(input.dimension(i) + 1, input.dimension(j));
        }
    }
}
