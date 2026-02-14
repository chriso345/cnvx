use cnvx_core::{Model, Solution, SolveError, Solver};
use cnvx_lp::LpAutoSolver;

pub struct AutoSolver<'model> {
    solver: Box<dyn Solver<'model> + 'model>,
}

impl<'model> Solver<'model> for AutoSolver<'model> {
    fn new(model: &'model Model) -> Self {
        // For now, assume LP
        let solver = Box::new(LpAutoSolver::new(model));

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
