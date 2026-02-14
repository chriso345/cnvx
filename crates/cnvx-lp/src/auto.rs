use cnvx_core::Solver;

#[derive(Debug, Default)]
pub struct LpAutoSolver {}

// impl Default for LpAutoSolver {
//     fn default() -> Self {
//         Self {}
//     }
// }

impl Solver for LpAutoSolver {
    fn solve(
        &self,
        model: &cnvx_core::Model,
    ) -> Result<cnvx_core::Solution, cnvx_core::SolveError> {
        _ = model;
        // For now always use the primal simplex solver. In the future logic will need to
        //  be added to choose which solver to use based on the problem shape and characteristics.
        let primal_simplex_solver = crate::PrimalSimplexSolver::default();
        primal_simplex_solver.solve(model)
    }
}
