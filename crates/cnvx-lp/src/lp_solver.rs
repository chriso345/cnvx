//! # LP Solver
//!
//! ```rust
//! use cnvx_core::{Model, Objective, solver::Solver};
//! use cnvx_lp::LpSolver;
//!
//! let mut model = Model::new();
//! let x = model.add_var().finish();
//! model.add_objective(Objective::maximize(x * 2.0).name("Z"));
//!
//! let mut solver = LpSolver::new();
//! let solution = solver.solve(&model).unwrap();
//! ```

use cnvx_core::{Solution, SolveError, problem::Problem, solver::Solver};

use crate::{DualSimplexSolver, PrimalSimplexSolver};

/// The recommended entry point for solving LP problems with `cnvx-lp`.
///
/// Internally holds a ranked list of LP solvers and delegates to the first one
/// that [`supports`](Solver::supports) the given problem.  The list is
/// constructed once at [`LpSolver::new()`].
///
/// See the [module-level documentation](self) for the current solver ranking.
pub struct LpSolver {
    /// Ranked list of candidate solvers, tried in order.
    ///
    /// The first solver for which `supports(problem)` returns `true` is used.
    /// If none match, `solve` returns [`SolveError::Unsupported`].
    solvers: Vec<Box<dyn Solver>>,
}

impl LpSolver {
    /// Creates a new `LpSolver` with the default solver ranking.
    ///
    /// The ranking can be customised after construction via
    /// [`push_solver`](Self::push_solver) or by building the solver list
    /// manually with [`from_solvers`](Self::from_solvers).
    pub fn new() -> Self {
        Self {
            solvers: vec![
                // Primal simplex first: fully implemented.
                Box::new(PrimalSimplexSolver::new()),
                // Dual simplex second: will take precedence for warm-started
                // re-optimisation once implemented.
                Box::new(DualSimplexSolver::new()),
            ],
        }
    }

    /// Creates an `LpSolver` from a custom ordered list of solvers.
    ///
    /// Solvers are tried in the order they appear in `solvers`.  This is the
    /// escape hatch for users who want precise control over the fallback chain.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cnvx_lp::{LpSolver, PrimalSimplexSolver};
    ///
    /// let solver = LpSolver::from_solvers(vec![
    ///     Box::new(PrimalSimplexSolver::new()),
    /// ]);
    /// ```
    pub fn from_solvers(solvers: Vec<Box<dyn Solver>>) -> Self {
        Self { solvers }
    }

    /// Appends a solver to the end of the candidate list (lowest priority).
    ///
    /// Useful for registering a fallback solver without rebuilding the whole
    /// list.
    pub fn push_solver(&mut self, solver: Box<dyn Solver>) {
        self.solvers.push(solver);
    }

    /// Returns the name of the solver that would be selected for `problem`,
    /// or `None` if no registered solver supports it.
    ///
    /// Useful for diagnostic output ("Using solver: primal-simplex").
    pub fn selected_for(&self, problem: &dyn Problem) -> Option<&str> {
        self.solvers.iter().find(|s| s.supports(problem)).map(|s| s.name())
    }
}

impl Default for LpSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Solver for LpSolver {
    fn name(&self) -> &str {
        "lp-solver"
    }

    /// Returns `true` if at least one candidate solver supports `problem`.
    fn supports(&self, problem: &dyn Problem) -> bool {
        self.solvers.iter().any(|s| s.supports(problem))
    }

    /// Delegates to the first candidate solver that supports `problem`.
    ///
    /// # Errors
    ///
    /// Returns [`SolveError::Unsupported`] if no registered solver supports
    /// the problem.  All other errors are propagated from the chosen solver.
    fn solve(&mut self, problem: &dyn Problem) -> Result<Solution, SolveError> {
        let solver =
            self.solvers.iter_mut().find(|s| s.supports(problem)).ok_or_else(|| {
                SolveError::Unsupported(format!(
                    "LpSolver: no solver supports a '{}' problem",
                    problem.kind()
                ))
            })?;

        solver.solve(problem)
    }

    fn objective_value(&self) -> Option<f64> {
        // Return the objective from whichever internal solver last ran.
        // In practice the caller should use the Solution returned by solve().
        self.solvers.iter().find_map(|s| s.objective_value())
    }

    fn solution_vector(&self) -> Vec<f64> {
        self.solvers
            .iter()
            .find(|s| !s.solution_vector().is_empty())
            .map(|s| s.solution_vector())
            .unwrap_or_default()
    }
}
