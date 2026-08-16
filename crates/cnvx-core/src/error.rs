use crate::Status;

/// Errors produced by building or solving a [`Model`](crate::Model), and
/// shared by every other `cnvx-*` crate.
#[derive(Debug, thiserror::Error)]
pub enum CnvxError {
    /// A [`Var`](crate::Var) or [`Con`](crate::Con) handle was passed to a
    /// [`Model`](crate::Model) other than the one that created it.
    #[error("handle from a different Model instance")]
    ForeignHandle,
    /// Two operands had incompatible shapes.
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    /// A lower bound exceeded its upper bound.
    #[error("invalid bounds: lower {lower} > upper {upper}")]
    InvalidBounds { lower: f64, upper: f64 },
    /// A model could not be converted to dual form (e.g. by an LP solver's
    /// `to_dual` extension).
    #[error("model could not be converted to dual form: {0}")]
    DualisationFailed(String),
    /// A solver ran to completion without reaching optimality.
    #[error("solver could not reach optimality: {0:?}")]
    SolveFailed(Status),
    /// A solver hit a numerical issue (e.g. a singular basis) it could not
    /// recover from.
    #[error("numerical error in solver internals: {0}")]
    Numerical(String),
    /// A lower-level [`cnvx_math::MathError`] propagated up unchanged.
    #[error(transparent)]
    Math(#[from] cnvx_math::MathError),
}
