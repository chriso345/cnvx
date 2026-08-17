use crate::{Basis, LpMethod};

/// A configured LP solver.
///
/// # Examples
///
/// ```rust
/// # use cnvx_lp::LpSolver;
/// let solver = LpSolver::primal_simplex().tolerance(1e-9).max_iterations(5_000);
/// ```
#[derive(Clone, Debug)]
pub struct LpSolver {
    method: LpMethod,
    pub(crate) tolerance: f64,
    pub(crate) max_iterations: u32,
    pub(crate) warm_start: Option<Basis>,
}

impl LpSolver {
    fn with_method(method: LpMethod) -> Self {
        LpSolver {
            method,
            tolerance: 1e-9,
            max_iterations: 10_000,
            warm_start: None,
        }
    }

    /// Configures the solver to use [`LpMethod::PrimalSimplex`].
    pub fn primal_simplex() -> Self {
        LpSolver::with_method(LpMethod::PrimalSimplex)
    }

    /// Configures the solver to use [`LpMethod::DualSimplex`].
    ///
    /// Not yet implemented; see [`LpMethod::DualSimplex`].
    pub fn dual_simplex() -> Self {
        LpSolver::with_method(LpMethod::DualSimplex)
    }

    /// Configures the solver to use [`LpMethod::InteriorPoint`].
    ///
    /// Not yet implemented; see [`LpMethod::InteriorPoint`].
    pub fn interior_point() -> Self {
        LpSolver::with_method(LpMethod::InteriorPoint)
    }

    /// The configured method.
    pub fn method(&self) -> LpMethod {
        self.method
    }

    /// Sets the numerical tolerance used for feasibility and optimality
    /// checks.
    pub fn tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Sets the maximum number of simplex iterations.
    pub fn max_iterations(mut self, max_iterations: u32) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Seeds the solve from a previously computed [`Basis`].
    /// Not yet used by any solver.
    pub fn warm_start(mut self, basis: Basis) -> Self {
        self.warm_start = Some(basis);
        self
    }
}

impl Default for LpSolver {
    /// Dispatches to [`LpMethod::PrimalSimplex`].
    ///
    /// TODO: Add auto detection of the best method to use, based on the
    /// model's structure and the solver's parameters.
    fn default() -> Self {
        LpSolver::primal_simplex()
    }
}
