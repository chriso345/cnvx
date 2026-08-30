use cnvx_core::CnvxError;

use crate::NodeId;
use crate::graph::GraphRef;

/// An admissible (never overestimating), ideally consistent, lower-bound
/// estimate of the remaining distance from any node to a fixed goal.
///
/// ```
/// use cnvx_graph::NodeId;
/// use cnvx_graph::heuristic::Heuristic;
///
/// fn straight_line_estimate(
///     goal_x: f64,
///     goal_y: f64,
///     positions: Vec<(f64, f64)>,
/// ) -> impl Heuristic {
///     move |node: NodeId| {
///         let (x, y) = positions[node.index()];
///         ((x - goal_x).powi(2) + (y - goal_y).powi(2)).sqrt()
///     }
/// }
/// ```
pub trait Heuristic {
    /// Estimates the remaining distance from `node` to the (implicit)
    /// goal.
    fn estimate(&self, node: NodeId) -> f64;
}

impl<F: Fn(NodeId) -> f64> Heuristic for F {
    fn estimate(&self, node: NodeId) -> f64 {
        self(node)
    }
}

/// The zero heuristic: estimates every remaining distance as `0`. Turns
/// A* into plain Dijkstra.
pub struct ZeroHeuristic;

impl Heuristic for ZeroHeuristic {
    fn estimate(&self, _node: NodeId) -> f64 {
        0.0
    }
}

/// The ALT (A*, Landmarks, Triangle inequality) heuristic: precomputes
/// exact shortest-path distances from a small set of *landmark* nodes to
/// every other node, then lower-bounds the distance from any node `v` to
/// a goal `t` as `max over landmarks L of |d(L, t) - d(L, v)|`, which
/// follows directly from the triangle inequality and is admissible for
/// any goal `t` without recomputation.
pub struct LandmarkHeuristic {
    /// `distance_from_landmark[i][v.index()]` = shortest-path distance
    /// from landmark `i` to node `v`.
    distance_from_landmark: Vec<Vec<f64>>,
    goal: NodeId,
}

impl LandmarkHeuristic {
    /// Precomputes distances from each of `landmarks` to every node in
    /// `graph`, for later use estimating distance to `goal`.
    ///
    /// `weight` should be the same edge-weight accessor the A* search
    /// itself will use, so the precomputed distances are consistent with
    /// the search's own notion of edge cost.
    pub fn precompute<G, N, E>(
        graph: G,
        landmarks: &[NodeId],
        weight: impl Fn(&E) -> f64 + Copy,
        goal: NodeId,
    ) -> std::result::Result<Self, CnvxError>
    where
        G: GraphRef<N, E> + Copy,
    {
        let mut distance_from_landmark = Vec::with_capacity(landmarks.len());
        for &landmark in landmarks {
            let solution = crate::shortest_path::dijkstra_impl(
                graph,
                &crate::shortest_path::Dijkstra::new(landmark).weight(weight),
            )
            .map_err(|_| {
                CnvxError::InvalidArgument(
                    "landmark heuristic precomputation failed (negative weight?)"
                        .to_string(),
                )
            })?;
            let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
            let mut row = vec![f64::INFINITY; n];
            for id in graph.node_ids() {
                if let Some(d) = solution.distance_to(id) {
                    row[id.index()] = d;
                }
            }
            distance_from_landmark.push(row);
        }
        Ok(Self { distance_from_landmark, goal })
    }
}

impl Heuristic for LandmarkHeuristic {
    fn estimate(&self, node: NodeId) -> f64 {
        self.distance_from_landmark
            .iter()
            .map(|row| {
                let to_goal = row[self.goal.index()];
                let to_node = row[node.index()];
                if to_goal.is_finite() && to_node.is_finite() {
                    (to_goal - to_node).abs()
                } else {
                    0.0
                }
            })
            .fold(0.0, f64::max)
    }
}
