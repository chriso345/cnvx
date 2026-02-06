use cnvx_core::*;

/// Validates a linear programming model before solving.
///
/// Checks that the model has a defined objective function.
/// You can add more LP-specific checks here in the future.
///
/// # Errors
///
/// Returns [`SolveError::NoObjective`] if the model does not have an objective.
pub fn check_lp(model: &Model) -> Result<(), SolveError> {
    if model.objective().is_none() {
        return Err(SolveError::NoObjective);
    }
    Ok(())
}
