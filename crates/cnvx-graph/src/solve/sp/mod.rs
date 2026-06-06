pub mod labelset;
pub mod pareto;

pub use labelset::*;
pub use pareto::*;

#[derive(Clone, Debug)]
pub enum SPType {
    OneToOne(usize, usize),
    OneToAll(usize),
    // AllToAll,
    // OneToSubset(usize, Vec<usize>),
    // SubsetToOne(Vec<usize>, usize),
    // SubsetToSubset(Vec<usize>, Vec<usize>),
    None,
}
