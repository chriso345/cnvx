// TODO: Should these all be errors, or should some be the solver state??
#[derive(Debug)]
pub enum SolveError {
    Infeasible,
    Unbounded,
    NoObjective,
    InvalidModel(String),
    Other(String),
}
