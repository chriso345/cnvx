use cnvx_core::*;

pub fn check_lp(model: &Model) -> Result<(), SolveError> {
    if model.objective().is_none() {
        return Err(SolveError::NoObjective);
    }
    Ok(())
}
