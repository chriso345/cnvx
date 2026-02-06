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
#[derive(Debug, Eq, PartialEq)]
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
