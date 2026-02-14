use std::fmt::Display;

/// Represents the state of a solution after attempting to solve an optimization problem.
///
/// Used in [`Solution`](crate::solution::Solution) to indicate whether the solver found an optimal solution,
/// whether the problem is infeasible or unbounded, or if it encountered some other state.
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::SolveStatus;
/// let status = SolveStatus::Optimal;
/// assert_eq!(status.to_string(), "Optimal");
/// ```
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SolveStatus {
    /// The solver has not attempted to solve the model yet.
    NotSolved,

    /// The solver found an optimal solution.
    Optimal,

    /// The problem is infeasible: no solution satisfies all constraints.
    Infeasible,

    /// The problem is unbounded: the objective can increase/decrease without limit.
    Unbounded,

    Other(String),
}

impl Display for SolveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolveStatus::NotSolved => write!(f, "Not Solved"),
            SolveStatus::Optimal => write!(f, "Optimal"),
            SolveStatus::Infeasible => write!(f, "Infeasible"),
            SolveStatus::Unbounded => write!(f, "Unbounded"),
            SolveStatus::Other(s) => write!(f, "Other: {}", s),
        }
    }
}

/// Represents the various errors that can occur during modeling or solving
/// an optimization problem.
///
/// This type is used by solvers and the modeling API to communicate problems
/// such as missing objectives, invalid models, or numerical issues.
#[derive(Debug, Eq, PartialEq)]
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
