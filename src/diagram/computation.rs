//! Owned mathematical context without borrowed inputs or reducer state.
use super::PersistenceDiagram;

use crate::filtration::FiltrationKind;

/// Owned source context and ordinary persistence analysis settings.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputationContext {
    pub(crate) filtration: crate::filtration::FiltrationContext,
    pub(crate) field: crate::algebra::PrimeField,
    pub(crate) requested_cutoff: Option<f64>,
}
impl ComputationContext {
    /// Reusable source facts, including scale convention and construction metadata.
    pub fn filtration(&self) -> &crate::filtration::FiltrationContext {
        &self.filtration
    }
    /// Sparse Rips provenance, when applicable.
    pub fn approximation(&self) -> Option<&super::RipsApproximation> {
        self.filtration.approximation()
    }
    /// Compatibility classification of the mathematical source.
    pub fn filtration_kind(&self) -> FiltrationKind {
        self.filtration.filtration_kind()
    }
    /// Source vertices, including vertices outside a smaller analysis cutoff.
    pub fn vertex_count(&self) -> usize {
        self.filtration.vertex_count()
    }
    /// Requested computation cutoff before internal stopping optimizations.
    pub fn requested_cutoff(&self) -> Option<f64> {
        self.requested_cutoff
    }
    /// Requested construction cutoff in the source's declared units.
    pub fn construction_cutoff(&self) -> Option<f64> {
        self.filtration.construction_cutoff()
    }
    /// Coefficient field characteristic.
    pub fn characteristic(&self) -> u32 {
        self.field.characteristic()
    }
    pub(crate) fn new(
        field: crate::algebra::PrimeField,
        kind: FiltrationKind,
        vertex_count: usize,
        requested_cutoff: Option<f64>,
        construction_cutoff: Option<f64>,
        approximation: Option<super::RipsApproximation>,
    ) -> Self {
        Self {
            filtration: crate::filtration::FiltrationContext::new(
                kind,
                vertex_count,
                construction_cutoff,
                approximation,
            ),
            field,
            requested_cutoff,
        }
    }
    pub(crate) fn set_kind(&mut self, kind: FiltrationKind) {
        self.filtration = crate::filtration::FiltrationContext::new(
            kind,
            self.vertex_count(),
            self.construction_cutoff(),
            self.approximation().cloned(),
        );
    }
}

/// Owned diagram and mathematical context, independent of source lifetimes.
/// Coverage and computed dimensions are recorded in the diagram, not duplicated.
#[derive(Clone, Debug, PartialEq)]
pub struct PersistenceResult {
    pub(crate) diagram: PersistenceDiagram,
    pub(crate) context: ComputationContext,
    pub(crate) representatives: Option<Vec<super::Representative>>,
}
impl PersistenceResult {
    /// Borrow the diagram for existing descriptor operations.
    pub fn diagram(&self) -> &PersistenceDiagram {
        &self.diagram
    }
    /// Borrow the mathematical context.
    pub fn context(&self) -> &ComputationContext {
        &self.context
    }
    /// Requested representatives, or `None` when no requests were supplied.
    /// An empty slice means requests were made but no intervals were active.
    pub fn representatives(&self) -> Option<&[super::Representative]> {
        self.representatives.as_deref()
    }
    /// Consume the result, explicitly discarding context and representatives.
    pub fn into_diagram(self) -> PersistenceDiagram {
        self.diagram
    }
}
