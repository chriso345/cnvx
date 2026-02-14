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
/// # impl Solver<'_> for DummySolver {
/// #   fn new(model: &Model) -> Self { DummySolver {} }
/// #   fn solve(&mut self) -> Result<Solution, SolveError> {
/// #       Err(SolveError::Unsupported("DummySolver does not implement solving".to_string()))
/// #   }
/// #   fn get_objective_value(&self) -> f64 { 0.0 }
/// #   fn get_solution(&self) -> Vec<f64> { vec![] }
/// # }
/// let mut model = Model::new();
/// // ... build model variables, constraints, objective ...
///
/// let mut solver = DummySolver::new(&model);
/// let result: Result<_, SolveError> = solver.solve();
/// match result {
///     Ok(solution) => println!("Optimal solution: {}", solution.objective_value.unwrap_or(0.0)),
///     Err(e) => println!("Solver error: {}", e),
/// }
/// ```
///
/// Generic over:
/// - `S`: solver state type
pub trait Solver<'model> {
    /// Create a new solver for the given model.
    fn new(model: &'model Model) -> Self
    where
        Self: Sized;

    /// Solve the model.
    fn solve(&mut self) -> Result<Solution, SolveError>;

    /// Get the current objective value (if available).
    fn get_objective_value(&self) -> f64;

    /// Return the current solution vector.
    fn get_solution(&self) -> Vec<f64>;
}
