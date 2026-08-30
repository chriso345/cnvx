/// A session of shared defaults: logging, thread count, and tolerances,
/// set once and reused by every [`Model`](crate::Model) built afterward.
///
/// # Examples
///
/// ```rust
/// # use cnvx_core::{Env, Params};
/// let env = Env::with_params(Params {
///     time_limit: Some(30.0),
///     threads: Some(8),
///     ..Default::default()
/// });
/// ```
#[derive(Debug)]
pub struct Env {
    pub params: Params,
}

impl Env {
    /// Creates an `Env` with sane, silent defaults.
    pub fn new() -> Self {
        Env { params: Params::default() }
    }

    /// Creates an `Env` with the given default parameters.
    pub fn with_params(params: Params) -> Self {
        Env { params }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

/// One parameter bag, shared vocabulary across every solver in the
/// workspace. Per-solve overrides on a solver builder (e.g.
/// `LpSolver::primal_simplex().tolerance(1e-9)`) take precedence over
/// whatever an `Env`/`Model` supplies as a default.
#[derive(Clone, Debug)]
pub struct Params {
    /// Wall-clock solve time limit, in seconds.
    pub time_limit: Option<f64>,
    /// Relative MIP optimality gap at which branch and bound stops.
    pub mip_gap: f64,
    /// Primal feasibility tolerance.
    pub feasibility_tol: f64,
    /// Dual/optimality tolerance.
    pub optimality_tol: f64,
    /// Number of threads to use, or `None` to let the solver choose.
    pub threads: Option<u32>,
    /// Whether solvers should log progress.
    pub output: bool,
    /// Whether solvers should presolve the model before solving.
    pub presolve: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            time_limit: None,
            mip_gap: 1e-4,
            feasibility_tol: 1e-6,
            optimality_tol: 1e-6,
            threads: None,
            output: true,
            presolve: true,
        }
    }
}
