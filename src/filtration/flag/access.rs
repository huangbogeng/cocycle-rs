//! Private contract implemented by dense matrices and sparse graphs.
use super::SimplexEntry;
use crate::Result;

/// Exact flag access: edges in forward order, cofacets in decreasing ID order,
/// latest facet in the same total order. All values are finite/canonical and all
/// triangle faces exist. Callers never assume a cone for arbitrary sparse input.
pub(crate) trait FlagAccess {
    fn vertex_count(&self) -> usize;
    fn edges(&self, checkpoint: &mut impl FnMut() -> Result<()>) -> Result<Vec<SimplexEntry>>;
    fn edge_vertices(&self, id: usize) -> [usize; 2];
    fn latest_facet(&self, triangle: SimplexEntry) -> SimplexEntry;
    /// Check every candidate (including rejected ones). Visitor false stops early.
    fn visit_cofacets(
        &self,
        edge: SimplexEntry,
        checkpoint: &mut impl FnMut() -> Result<()>,
        visitor: impl FnMut(SimplexEntry) -> Result<bool>,
    ) -> Result<()>;
}
