use cnvx_core::{Model, Solution, SolveError, Solver};
use cnvx_math::Matrix;

use crate::{DualSimplexSolver, PrimalSimplexSolver};

pub enum LpAutoSolver<'model, A: Matrix> {
    Primal(PrimalSimplexSolver<'model, A>),
    Dual(DualSimplexSolver<'model, A>),
}

impl<'model, A: Matrix> LpAutoSolver<'model, A> {
    pub fn new(model: &'model Model) -> Self {
        // let (m, n) = model.shape();
        // if m > n {
        //     LpAutoSolver::Dual(DualSimplexSolver::new(model))
        // } else {
        //     LpAutoSolver::Primal(PrimalSimplexSolver::new(model))
        // }

        // For now, just use the primal simplex solver. We can add more heuristics later.
        LpAutoSolver::Primal(PrimalSimplexSolver::new(model))
    }

    pub fn solve(&mut self) -> Result<Solution, SolveError> {
        match self {
            LpAutoSolver::Primal(s) => s.solve(),
            LpAutoSolver::Dual(s) => s.solve(),
        }
    }
}
