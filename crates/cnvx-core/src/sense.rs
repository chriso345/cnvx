/// The optimization direction of an objective function.
#[derive(Copy, Clone, Debug, PartialEq, Eq, strum::Display)]
pub enum Sense {
    /// Minimize the objective function.
    Minimize,
    /// Maximize the objective function.
    Maximize,
}
