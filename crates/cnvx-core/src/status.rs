/// Represents the state of a solution after attempting to solve an optimization
/// problem.
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::Status;
/// let status = Status::Optimal;
/// assert_eq!(status.to_string(), "Optimal");
/// ```
#[derive(Debug, strum::Display)]
pub enum Status {
    /// The solver has found an optimal solution to the problem.
    Optimal,
    /// The solver has determined that the problem is infeasible (i.e., there is
    /// no solution that satisfies all constraints).
    Infeasible,
    /// The solver has determined that the problem is either infeasible or
    /// unbounded.
    InfeasibleOrUnbounded,
    /// The solver has determined that the problem is unbounded (i.e., the
    /// objective can be improved indefinitely).
    Unbounded,
    /// The solver has reached the maximum number of iterations allowed without
    /// finding an optimal solution.
    IterationLimit,
    /// The solver has reached the maximum time limit allowed without finding an
    /// optimal solution.
    TimeLimit,
    /// The solver has reached the maximum number of nodes explored without
    /// finding an optimal solution.
    NodeLimit,
    /// The solver has reached the maximum number of solutions found without
    /// finding an optimal solution.
    SolutionLimit,
    /// The solver has been interrupted (e.g., by the user) before finding an
    /// optimal solution.
    Interrupted,
    /// The solver has found a solution that is not optimal, but is acceptable
    /// for the problem at hand. Also known as a Feasible solution.
    Suboptimal,
    /// The solver has encountered a numerical issue that prevents it from
    /// finding an optimal solution.
    Numerical,
    /// The solver is still processing the problem and has not yet reached a
    /// conclusion.
    InProgress,
}
