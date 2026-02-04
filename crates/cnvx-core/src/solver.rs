use crate::{Model, Solution, SolveError};

pub trait Solver: Default {
    fn solve(&self, model: &Model) -> Result<Solution, SolveError>;
}
