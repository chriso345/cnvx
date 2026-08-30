use cnvx_core::{CnvxError, Status};

use super::CentralityScores;
use crate::NodeId;
use crate::common::dispatch::impl_solve;
use crate::graph::GraphRef;

/// Eigenvector centrality: the dominant eigenvector of the graph's
/// adjacency matrix, computed by power iteration
/// (`x_{t+1} = normalize(A x_t)`).
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::centrality::Eigenvector;
///
/// // A star: the center is connected to everyone, so it dominates.
/// let mut g = Graph::<(), ()>::undirected();
/// let center = g.add_node(());
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(center, a, ()).unwrap();
/// g.add_edge(center, b, ()).unwrap();
/// g.add_edge(center, c, ()).unwrap();
///
/// let result = (&g).solve(&Eigenvector::new()).unwrap();
/// assert!(result.score(center) > result.score(a));
/// ```
pub struct Eigenvector {
    tolerance: f64,
    max_iterations: u32,
}

impl Default for Eigenvector {
    fn default() -> Self {
        Self::new()
    }
}

impl Eigenvector {
    /// Creates an eigenvector centrality solver.
    pub fn new() -> Self {
        Self { tolerance: 1e-10, max_iterations: 1000 }
    }

    /// Sets the convergence tolerance and iteration budget.
    pub fn tolerance(mut self, tolerance: f64, max_iterations: u32) -> Self {
        self.tolerance = tolerance;
        self.max_iterations = max_iterations;
        self
    }
}

fn eigenvector_impl<G, N, E>(
    graph: G,
    solver: &Eigenvector,
) -> Result<CentralityScores, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let ids: Vec<NodeId> = graph.node_ids().collect();
    let n = ids.len();
    if n == 0 {
        return Ok(CentralityScores { status: Status::Optimal, score: Vec::new() });
    }
    let index_of = |id: NodeId| ids.iter().position(|&x| x == id).unwrap();

    let mut score = vec![1.0 / (n as f64).sqrt(); n];
    let mut converged = false;
    for _ in 0..solver.max_iterations {
        let mut next = vec![0.0; n];
        for &u in &ids {
            let ui = index_of(u);
            for v in graph.neighbors(u) {
                next[index_of(v)] += score[ui];
            }
        }
        let norm = next.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm == 0.0 {
            // No edges at all (or every neighbor score already zero):
            // there is no meaningful dominant eigenvector to converge to.
            break;
        }
        for x in &mut next {
            *x /= norm;
        }
        let diff: f64 = next.iter().zip(&score).map(|(a, b)| (a - b).abs()).sum();
        score = next;
        if diff < solver.tolerance {
            converged = true;
            break;
        }
    }

    Ok(CentralityScores {
        status: if converged { Status::Optimal } else { Status::IterationLimit },
        score,
    })
}

impl_solve!(Eigenvector => CentralityScores, eigenvector_impl);
