use cnvx_core::{CnvxError, Model, ObjectiveId, Sense};

use crate::solution::MopSolution;
use crate::solver::MopSolver;
use crate::util::solve_single_point;

/// Implements [`crate::MopMethod::EpsilonConstraint`].
///
/// # Errors
/// Returns `Err(CnvxError::ForeignHandle)` if `primary` or any id in
/// `limits` came from a different `Model`.
pub(crate) fn solve(
    model: &Model,
    primary: ObjectiveId,
    limits: &[(ObjectiveId, f64)],
    mop: &MopSolver,
) -> Result<MopSolution, CnvxError> {
    let primary_objective = model.objective_at(primary)?;
    let mut working =
        model.with_objective(primary_objective.sense, primary_objective.expr)?;

    for &(id, limit) in limits {
        let objective = model.objective_at(id)?;
        let constraint = match objective.sense {
            Sense::Minimize => objective.expr.leq(limit),
            Sense::Maximize => objective.expr.geq(limit),
        };
        working.add_constraint(constraint)?;
    }

    solve_single_point(&working, model, mop)
}
