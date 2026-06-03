use cnvx_core::SolveError;

use crate::{LpModel, LpSolution};

/// Trait for optimization solvers.
///
/// Any struct implementing this trait can solve a [`Model`](crate::model::Model) and produce a [`Solution`].
/// This trait provides a consistent interface across different solver implementations,
/// such as simplex, interior point, branch-and-bound, or lexicographic solvers.
///
/// ```rust
/// use cnvx_core::{SolveError};
///
/// pub struct MySolver { /* internal state */ }
///
/// impl Solver for MySolver {
///     fn solve(
///         &mut self,
///         model: &LpModel,
///     ) -> Result<Solution, SolveError> {
///         todo!()
///     }
///
///     fn objective_value(&self) -> Option<f64> { todo!() }
///     fn solution_vector(&self) -> Vec<f64> { todo!() }
/// }
/// ```
pub trait Solver: Send {
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
    fn solve(&mut self, model: &LpModel) -> Result<LpSolution, SolveError>;

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
