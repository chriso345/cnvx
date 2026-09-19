use cnvx_core::{CnvxError, Status};

use super::solution::ShortestPathSolution;
use crate::NodeId;
use crate::common::distance_map::PredecessorMap;
use crate::common::priority_queue::PriorityQueue;
use crate::graph::GraphRef;
use crate::heuristic::Heuristic;
use crate::weight::WeightFn;

/// A*: identical to Dijkstra except the priority queue orders by
/// `distance_so_far + heuristic.estimate(node)`.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::heuristic::ZeroHeuristic;
/// use cnvx_graph::shortest_path::AStar;
///
/// let mut g = Graph::<(), f64>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, 1.0).unwrap();
/// g.add_edge(b, c, 2.0).unwrap();
///
/// let result = (&g)
///     .solve(&AStar::new(a, c, ZeroHeuristic).weight(|w: &f64| *w))
///     .unwrap();
/// assert_eq!(result.distance_to(c), Some(3.0));
/// ```
pub struct AStar<'a, E, H> {
    source: NodeId,
    target: NodeId,
    heuristic: H,
    weight: Option<WeightFn<'a, E>>,
}

impl<'a, E, H: Heuristic> AStar<'a, E, H> {
    /// Creates an A* search from `source` to `target` guided by
    /// `heuristic`. A weight accessor must be supplied via
    /// [`Self::weight`] before solving.
    pub fn new(source: NodeId, target: NodeId, heuristic: H) -> Self {
        Self { source, target, heuristic, weight: None }
    }

    /// Sets the edge-weight accessor.
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }
}

fn astar_impl<G, N, E, H>(
    graph: G,
    solver: &AStar<'_, E, H>,
) -> Result<ShortestPathSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
    H: Heuristic,
{
    if !graph.contains_node(solver.source) || !graph.contains_node(solver.target) {
        return Err(CnvxError::ForeignHandle);
    }
    let Some(weight) = &solver.weight else {
        return Err(CnvxError::Numerical(
            "AStar: no weight accessor set; call .weight(...)".to_string(),
        ));
    };

    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut distance = vec![f64::INFINITY; n];
    let mut predecessor = PredecessorMap::new(n);
    let mut finalized = vec![false; n];
    let mut queue = PriorityQueue::new();

    distance[solver.source.index()] = 0.0;
    queue.push(solver.source, solver.heuristic.estimate(solver.source));

    while let Some((u, _priority)) = queue.pop() {
        if finalized[u.index()] {
            continue;
        }
        finalized[u.index()] = true;
        if u == solver.target {
            break;
        }
        let du = distance[u.index()];
        for e in graph.out_edges(u) {
            let Some(v) = graph.other_endpoint(e, u) else { continue };
            let Some(w) = graph.edge_weight(e).map(weight) else { continue };
            if w < 0.0 {
                return Err(CnvxError::Numerical(format!(
                    "AStar requires non-negative edge weights; found {w} on an edge out of node {}",
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
                queue.push(v, candidate + solver.heuristic.estimate(v));
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

impl<N, E, H: Heuristic> cnvx_core::Solve<AStar<'_, E, H>> for &crate::Graph<N, E> {
    type Solution = ShortestPathSolution;

    fn solve(&self, solver: &AStar<'_, E, H>) -> Result<ShortestPathSolution, CnvxError> {
        astar_impl(*self, solver)
    }
}
impl<N, E, H: Heuristic> cnvx_core::Solve<AStar<'_, E, H>>
    for &crate::GraphView<'_, N, E>
{
    type Solution = ShortestPathSolution;

    fn solve(&self, solver: &AStar<'_, E, H>) -> Result<ShortestPathSolution, CnvxError> {
        astar_impl(*self, solver)
    }
}
