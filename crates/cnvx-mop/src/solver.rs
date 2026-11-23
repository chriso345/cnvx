use std::time::Duration;

use cnvx_core::ObjectiveId;
use cnvx_lp::LpSolver;
use cnvx_milp::MilpSolver;

use crate::method::MopMethod;

/// A configured multi-objective solver
///
/// # Examples
///
/// ```rust
/// # use cnvx_mop::MopSolver;
/// let solver = MopSolver::lexicographic().tolerance(1e-6).mip_gap(1e-3);
/// ```
#[derive(Clone, Debug)]
pub struct MopSolver {
    method: MopMethod,
    pub(crate) lp_tolerance: f64,
    pub(crate) max_iterations: u32,
    pub(crate) time_limit: Option<Duration>,
    pub(crate) mip_gap: f64,
    pub(crate) node_limit: u64,
    pub(crate) tolerance: f64,
}

impl MopSolver {
    fn with_method(method: MopMethod) -> Self {
        MopSolver {
            method,
            lp_tolerance: 1e-9,
            max_iterations: 10_000,
            time_limit: None,
            mip_gap: 1e-4,
            node_limit: 100_000,
            tolerance: 1e-7,
        }
    }

    /// Configures the solver to use [`MopMethod::Lexicographic`].
    pub fn lexicographic() -> Self {
        MopSolver::with_method(MopMethod::Lexicographic)
    }

    /// Configures the solver to use [`MopMethod::EpsilonConstraint`].
    pub fn epsilon_constraint(
        primary: ObjectiveId,
        limits: Vec<(ObjectiveId, f64)>,
    ) -> Self {
        MopSolver::with_method(MopMethod::EpsilonConstraint { primary, limits })
    }

    /// Configures the solver to use [`MopMethod::WeightedSum`].
    pub fn weighted_sum(weights: Vec<f64>) -> Self {
        MopSolver::with_method(MopMethod::WeightedSum { weights })
    }

    /// Configures the solver to use [`MopMethod::Biobjective`].
    pub fn biobjective() -> Self {
        MopSolver::with_method(MopMethod::Biobjective)
    }

    /// The configured method.
    pub fn method(&self) -> &MopMethod {
        &self.method
    }

    /// Sets the numerical tolerance forwarded to every LP relaxation
    /// solve.
    pub fn lp_tolerance(mut self, tolerance: f64) -> Self {
        self.lp_tolerance = tolerance;
        self
    }

    /// Sets the maximum number of simplex iterations forwarded to every
    /// LP solve.
    pub fn max_iterations(mut self, max_iterations: u32) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Sets a time limit, in seconds, forwarded to every LP solve.
    pub fn time_limit<T: Into<f64>>(mut self, seconds: T) -> Self {
        let seconds = seconds.into();
        assert!(
            seconds.is_finite() && seconds >= 0.0,
            "time limit must be a non-negative finite number"
        );

        self.time_limit = Some(Duration::from_secs_f64(seconds));
        self
    }

    /// Sets the relative optimality gap forwarded to every
    /// mixed-integer solve.
    pub fn mip_gap(mut self, gap: f64) -> Self {
        self.mip_gap = gap;
        self
    }

    /// Sets the branch-and-bound node limit forwarded to every
    /// mixed-integer solve.
    pub fn node_limit(mut self, node_limit: u64) -> Self {
        self.node_limit = node_limit;
        self
    }

    /// Sets the general numerical tolerance this crate's own
    /// multi-objective logic uses: how tightly
    /// [`MopMethod::Lexicographic`] fixes a tier's achieved value
    /// before moving to the next tier, and how close two points'
    /// objective vectors must be for
    /// [`MopMethod::Biobjective`] to treat them as
    /// the same point. Default: `1e-7`.
    pub fn tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// The [`cnvx_lp::LpSolver`] this solver uses for every LP
    /// subproblem, built from `lp_tolerance`, `max_iterations`, and
    /// `time_limit`.
    pub(crate) fn lp_solver(&self) -> LpSolver {
        let mut solver = LpSolver::primal_simplex()
            .tolerance(self.lp_tolerance)
            .max_iterations(self.max_iterations);
        if let Some(time_limit) = self.time_limit {
            solver = solver.time_limit(time_limit.as_secs_f64());
        }
        solver
    }

    /// The [`cnvx_milp::MilpSolver`] this solver uses for every
    /// mixed-integer subproblem, built from `lp_tolerance`, `mip_gap`,
    /// and `node_limit`.
    pub(crate) fn milp_solver(&self) -> MilpSolver {
        MilpSolver::branch_and_bound()
            .lp_tolerance(self.lp_tolerance)
            .mip_gap(self.mip_gap)
            .node_limit(self.node_limit)
    }
}

impl Default for MopSolver {
    /// Dispatches to [`MopMethod::Lexicographic`].
    fn default() -> Self {
        MopSolver::lexicographic()
    }
}
