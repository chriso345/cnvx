use crate::Status;

#[derive(Debug, thiserror::Error)]
pub enum CnvxError {
    #[error("handle from a different Model instance")]
    ForeignHandle,
    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("invalid bounds: lower {lower} > upper {upper}")]
    InvalidBounds { lower: f64, upper: f64 },
    #[error("model could not be converted to dual form: {0}")]
    DualisationFailed(String),
    #[error("solver could not reach optimality: {0:?}")]
    SolveFailed(Status),
    #[error("numerical error in solver internals: {0}")]
    Numerical(String),
    #[error(transparent)]
    Math(#[from] cnvx_math::MathError),
}
