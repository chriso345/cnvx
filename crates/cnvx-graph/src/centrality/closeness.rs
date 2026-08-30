use cnvx_core::{CnvxError, Status};

use super::CentralityScores;
use crate::common::dispatch::impl_solve;
use crate::graph::GraphRef;
use crate::traversal::bfs_impl;
use crate::weight::WeightFn;

/// Closeness centrality: for node `v` reaching `r` other nodes at total
/// distance `sum_d`, the score is `(r / (n - 1)) * (r / sum_d)`.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::centrality::Closeness;
///
/// // a - b - c: b is maximally close to both others.
/// let mut g = Graph::<(), ()>::undirected();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, ()).unwrap();
/// g.add_edge(b, c, ()).unwrap();
///
/// let result = (&g).solve(&Closeness::new()).unwrap();
/// assert!(result.score(b) > result.score(a));
/// ```
pub struct Closeness<'a, E> {
    weight: Option<WeightFn<'a, E>>,
}

impl<'a, E> Default for Closeness<'a, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E> Closeness<'a, E> {
    /// Creates an unweighted (BFS-based) closeness solver.
    pub fn new() -> Self {
        Self { weight: None }
    }

    /// Uses Dijkstra with this weight accessor instead of unweighted BFS.
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }
}

fn closeness_impl<G, N, E>(
    graph: G,
    solver: &Closeness<'_, E>,
) -> Result<CentralityScores, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut score = vec![0.0; n];
    if n <= 1 {
        return Ok(CentralityScores { status: Status::Optimal, score });
    }

    for source in graph.node_ids() {
        let (reached, total_distance) = if let Some(weight) = &solver.weight {
            let sp = crate::shortest_path::dijkstra_impl(
                graph,
                &crate::shortest_path::Dijkstra::new(source).weight(|d: &E| weight(d)),
            )?;
            let mut reached = 0usize;
            let mut total = 0.0;
            for v in graph.node_ids() {
                if v == source {
                    continue;
                }
                if let Some(d) = sp.distance_to(v) {
                    reached += 1;
                    total += d;
                }
            }
            (reached, total)
        } else {
            let bfs = bfs_impl(graph, &crate::traversal::Bfs::new(source))?;
            let mut reached = 0usize;
            let mut total = 0.0;
            for v in graph.node_ids() {
                if v == source {
                    continue;
                }
                if let Some(d) = bfs.depth(v) {
                    reached += 1;
                    total += d as f64;
                }
            }
            (reached, total)
        };
        if reached > 0 && total_distance > 0.0 {
            let r = reached as f64;
            score[source.index()] = (r / (n as f64 - 1.0)) * (r / total_distance);
        }
    }

    Ok(CentralityScores { status: Status::Optimal, score })
}

impl_solve!(Closeness<'_, E> => CentralityScores, closeness_impl);
