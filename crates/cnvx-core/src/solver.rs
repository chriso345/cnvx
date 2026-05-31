use crate::{Solution, SolveError, problem::Problem};

/// Trait for optimization solvers.
///
/// Any struct implementing this trait can solve a [`Model`] and produce a [`Solution`].
/// This trait provides a consistent interface across different solver implementations,
/// such as simplex, interior point, branch-and-bound, or lexicographic solvers.
///
/// ```rust
/// use cnvx_core::{Solution, SolveError, problem::Problem, solver::Solver};
///
/// pub struct MySolver { /* internal state */ }
///
/// impl Solver for MySolver {
///     fn supports(&self, problem: &dyn Problem) -> bool {
///         // Accept any LP problem.
///         problem.kind() == "lp" && problem.has_objective()
///     }
///
///     fn solve(
///         &mut self,
///         problem: &dyn Problem,
///     ) -> Result<Solution, SolveError> {
///         let model = problem
///             .as_any()
///             .downcast_ref::<cnvx_core::Model>()
///             .ok_or_else(|| SolveError::InvalidModel("expected LP Model".into()))?;
///         // ... run algorithm on `model` ...
///         todo!()
///     }
///
///     fn objective_value(&self) -> Option<f64> { todo!() }
///     fn solution_vector(&self) -> Vec<f64> { todo!() }
/// }
/// ```
pub trait Solver: Send {
    /// Returns `true` if this solver is able to handle `problem`.
    ///
    /// Implementations should check [`Problem::kind()`](Problem::kind) and any
    /// other preconditions (e.g. whether an objective is defined, whether all
    /// variables are continuous).  A return value of `false` does not mean the
    /// problem is infeasible — only that this solver cannot attempt it.
    ///
    /// # Conventions
    ///
    /// - Return `false` early and cheaply; do not inspect the full problem.
    /// - Document which problem kinds and properties are supported in the
    ///   solver's own doc comment.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn supports(&self, problem: &dyn Problem) -> bool {
    ///     problem.kind() == "lp" && problem.has_objective()
    /// }
    /// ```
    fn supports(&self, problem: &dyn Problem) -> bool;

    /// Attempt to solve `problem` and return a [`Solution`].
    ///
    /// The solver borrows `problem` only for the duration of this call.  After
    /// `solve` returns, the solver retains any internal state needed for a
    /// subsequent warm-started solve (tableau, dual variables, etc.).
    ///
    /// # Errors
    ///
    /// | Error                          | Meaning                                      |
    /// |--------------------------------|----------------------------------------------|
    /// | [`SolveError::Unsupported`]    | [`supports`](Self::supports) would be `false`|
    /// | [`SolveError::NoObjective`]    | Problem has no objective                     |
    /// | [`SolveError::InvalidModel`]   | Problem data is inconsistent                 |
    /// | [`SolveError::NumericalFailure`]| Numerical breakdown during solving           |
    /// | [`SolveError::Other`]          | Iteration limit or other termination         |
    ///
    /// # Panics
    ///
    /// Does not panic under normal circumstances.  Implementations may panic
    /// on internal assertion failures that indicate a programming error.
    fn solve(&mut self, problem: &dyn Problem) -> Result<Solution, SolveError>;

    /// Returns the objective value from the most recent call to [`solve`](Self::solve).
    ///
    /// Returns `None` if `solve` has not been called yet, or if the most
    /// recent solve did not produce a primal solution (e.g. infeasible).
    fn objective_value(&self) -> Option<f64>;

    /// Returns the primal solution vector from the most recent call to
    /// [`solve`](Self::solve), indexed by variable ID.
    ///
    /// Returns an empty `Vec` if `solve` has not been called or produced no
    /// primal solution.
    ///
    /// # TODO:
    ///
    /// Return a richer named-variable map once [`VarId`](crate::VarId) metadata
    /// is more expressive.
    fn solution_vector(&self) -> Vec<f64>;

    /// Returns a human-readable name for this solver.
    ///
    /// Used in diagnostics, error messages, and logging.  The default
    /// implementation returns `"<unnamed solver>"`; override with a
    /// descriptive name.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn name(&self) -> &str { "primal-simplex" }
    /// ```
    fn name(&self) -> &str {
        "<unnamed solver>"
    }
}
