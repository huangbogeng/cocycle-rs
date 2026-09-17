use crate::Result;

/// Internal contract for a finite F2 filtered chain complex.
///
/// Values are finite and nondecreasing. A boundary contains sorted, distinct
/// indices smaller than the current index, each of dimension one lower. The
/// boundary operator squares to zero. Implementors establish these properties;
/// reduction does not revalidate the whole complex on each column access.
pub(crate) trait FilteredBoundary {
    fn len(&self) -> usize;
    fn dimension(&self, index: usize) -> usize;
    fn value(&self, index: usize) -> f64;
    /// Replace the buffer's contents, retaining its capacity when possible.
    fn write_boundary(&self, index: usize, output: &mut Vec<usize>) -> Result<()>;
}
