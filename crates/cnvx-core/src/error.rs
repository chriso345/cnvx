use std::fmt::Display;

/// Represents the various errors that can occur during modeling or solving
/// an optimization problem.
///
/// This type is used by solvers and the modeling API to communicate problems
/// such as missing objectives, invalid models, or numerical issues.
#[derive(Debug)]
pub enum SolveError {
    /// The model has no objective function defined.
    NoObjective,

    /// The model is invalid (e.g., constraints are inconsistent or malformed).
    InvalidModel(String),

    /// A numerical failure occurred during solving (e.g., singular matrix).
    NumericalFailure(String),

    /// Internal solver error (unexpected state or panic inside the solver).
    InternalSolverError(String),

    /// The solver does not support a required feature (e.g., non-linear constraints).
    Unsupported(String),

    Other(String),
}

impl Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolveError::NoObjective => write!(f, "No objective function defined"),
            SolveError::InvalidModel(msg) => write!(f, "Invalid model: {}", msg),
            SolveError::NumericalFailure(msg) => write!(f, "Numerical failure: {}", msg),
            SolveError::InternalSolverError(msg) => {
                write!(f, "Internal solver error: {}", msg)
            }
            SolveError::Unsupported(msg) => write!(f, "Unsupported feature: {}", msg),
            SolveError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SolveError {}
