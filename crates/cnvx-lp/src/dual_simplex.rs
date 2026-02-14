use cnvx_core::Solver;

#[derive(Debug)]
pub struct DualSimplexSolver {}

impl Default for DualSimplexSolver {
    fn default() -> Self {
        Self {}
    }
}

impl Solver for DualSimplexSolver {
    fn solve(
        &self,
        model: &cnvx_core::Model,
    ) -> Result<cnvx_core::Solution, cnvx_core::SolveError> {
        _ = model;
        todo!()
    }
}
