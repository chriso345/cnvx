use cnvx_core::{CnvxError, Status};

use super::solution::ShortestPathSolution;
use crate::NodeId;
use crate::common::dispatch::impl_solve;
use crate::common::distance_map::PredecessorMap;
use crate::common::priority_queue::PriorityQueue;
use crate::graph::GraphRef;
use crate::weight::WeightFn;

/// Dijkstra's algorithm: a *label-setting* method (each node's distance is
/// finalized exactly once, when popped from the priority queue, and never
/// revisited) that requires non-negative edge weights.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::shortest_path::Dijkstra;
///
/// let mut g = Graph::<(), f64>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, 1.0).unwrap();
/// g.add_edge(b, c, 2.0).unwrap();
/// g.add_edge(a, c, 10.0).unwrap();
///
/// let result = (&g).solve(&Dijkstra::new(a).weight(|w: &f64| *w)).unwrap();
/// assert_eq!(result.distance_to(c), Some(3.0));
/// ```
pub struct Dijkstra<'a, E> {
    source: NodeId,
    target: Option<NodeId>,
    weight: Option<WeightFn<'a, E>>,
}

impl<'a, E> Dijkstra<'a, E> {
    /// Creates a Dijkstra search rooted at `source`. A weight accessor
    /// must be supplied via [`Self::weight`] before solving.
    pub fn new(source: NodeId) -> Self {
        Self { source, target: None, weight: None }
    }

    /// Sets the edge-weight accessor, projecting each edge's data down to
    /// a non-negative `f64` cost.
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }

    /// Stops the search as soon as `target`'s distance is finalized,
    /// rather than continuing until every reachable node is finalized.
    pub fn target(mut self, target: NodeId) -> Self {
        self.target = Some(target);
        self
    }
}

pub(crate) fn dijkstra_impl<G, N, E>(
    graph: G,
    solver: &Dijkstra<'_, E>,
) -> Result<ShortestPathSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    if !graph.contains_node(solver.source) {
        return Err(CnvxError::ForeignHandle);
    }
    let Some(weight) = &solver.weight else {
        return Err(CnvxError::Numerical(
            "Dijkstra: no weight accessor set; call .weight(...)".to_string(),
        ));
    };

    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut distance = vec![f64::INFINITY; n];
    let mut predecessor = PredecessorMap::new(n);
    let mut finalized = vec![false; n];
    let mut queue = PriorityQueue::new();

    distance[solver.source.index()] = 0.0;
    queue.push(solver.source, 0.0);

    while let Some((u, du)) = queue.pop() {
        if finalized[u.index()] {
            continue;
        }
        finalized[u.index()] = true;
        if Some(u) == solver.target {
            break;
        }
        for e in graph.out_edges(u) {
            let Some(v) = graph.other_endpoint(e, u) else { continue };
            let Some(w) = graph.edge_weight(e).map(weight) else { continue };
            if w < 0.0 {
                return Err(CnvxError::Numerical(format!(
                    "Dijkstra requires non-negative edge weights; found {w} on an edge out of node {}",
                    u.index()
                )));
            }
            if finalized[v.index()] {
                continue;
            }
            let candidate = du + w;
            if candidate < distance[v.index()] {
                distance[v.index()] = candidate;
                predecessor.set(v, (u, e));
                queue.push(v, candidate);
            }
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

impl_solve!(Dijkstra<'_, E> => ShortestPathSolution, dijkstra_impl);
