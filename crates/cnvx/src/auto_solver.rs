use cnvx_core::{Model, Solution, SolveError};
use cnvx_lp::LpAutoSolver;
use cnvx_math::Matrix;

// === Top-level auto solver ===
pub enum AutoSolver<'model, A: Matrix> {
    LP(LpAutoSolver<'model, A>),
    // IP(IpAutoSolver<'model>),
    // QP(QpAutoSolver<'model>), etc.
}

impl<'model, A: Matrix> AutoSolver<'model, A> {
    pub fn new(model: &'model Model) -> Self {
        // For now, just assume it's an LP. We can add more heuristics later.
        AutoSolver::LP(LpAutoSolver::new(model))
    }

    pub fn solve(&mut self) -> Result<Solution, SolveError> {
        match self {
            AutoSolver::LP(s) => s.solve(),
            // AutoSolver::IP(s) => s.solve(),
        }
    }
}
