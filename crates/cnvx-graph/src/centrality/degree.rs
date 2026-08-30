use cnvx_core::{CnvxError, Status};

use super::CentralityScores;
use crate::common::dispatch::impl_solve;
use crate::graph::GraphRef;

/// Which degree to score by, for a directed graph (ignored for an
/// undirected graph, where all three coincide).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DegreeMode {
    /// In-degree plus out-degree.
    Total,
    /// In-degree only.
    In,
    /// Out-degree only.
    Out,
}

/// Degree centrality: each node's score is its degree, optionally
/// normalized to `[0, 1]` by dividing by `node_count - 1` (the maximum
/// possible degree in a simple graph).
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::centrality::Degree;
///
/// let mut g = Graph::<(), ()>::undirected();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, ()).unwrap();
/// g.add_edge(a, c, ()).unwrap();
///
/// let result = (&g).solve(&Degree::new()).unwrap();
/// assert_eq!(result.score(a), 2.0);
/// assert_eq!(result.score(b), 1.0);
/// ```
pub struct Degree {
    mode: DegreeMode,
    normalized: bool,
}

impl Default for Degree {
    fn default() -> Self {
        Self::new()
    }
}

impl Degree {
    /// Creates an (unnormalized, total-degree) degree centrality solver.
    pub fn new() -> Self {
        Self { mode: DegreeMode::Total, normalized: false }
    }

    /// Sets which degree to score by (directed graphs only).
    pub fn mode(mut self, mode: DegreeMode) -> Self {
        self.mode = mode;
        self
    }

    /// Normalizes scores to `[0, 1]`.
    pub fn normalized(mut self) -> Self {
        self.normalized = true;
        self
    }
}

fn degree_impl<G, N, E>(graph: G, solver: &Degree) -> Result<CentralityScores, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut score = vec![0.0; n];
    for id in graph.node_ids() {
        let d = match solver.mode {
            DegreeMode::Total => {
                if graph.is_directed() {
                    graph.in_edges(id).count() + graph.out_edges(id).count()
                } else {
                    graph.out_edges(id).count()
                }
            }
            DegreeMode::In => {
                if graph.is_directed() {
                    graph.in_edges(id).count()
                } else {
                    graph.out_edges(id).count()
                }
            }
            DegreeMode::Out => graph.out_edges(id).count(),
        };
        score[id.index()] = d as f64;
    }
    if solver.normalized && n > 1 {
        let denom = (n - 1) as f64;
        for s in &mut score {
            *s /= denom;
        }
    }
    Ok(CentralityScores { status: Status::Optimal, score })
}

impl_solve!(Degree => CentralityScores, degree_impl);
