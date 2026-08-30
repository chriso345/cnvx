use cnvx_core::{CnvxError, Status};

use super::solution::ShortestPathSolution;
use crate::NodeId;
use crate::common::dispatch::impl_solve;
use crate::common::distance_map::PredecessorMap;
use crate::graph::GraphRef;
use crate::weight::WeightFn;

/// Bellman-Ford: relaxes every edge up to `node_count - 1` times, then
/// performs one further pass to detect a negative-weight cycle reachable
/// from the source (a node whose distance can still be improved after
/// `V - 1` rounds must lie on, or be reachable through, such a cycle).
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::shortest_path::BellmanFord;
///
/// let mut g = Graph::<(), f64>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, 4.0).unwrap();
/// g.add_edge(a, c, 5.0).unwrap();
/// g.add_edge(c, b, -2.0).unwrap();
///
/// let result = (&g).solve(&BellmanFord::new(a).weight(|w: &f64| *w)).unwrap();
/// assert_eq!(result.distance_to(b), Some(3.0));
/// assert!(!result.has_negative_cycle());
/// ```
pub struct BellmanFord<'a, E> {
    source: NodeId,
    weight: Option<WeightFn<'a, E>>,
}

impl<'a, E> BellmanFord<'a, E> {
    /// Creates a Bellman-Ford search rooted at `source`.
    pub fn new(source: NodeId) -> Self {
        Self { source, weight: None }
    }

    /// Sets the edge-weight accessor (may return negative values).
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }
}

pub(crate) fn bellman_ford_impl<G, N, E>(
    graph: G,
    solver: &BellmanFord<'_, E>,
) -> Result<ShortestPathSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    if !graph.contains_node(solver.source) {
        return Err(CnvxError::ForeignHandle);
    }
    let Some(weight) = &solver.weight else {
        return Err(CnvxError::Numerical(
            "BellmanFord: no weight accessor set; call .weight(...)".to_string(),
        ));
    };

    let nodes: Vec<NodeId> = graph.node_ids().collect();
    let n = nodes.iter().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut distance = vec![f64::INFINITY; n];
    let mut predecessor = PredecessorMap::new(n);
    distance[solver.source.index()] = 0.0;

    let mut directed_edges = Vec::new();
    for &u in &nodes {
        for e in graph.out_edges(u) {
            if let Some(v) = graph.other_endpoint(e, u) {
                directed_edges.push((u, v, e));
            }
        }
    }

    let mut improved_on_last_pass;
    for pass in 0..nodes.len() {
        improved_on_last_pass = false;
        for &(u, v, e) in &directed_edges {
            if !distance[u.index()].is_finite() {
                continue;
            }
            let w = weight(graph.edge_weight(e).expect("edge from out_edges is valid"));
            let candidate = distance[u.index()] + w;
            if candidate < distance[v.index()] {
                distance[v.index()] = candidate;
                predecessor.set(v, (u, e));
                improved_on_last_pass = true;
                if pass == nodes.len() - 1 {
                    return Ok(ShortestPathSolution {
                        status: Status::Infeasible,
                        source: solver.source,
                        distance,
                        predecessor,
                        negative_cycle: true,
                    });
                }
            }
        }
        if !improved_on_last_pass {
            break;
        }
    }

    Ok(ShortestPathSolution {
        status: Status::Optimal,
        source: solver.source,
        distance,
        predecessor,
        negative_cycle: false,
    })
}

impl_solve!(BellmanFord<'_, E> => ShortestPathSolution, bellman_ford_impl);
