use cnvx_core::{CnvxError, Model, Sense, Solve};

use crate::method::MopMethod;
use crate::solution::MopSolution;
use crate::solver::MopSolver;
use crate::util::{solve_single_point, weighted_sum_expr};
use crate::{biobjective, epsilon_constraint, lexicographic};

impl Solve<MopSolver> for Model {
    type Solution = MopSolution;

    fn solve(&self, solver: &MopSolver) -> Result<MopSolution, CnvxError> {
        match solver.method() {
            MopMethod::Lexicographic => lexicographic::solve(self, solver),
            MopMethod::EpsilonConstraint { primary, limits } => {
                epsilon_constraint::solve(self, *primary, limits, solver)
            }
            MopMethod::WeightedSum { weights } => {
                let ids: Vec<_> = self.objectives().map(|(id, _)| id).collect();
                if weights.len() != ids.len() {
                    return Err(CnvxError::DimensionMismatch {
                        expected: ids.len(),
                        got: weights.len(),
                    });
                }
                let pairs: Vec<_> =
                    ids.into_iter().zip(weights.iter().copied()).collect();
                let expr = weighted_sum_expr(self, &pairs)?;
                let working = self.with_objective(Sense::Minimize, expr)?;
                solve_single_point(&working, self, solver)
            }
            MopMethod::Biobjective => biobjective::solve(self, solver),
        }
    }
}
