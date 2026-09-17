use std::fmt;

/// A recoverable input, configuration or result error.
///
/// Match variants to handle errors; the human-readable messages are not a stable
/// serialization format. Additional variants may be added as algorithms arrive.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// A Betti-curve grid is not finite, nonnegative and strictly increasing.
    InvalidGrid {
        /// Position of the first invalid grid value.
        index: usize,
        /// Constraint that was violated.
        reason: &'static str,
    },
    /// A query extends beyond a diagram's known range.
    QueryOutsideCoverage {
        /// Position of the query in its grid.
        index: usize,
        /// Requested scale.
        value: f64,
        /// Last known scale.
        through: f64,
    },
    /// A fallible memory reservation failed.
    AllocationFailed {
        /// Buffer being grown.
        context: &'static str,
    },
    /// An internal construction violated an algorithm invariant.
    InternalInvariant {
        /// Invariant that was violated.
        reason: &'static str,
    },
    /// A numerical computation produced a non-finite result.
    NumericalFailure {
        /// Computation that failed.
        context: &'static str,
    },
    /// A buffer does not match its declared shape.
    ShapeMismatch {
        /// Name of the input buffer.
        input: &'static str,
        /// Required number of elements.
        expected: usize,
        /// Supplied number of elements.
        actual: usize,
    },
    /// A shape cannot be represented by `usize`.
    SizeOverflow {
        /// Operation whose result overflowed.
        operation: &'static str,
    },
    /// A scalar or buffer element is NaN or infinite.
    NonFiniteValue {
        /// Name of the value or input buffer.
        field: &'static str,
        /// Buffer position, or `None` for a scalar.
        index: Option<usize>,
    },
    /// A value required to be nonnegative is negative.
    NegativeValue {
        /// Name of the value or input buffer.
        field: &'static str,
        /// Buffer position, or `None` for a scalar.
        index: Option<usize>,
    },
    /// A configuration value violates a parameter constraint.
    InvalidParameter {
        /// Name of the parameter.
        parameter: &'static str,
        /// Constraint that was violated.
        reason: &'static str,
    },
    /// A computation was requested in an unsupported dimension.
    UnsupportedDimension {
        /// Requested maximum homology dimension.
        requested: usize,
        /// Largest dimension supported by this computation.
        max_supported: usize,
    },
    /// Interval endpoints are inconsistent.
    InvalidInterval {
        /// Constraint that was violated.
        reason: &'static str,
    },
    /// An interval conflicts with the diagram's computation range.
    InconsistentDiagram {
        /// Position in the supplied, unsorted interval vector.
        interval: usize,
        /// Constraint that was violated.
        reason: &'static str,
    },
    /// A dimension lies outside those recorded as computed.
    DimensionNotComputed {
        /// Dimension requested or supplied by the caller.
        requested: usize,
        /// Largest dimension recorded as computed, including all lower ones.
        computed_max: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGrid { index, reason } => write!(f, "grid[{index}]: {reason}"),
            Self::QueryOutsideCoverage {
                index,
                value,
                through,
            } => {
                write!(
                    f,
                    "grid[{index}] = {value} exceeds coverage through {through}"
                )
            }
            Self::AllocationFailed { context } => write!(f, "allocation failed for {context}"),
            Self::InternalInvariant { reason } => write!(f, "internal invariant failed: {reason}"),
            Self::NumericalFailure { context } => write!(f, "non-finite result in {context}"),
            Self::ShapeMismatch {
                input,
                expected,
                actual,
            } => {
                write!(f, "{input}: expected {expected} elements, got {actual}")
            }
            Self::SizeOverflow { operation } => write!(f, "size overflow in {operation}"),
            Self::NonFiniteValue { field, index } => {
                write_location(f, field, *index)?;
                f.write_str(" must be finite")
            }
            Self::NegativeValue { field, index } => {
                write_location(f, field, *index)?;
                f.write_str(" must be nonnegative")
            }
            Self::InvalidParameter { parameter, reason } => write!(f, "{parameter}: {reason}"),
            Self::UnsupportedDimension {
                requested,
                max_supported,
            } => {
                write!(
                    f,
                    "homology dimension {requested} is unsupported; maximum is {max_supported}"
                )
            }
            Self::InvalidInterval { reason } => write!(f, "invalid interval: {reason}"),
            Self::InconsistentDiagram { interval, reason } => {
                write!(
                    f,
                    "interval {interval} conflicts with the diagram: {reason}"
                )
            }
            Self::DimensionNotComputed {
                requested,
                computed_max,
            } => {
                write!(
                    f,
                    "dimension {requested} was not computed; maximum is {computed_max}"
                )
            }
        }
    }
}

fn write_location(f: &mut fmt::Formatter<'_>, field: &str, index: Option<usize>) -> fmt::Result {
    f.write_str(field)?;
    if let Some(index) = index {
        write!(f, "[{index}]")?;
    }
    Ok(())
}

impl std::error::Error for Error {}

/// The result type returned by fallible Cocycle operations.
pub type Result<T> = std::result::Result<T, Error>;
