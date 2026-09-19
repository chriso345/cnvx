use cnvx_core::CnvxError;

use super::bellman_ford::bellman_ford_impl;
use super::dijkstra::dijkstra_impl;
use super::solution::ShortestPathSolution;
use super::{BellmanFord, Dijkstra};
use crate::NodeId;
use crate::common::dispatch::impl_solve;
use crate::graph::GraphRef;
use crate::weight::WeightFn;

/// Runs Dijkstra first (the fast path, valid whenever every edge weight
/// happens to be non-negative), and transparently falls back to
/// Bellman-Ford only if a negative weight is actually encountered.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::shortest_path::ShortestPath;
///
/// let mut g = Graph::<(), f64>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// g.add_edge(a, b, -3.0).unwrap();
///
/// // Falls back to Bellman-Ford automatically because of the negative weight.
/// let result = (&g).solve(&ShortestPath::new(a).weight(|w: &f64| *w)).unwrap();
/// assert_eq!(result.distance_to(b), Some(-3.0));
/// ```
pub struct ShortestPath<'a, E> {
    source: NodeId,
    weight: Option<WeightFn<'a, E>>,
}

impl<'a, E> ShortestPath<'a, E> {
    /// Creates a shortest-path search rooted at `source`.
    pub fn new(source: NodeId) -> Self {
        Self { source, weight: None }
    }

    /// Sets the edge-weight accessor.
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }
}

fn dispatch_impl<G, N, E>(
    graph: G,
    solver: &ShortestPath<'_, E>,
) -> Result<ShortestPathSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let Some(weight) = &solver.weight else {
        return Err(CnvxError::Numerical(
            "ShortestPath: no weight accessor set; call .weight(...)".to_string(),
        ));
    };

    let has_negative = graph
        .edge_ids()
        .any(|e| graph.edge_weight(e).is_some_and(|d| weight(d) < 0.0));
    if has_negative {
        bellman_ford_impl(
            graph,
            &BellmanFord::new(solver.source).weight(|d: &E| weight(d)),
        )
    } else {
        dijkstra_impl(graph, &Dijkstra::new(solver.source).weight(|d: &E| weight(d)))
    }
}

impl_solve!(ShortestPath<'_, E> => ShortestPathSolution, dispatch_impl);
