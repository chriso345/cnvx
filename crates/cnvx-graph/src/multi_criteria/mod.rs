mod lexicographic;
mod pareto;
mod scalarize;

pub use lexicographic::{LexPriority, LexicographicShortestPath};
pub use pareto::{ParetoPath, ParetoShortestPath, ParetoShortestPathSolution};
pub use scalarize::scalarize_weighted_sum;

pub use crate::weight::{CriteriaFn, Dominance};
