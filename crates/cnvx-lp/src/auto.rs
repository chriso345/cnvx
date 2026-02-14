use cnvx_core::{Model, Solution, SolveError, Solver};

use crate::PrimalSimplexSolver;

pub struct LpAutoSolver<'model> {
    solver: Box<dyn Solver<'model> + 'model>,
}

impl<'model> Solver<'model> for LpAutoSolver<'model> {
    fn new(model: &'model Model) -> Self {
        let solver = Box::new(PrimalSimplexSolver::new(model));

        Self { solver }
    }

    fn solve(&mut self) -> Result<Solution, SolveError> {
        self.solver.solve()
    }

    fn get_objective_value(&self) -> f64 {
        todo!()
    }

    fn get_solution(&self) -> Vec<f64> {
        todo!()
    }
}
