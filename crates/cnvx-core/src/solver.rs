use crate::{Model, Solution, SolveError};

/// Trait for optimization solvers.
///
/// Any struct implementing this trait can solve a [`Model`] and produce a [`Solution`].
/// This trait provides a consistent interface across different solver implementations,
/// such as simplex, interior point, branch-and-bound, or lexicographic solvers.
///
/// ```rust
/// # use cnvx_core::{Model, Solver, SolveError, Solution};
/// # struct DummySolver {}
/// #
/// # impl Default for DummySolver {
/// #   fn default() -> Self { Self {} }
/// # }
/// #
/// # impl Solver for DummySolver {
/// #   fn solve(&self, _model: &Model) -> Result<Solution, SolveError> {
/// #       Err(SolveError::Unsupported("DummySolver does not implement solving".to_string()))
/// #   }
/// # }
/// let mut model = Model::new();
/// // ... build model variables, constraints, objective ...
///
/// let solver = DummySolver::default();
/// let result: Result<_, SolveError> = solver.solve(&model);
/// match result {
///     Ok(solution) => println!("Optimal solution: {}", solution.objective_value.unwrap_or(0.0)),
///     Err(e) => println!("Solver error: {}", e),
/// }
/// ```
pub trait Solver: Default {
    /// Solves the given model and returns a solution or an error.
    /// - [`Ok(Solution)`](Solution) if the model is solved successfully, containing variable assignments and objective value.
    /// - [`Err(SolveError)`](SolveError) if the solver encounters an error, such as invalid model, numerical issues, or unsupported features.
    fn solve(&self, model: &Model) -> Result<Solution, SolveError>;
}
