use crate::expr::Expression;
use crate::sense::Sense;

/// A handle to one objective on a [`Model`](crate::Model).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ObjectiveId {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

/// One objective on a [`Model`](crate::Model): a linear expression, an
/// optimization direction, and a priority.
#[derive(Clone, Debug)]
pub struct Objective {
    /// The optimization direction.
    pub sense: Sense,
    /// The linear expression being optimized.
    pub expr: Expression,
    /// The tier this objective is optimized in by a lexicographic
    /// solver. Higher values are optimized first.
    pub priority: i32,
    /// A human-readable name, for reporting/debugging.
    pub name: Option<String>,
}
