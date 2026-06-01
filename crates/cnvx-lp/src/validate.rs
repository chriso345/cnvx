use cnvx_core::*;

use crate::LpModel;

/// Validates a linear programming model before solving.
///
/// Checks that the model has a defined objective function.
/// You can add more LP-specific checks here in the future.
///
/// # Errors
///
/// Returns [`SolveError::NoObjective`] if the model does not have an objective.
pub fn check_lp(model: &LpModel) -> Result<(), SolveError> {
    if model.objective().is_none() {
        return Err(SolveError::NoObjective);
    }
    Ok(())
}
