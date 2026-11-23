use crate::error::CnvxError;

/// A purely organizing trait, implemented once per problem-type/solver-type
/// pair, so generic code (CLI tools, benchmarking harnesses, logging
/// wrappers) can be written once against `Solve<S>` instead of once per
/// concrete solver. This is not a plug-point for alternative solver
/// backends: each `S` still wraps exactly one native algorithm per
/// method, chosen at the `S`'s construction (e.g.
/// `LpSolver::primal_simplex()`), not at the trait level.
///
/// # Examples
///
/// `cnvx-lp` implements `Solve<LpSolver> for Model`:
///
/// ```rust,ignore
/// let solution = model.solve(&LpSolver::primal_simplex())?;
/// ```
pub trait Solve<S> {
    /// The solution type this problem/solver pair produces.
    type Solution;

    /// Solves `self` with `solver`, returning `Ok` for any well-defined
    /// terminal outcome (including e.g. an infeasible or unbounded LP --
    /// see [`crate::Status`]) and `Err` only when the problem or solver
    /// configuration itself was invalid.
    fn solve(&self, solver: &S) -> Result<Self::Solution, CnvxError>;
}
