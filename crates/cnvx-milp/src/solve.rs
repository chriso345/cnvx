use cnvx_core::{CnvxError, Model, Solve, VarKind};

use crate::branch_and_bound;
use crate::method::MilpMethod;
use crate::solution::MilpSolution;
use crate::solver::MilpSolver;

impl Solve<MilpSolver> for Model {
    type Solution = MilpSolution;

    fn solve(&self, solver: &MilpSolver) -> Result<MilpSolution, CnvxError> {
        match solver.method() {
            MilpMethod::BranchAndCut => Err(CnvxError::Numerical(
                "MilpMethod::BranchAndCut is not implemented yet; use MilpMethod::BranchAndBound".to_string(),
            )),
            MilpMethod::BranchAndPrice => Err(CnvxError::Numerical(
                "MilpMethod::BranchAndPrice is not implemented yet; use MilpMethod::BranchAndBound".to_string(),
            )),
            MilpMethod::Binary => {
                // TODO: Replace with a specialized binary solver that uses bitsets and other optimizations for binary variables.
                validate_binary(self)?;
                branch_and_bound::solve(self, solver)
            }
            MilpMethod::BranchAndBound => branch_and_bound::solve(self, solver),
        }
    }
}

/// Checks that every variable in `model` is `VarKind::Binary`, as
/// required by [`MilpMethod::Binary`].
fn validate_binary(model: &Model) -> Result<(), CnvxError> {
    for v in model.vars() {
        let kind = model.kind(v)?;
        if kind != VarKind::Binary {
            let name =
                model.var_name(v)?.map(|n| format!(" \"{n}\"")).unwrap_or_default();
            return Err(CnvxError::Numerical(format!(
                "MilpMethod::Binary requires every variable to be VarKind::Binary; variable{name} is \
                 {kind}. Use MilpMethod::BranchAndBound for mixed-integer or mixed-kind problems."
            )));
        }
    }
    Ok(())
}
