//! Per-call cooperative computation control.
use crate::{Error, Result};
use std::sync::atomic::{AtomicBool, Ordering};

/// Optional controls for one persistence computation. Defaults are unlimited.
///
/// One work unit is a cone-bound distance visit, candidate edge scan, forward
/// edge processing, reverse column/reduction step, cofacet candidate test (one
/// neighbor-intersection comparison for sparse input), or heap removal including
/// parity cancellations. Dimension-generic computation also counts dimension
/// traversal checkpoints, candidate vertices, adjacency tests and transformation
/// additions. Representative computation additionally counts simplex/face visits,
/// boundary reduction and dual-basis steps, coefficient updates and output terms.
/// Repeated work counts again. The next unit fails before
/// exceeding the limit. Counts are algorithm-specific, not comparable runtimes.
///
/// Cancellation is checked at work units and phase boundaries. Allocation,
/// index-table/vertex initialization, sorting, heap construction from a collected
/// column and result normalization are not interruptible internally; cancellation
/// is checked around those phases.
/// Opt-in representative skeleton materialization is part of computation and
/// is controlled. Standalone graph/complex construction is not. These controls
/// do not constrain memory bytes, wall time or RSS.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExecutionLimits<'a> {
    max_work: Option<u64>,
    cancellation: Option<&'a AtomicBool>,
}
impl<'a> ExecutionLimits<'a> {
    /// Create cooperative controls. A zero limit permits no counted work units.
    /// Set the optional flag to true to request cancellation; the flag is never reset.
    pub fn new(max_work: Option<u64>, cancellation: Option<&'a AtomicBool>) -> Self {
        Self {
            max_work,
            cancellation,
        }
    }
}

pub(super) struct WorkBudget<'a> {
    limits: ExecutionLimits<'a>,
    used: u64,
}
impl<'a> WorkBudget<'a> {
    pub(super) fn new(limits: &ExecutionLimits<'a>) -> Result<Self> {
        let result = Self {
            limits: *limits,
            used: 0,
        };
        result.check()?;
        Ok(result)
    }
    pub(super) fn check(&self) -> Result<()> {
        if self
            .limits
            .cancellation
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            return Err(Error::Cancelled);
        }
        Ok(())
    }
    pub(super) fn step(&mut self) -> Result<()> {
        self.check()?;
        if let Some(limit) = self.limits.max_work {
            if self.used >= limit {
                return Err(Error::WorkLimitExceeded { limit });
            }
            self.used += 1;
        }
        Ok(())
    }
}
