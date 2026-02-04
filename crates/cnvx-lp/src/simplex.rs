use cnvx_core::*;

pub struct SimplexSolver {
    pub tolerance: f64,
    pub max_iterations: usize,
}

impl Solver for SimplexSolver {
    fn solve(&self, model: &Model) -> Result<Solution, SolveError> {
        crate::validate::check_lp(model)?;

        println!("SimplexSolver.solve called with model: {:?}", model);
        println!(
            "Tolerance: {}, Max Iterations: {}",
            self.tolerance, self.max_iterations
        );

        Err(SolveError::InvalidModel("Simplex not implemented yet".into()))
    }
}

impl Default for SimplexSolver {
    fn default() -> Self {
        Self { tolerance: 1e-8, max_iterations: 1000 }
    }
}
