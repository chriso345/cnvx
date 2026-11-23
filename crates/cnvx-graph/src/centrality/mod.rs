mod betweenness;
mod closeness;
mod degree;
mod eigenvector;

pub use betweenness::Betweenness;
pub use closeness::Closeness;
use cnvx_core::Status;
pub use degree::{Degree, DegreeMode};
pub use eigenvector::Eigenvector;

use crate::NodeId;

/// The result of every centrality measure in this module: one score per
/// node. Higher is always "more central" for every measure here.
pub struct CentralityScores {
    pub(crate) status: Status,
    pub(crate) score: Vec<f64>,
}

impl CentralityScores {
    pub fn status(&self) -> Status {
        self.status
    }

    /// The score for `node`.
    pub fn score(&self, node: NodeId) -> f64 {
        self.score.get(node.index()).copied().unwrap_or(0.0)
    }
}

impl std::ops::Index<NodeId> for CentralityScores {
    type Output = f64;

    fn index(&self, node: NodeId) -> &f64 {
        &self.score[node.index()]
    }
}
