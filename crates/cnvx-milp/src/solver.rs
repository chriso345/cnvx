use crate::method::MilpMethod;

/// A configured MILP solver, built fluently, mirroring
/// [`cnvx_lp::LpSolver`]'s shape.
///
/// # Examples
///
/// ```rust
/// # use cnvx_milp::MilpSolver;
/// let solver = MilpSolver::branch_and_bound().mip_gap(1e-3).node_limit(10_000);
/// ```
#[derive(Clone, Debug)]
pub struct MilpSolver {
    method: MilpMethod,
    pub(crate) lp_tolerance: f64,
    pub(crate) integrality_tolerance: f64,
    pub(crate) mip_gap: f64,
    pub(crate) node_limit: u64,
}

impl MilpSolver {
    fn with_method(method: MilpMethod) -> Self {
        MilpSolver {
            method,
            lp_tolerance: 1e-9,
            integrality_tolerance: 1e-6,
            mip_gap: 1e-4,
            node_limit: 100_000,
        }
    }

    /// Configures the solver to use [`MilpMethod::BranchAndBound`].
    pub fn branch_and_bound() -> Self {
        MilpSolver::with_method(MilpMethod::BranchAndBound)
    }

    /// Configures the solver to use [`MilpMethod::Binary`]. Solving will
    /// return `Err` unless every variable in the model is
    /// `VarKind::Binary`.
    pub fn binary() -> Self {
        MilpSolver::with_method(MilpMethod::Binary)
    }

    /// Configures the solver to use [`MilpMethod::BranchAndCut`].
    pub fn branch_and_cut() -> Self {
        MilpSolver::with_method(MilpMethod::BranchAndCut)
    }

    /// Configures the solver to use [`MilpMethod::BranchAndPrice`].
    pub fn branch_and_price() -> Self {
        MilpSolver::with_method(MilpMethod::BranchAndPrice)
    }

    /// The configured method.
    pub fn method(&self) -> MilpMethod {
        self.method
    }

    /// Sets the numerical tolerance used by the LP relaxation solved at
    /// every node. Default: `1e-9`. See [`cnvx_lp::LpSolver::tolerance`].
    pub fn lp_tolerance(mut self, tolerance: f64) -> Self {
        self.lp_tolerance = tolerance;
        self
    }

    /// Sets how close a relaxation value must be to the nearest integer
    /// to be treated as integral, rather than branched on. Default:
    /// `1e-6`.
    pub fn integrality_tolerance(mut self, tolerance: f64) -> Self {
        self.integrality_tolerance = tolerance;
        self
    }

    /// Sets the relative optimality gap at which a node is pruned even
    /// though its relaxation bound hasn't been strictly beaten by the
    /// incumbent yet. Default: `1e-4`.
    ///
    /// A search that stops because of this tolerance still reports
    /// [`cnvx_core::Status::Optimal`] as the incumbent is optimal within
    /// this gap, not necessarily exactly.
    pub fn mip_gap(mut self, gap: f64) -> Self {
        self.mip_gap = gap;
        self
    }

    /// Sets the maximum number of branch-and-bound nodes to explore
    /// before giving up with [`cnvx_core::Status::IterationLimit`].
    /// Default: `100_000`.
    pub fn node_limit(mut self, node_limit: u64) -> Self {
        self.node_limit = node_limit;
        self
    }
}

impl Default for MilpSolver {
    /// Dispatches to [`MilpMethod::BranchAndBound`].
    fn default() -> Self {
        MilpSolver::branch_and_bound()
    }
}
