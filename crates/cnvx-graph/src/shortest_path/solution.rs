use cnvx_core::Status;

use crate::common::distance_map::PredecessorMap;
use crate::common::path_reconstruction::reconstruct_edge_path;
use crate::{EdgeId, NodeId};

/// The result of a single-source shortest-path search.
pub struct ShortestPathSolution {
    pub(super) status: Status,
    pub(super) source: NodeId,
    pub(super) distance: Vec<f64>,
    pub(super) predecessor: PredecessorMap,
    pub(super) negative_cycle: bool,
}

impl ShortestPathSolution {
    /// [`Status::Optimal`] normally. [`Status::Infeasible`] only for
    /// Bellman-Ford when a negative-weight cycle reachable from the
    /// source was detected.
    pub fn status(&self) -> Status {
        self.status
    }

    /// The source node this search was rooted at.
    pub fn source(&self) -> NodeId {
        self.source
    }

    /// `true` if Bellman-Ford detected a negative-weight cycle reachable
    /// from the source.
    pub fn has_negative_cycle(&self) -> bool {
        self.negative_cycle
    }

    /// The shortest-path distance from the source to `node`, or `None` if
    /// `node` is unreachable (or invalid).
    pub fn distance_to(&self, node: NodeId) -> Option<f64> {
        let d = *self.distance.get(node.index())?;
        if d.is_finite() { Some(d) } else { None }
    }

    /// The sequence of edges on a shortest path from the source to `node`,
    /// or `None` if `node` is unreachable. Empty if `node` is the source.
    pub fn path_to(&self, node: NodeId) -> Option<Vec<EdgeId>> {
        self.distance_to(node)?;
        reconstruct_edge_path(&self.predecessor, self.source, node)
    }

    /// The sequence of nodes on a shortest path from the source to `node`
    /// (source first, `node` last), or `None` if `node` is unreachable.
    pub fn node_path_to<N, E>(
        &self,
        graph: &crate::Graph<N, E>,
        node: NodeId,
    ) -> Option<Vec<NodeId>> {
        let edges = self.path_to(node)?;
        let mut nodes = vec![self.source];
        let mut current = self.source;
        for e in edges {
            let next = graph.other_endpoint(e, current)?;
            nodes.push(next);
            current = next;
        }
        Some(nodes)
    }
}

impl std::ops::Index<NodeId> for ShortestPathSolution {
    type Output = f64;

    fn index(&self, node: NodeId) -> &f64 {
        // `distance` already stores +INF for unreached nodes; `Index`
        // just skips the `Option` wrapping `distance_to` adds.
        &self.distance[node.index()]
    }
}
