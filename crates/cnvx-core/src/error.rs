#[derive(Debug)]
pub enum SolveError {
    Infeasible,
    Unbounded,
    NoObjective,
    InvalidModel(String),
}
