use cnvx_core::CnvxError;

use crate::ids::{EdgeId, NodeId};

static NEXT_GENERATION: std::sync::atomic::AtomicU32 =
    std::sync::atomic::AtomicU32::new(1);

pub(crate) fn next_generation() -> u32 {
    NEXT_GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub(crate) struct EdgeRecord<E> {
    pub(crate) source: u32,
    pub(crate) target: u32,
    pub(crate) data: E,
}

/// A directed or undirected graph with arbitrary node data `N` and edge
/// data `E`.
pub struct Graph<N, E> {
    pub(crate) generation: u32,
    pub(crate) directed: bool,
    pub(crate) nodes: Vec<N>,
    /// Outgoing (directed) or all-incident (undirected) edge ids, per node.
    pub(crate) out_adj: Vec<Vec<u32>>,
    /// Incoming edge ids, per node. Only populated for directed graphs;
    /// stays empty for undirected graphs, whose `out_adj` already holds
    /// every incident edge in both directions.
    pub(crate) in_adj: Vec<Vec<u32>>,
    pub(crate) edges: Vec<EdgeRecord<E>>,
}

impl<N, E> Graph<N, E> {
    /// Creates an empty directed graph.
    pub fn directed() -> Self {
        Self {
            generation: next_generation(),
            directed: true,
            nodes: Vec::new(),
            out_adj: Vec::new(),
            in_adj: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Creates an empty undirected graph.
    pub fn undirected() -> Self {
        Self {
            generation: next_generation(),
            directed: false,
            nodes: Vec::new(),
            out_adj: Vec::new(),
            in_adj: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// `true` if this graph is directed.
    pub fn is_directed(&self) -> bool {
        self.directed
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn check_node(&self, id: NodeId) -> std::result::Result<(), CnvxError> {
        if id.generation != self.generation || id.index() >= self.nodes.len() {
            return Err(CnvxError::ForeignHandle);
        }
        Ok(())
    }

    fn check_edge(&self, id: EdgeId) -> std::result::Result<(), CnvxError> {
        if id.generation != self.generation || id.index() >= self.edges.len() {
            return Err(CnvxError::ForeignHandle);
        }
        Ok(())
    }

    /// Adds a node carrying `data`, returning its handle.
    pub fn add_node(&mut self, data: N) -> NodeId {
        let index = self.nodes.len() as u32;
        self.nodes.push(data);
        self.out_adj.push(Vec::new());
        if self.directed {
            self.in_adj.push(Vec::new());
        }
        NodeId { index, generation: self.generation }
    }

    /// Adds an edge from `source` to `target` carrying `data`, returning
    /// its handle.
    ///
    /// Both endpoints must already exist. Parallel edges and self-loops
    /// (`source == target`) are permitted.
    ///
    /// # Errors
    /// Returns `Err(GraphError::ForeignHandle)` (via
    /// `cnvx_core::CnvxError::ForeignHandle`) if either endpoint is not a
    /// valid handle for this graph.
    pub fn add_edge(
        &mut self,
        source: NodeId,
        target: NodeId,
        data: E,
    ) -> std::result::Result<EdgeId, CnvxError> {
        self.check_node(source)?;
        self.check_node(target)?;
        let index = self.edges.len() as u32;
        self.edges
            .push(EdgeRecord { source: source.index, target: target.index, data });
        self.out_adj[source.index()].push(index);
        if self.directed {
            self.in_adj[target.index()].push(index);
        } else if source.index() != target.index() {
            self.out_adj[target.index()].push(index);
        } else {
            // Undirected self-loop: convention is that it contributes 2
            // to the node's degree, so it is recorded twice in the same
            // adjacency list rather than once.
            self.out_adj[source.index()].push(index);
        }
        Ok(EdgeId { index, generation: self.generation })
    }

    /// Shared access to a node's data.
    pub fn node_weight(&self, id: NodeId) -> Option<&N> {
        if self.check_node(id).is_err() {
            return None;
        }
        self.nodes.get(id.index())
    }

    /// Exclusive access to a node's data.
    pub fn node_weight_mut(&mut self, id: NodeId) -> Option<&mut N> {
        if self.check_node(id).is_err() {
            return None;
        }
        self.nodes.get_mut(id.index())
    }

    /// Shared access to an edge's data.
    pub fn edge_weight(&self, id: EdgeId) -> Option<&E> {
        if self.check_edge(id).is_err() {
            return None;
        }
        self.edges.get(id.index()).map(|e| &e.data)
    }

    /// Exclusive access to an edge's data.
    pub fn edge_weight_mut(&mut self, id: EdgeId) -> Option<&mut E> {
        if self.check_edge(id).is_err() {
            return None;
        }
        self.edges.get_mut(id.index()).map(|e| &mut e.data)
    }

    /// The `(source, target)` endpoints of an edge, in the order they were
    /// added (arbitrary, for an undirected edge).
    pub fn endpoints(&self, id: EdgeId) -> Option<(NodeId, NodeId)> {
        if self.check_edge(id).is_err() {
            return None;
        }
        let rec = &self.edges[id.index()];
        Some((
            NodeId { index: rec.source, generation: self.generation },
            NodeId { index: rec.target, generation: self.generation },
        ))
    }

    /// Given one endpoint of `edge`, returns the other. Returns `None` if
    /// `from` is not actually an endpoint of `edge` (or either handle is
    /// invalid). For a self-loop, returns `from` itself.
    pub fn other_endpoint(&self, edge: EdgeId, from: NodeId) -> Option<NodeId> {
        let (a, b) = self.endpoints(edge)?;
        if a == from {
            Some(b)
        } else if b == from {
            Some(a)
        } else {
            None
        }
    }

    /// Every node handle, in insertion order.
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        let generation = self.generation;
        (0..self.nodes.len() as u32).map(move |index| NodeId { index, generation })
    }

    /// Every edge handle, in insertion order.
    pub fn edge_ids(&self) -> impl Iterator<Item = EdgeId> + '_ {
        let generation = self.generation;
        (0..self.edges.len() as u32).map(move |index| EdgeId { index, generation })
    }

    /// `(NodeId, &N)` pairs for every node, in insertion order.
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &N)> + '_ {
        self.node_ids().map(move |id| (id, &self.nodes[id.index()]))
    }

    /// `(EdgeId, &E)` pairs for every edge, in insertion order.
    pub fn edges(&self) -> impl Iterator<Item = (EdgeId, &E)> + '_ {
        self.edge_ids().map(move |id| (id, &self.edges[id.index()].data))
    }

    /// Outgoing edges of `id` (directed graphs), or all incident edges
    /// (undirected graphs).
    pub fn out_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_ {
        let generation = self.generation;
        let slice: &[u32] =
            if self.check_node(id).is_ok() { &self.out_adj[id.index()] } else { &[] };
        slice.iter().map(move |&index| EdgeId { index, generation })
    }

    /// Incoming edges of `id` (directed graphs), or all incident edges
    /// (undirected graphs, identical to [`Graph::out_edges`]).
    pub fn in_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_ {
        let generation = self.generation;
        let slice: &[u32] = if self.check_node(id).is_err() {
            &[]
        } else if self.directed {
            &self.in_adj[id.index()]
        } else {
            &self.out_adj[id.index()]
        };
        slice.iter().map(move |&index| EdgeId { index, generation })
    }

    /// Neighbors reachable via an outgoing edge (directed graphs), or any
    /// incident edge (undirected graphs). A self-loop yields `id` itself.
    pub fn neighbors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.out_edges(id).filter_map(move |e| self.other_endpoint(e, id))
    }

    /// Total degree: `out_degree + in_degree` for a directed graph
    /// (double-counting a self-loop, as is conventional), or the number of
    /// incident edge-ends for an undirected graph.
    pub fn degree(&self, id: NodeId) -> usize {
        if self.directed {
            self.out_degree(id) + self.in_degree(id)
        } else {
            self.out_adj.get(id.index()).map_or(0, Vec::len)
        }
    }

    /// Number of outgoing edges (directed), or incident edges
    /// (undirected - identical to [`Graph::degree`]).
    pub fn out_degree(&self, id: NodeId) -> usize {
        self.out_adj.get(id.index()).map_or(0, Vec::len)
    }

    /// Number of incoming edges (directed), or incident edges
    /// (undirected - identical to [`Graph::degree`]).
    pub fn in_degree(&self, id: NodeId) -> usize {
        if self.directed {
            self.in_adj.get(id.index()).map_or(0, Vec::len)
        } else {
            self.out_adj.get(id.index()).map_or(0, Vec::len)
        }
    }

    /// `true` if `id` is a valid handle for this graph.
    pub fn contains_node(&self, id: NodeId) -> bool {
        self.check_node(id).is_ok()
    }

    /// `true` if `id` is a valid handle for this graph.
    pub fn contains_edge(&self, id: EdgeId) -> bool {
        self.check_edge(id).is_ok()
    }

    /// Borrows the whole graph as a [`GraphView`] with no filtering
    /// applied - the identity view, useful as a starting point for
    /// [`GraphView::filter_nodes`]/[`GraphView::filter_edges`].
    pub fn view(&self) -> GraphView<'_, N, E> {
        GraphView { graph: self, node_filter: None, edge_filter: None }
    }
}

mod sealed {
    pub trait Sealed {}
}

/// The read-only query surface every algorithm in this crate is written
/// against, implemented by `&Graph<N, E>` and by [`GraphView`].
///
/// Algorithms are generic over `G: GraphRef<N, E>` rather than taking a
/// concrete `&Graph` directly.
pub trait GraphRef<N, E>: sealed::Sealed {
    /// `true` if the underlying graph is directed.
    fn is_directed(&self) -> bool;
    /// Number of nodes visible through this view.
    fn node_count(&self) -> usize;
    /// Number of edges visible through this view.
    fn edge_count(&self) -> usize;
    /// `true` if `id` is a valid, visible node handle.
    fn contains_node(&self, id: NodeId) -> bool;
    /// Shared access to a node's data, if `id` is valid and visible.
    fn node_weight(&self, id: NodeId) -> Option<&N>;
    /// Shared access to an edge's data, if `id` is valid and visible.
    fn edge_weight(&self, id: EdgeId) -> Option<&E>;
    /// The endpoints of `id`, if valid and visible.
    fn endpoints(&self, id: EdgeId) -> Option<(NodeId, NodeId)>;
    /// Given one endpoint, the other. See [`Graph::other_endpoint`].
    fn other_endpoint(&self, edge: EdgeId, from: NodeId) -> Option<NodeId>;
    /// Every visible node handle.
    fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_;
    /// Every visible edge handle.
    fn edge_ids(&self) -> impl Iterator<Item = EdgeId> + '_;
    /// Outgoing (directed) or incident (undirected) edges of `id`, filtered
    /// to those visible through this view.
    fn out_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_;
    /// Incoming (directed) or incident (undirected) edges of `id`, filtered
    /// to those visible through this view.
    fn in_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_;
    /// Neighbors of `id` reachable via a visible outgoing edge.
    fn neighbors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_;
}

impl<N, E> sealed::Sealed for &Graph<N, E> {}

impl<N, E> GraphRef<N, E> for &Graph<N, E> {
    fn is_directed(&self) -> bool {
        Graph::is_directed(self)
    }

    fn node_count(&self) -> usize {
        Graph::node_count(self)
    }

    fn edge_count(&self) -> usize {
        Graph::edge_count(self)
    }

    fn contains_node(&self, id: NodeId) -> bool {
        Graph::contains_node(self, id)
    }

    fn node_weight(&self, id: NodeId) -> Option<&N> {
        Graph::node_weight(self, id)
    }

    fn edge_weight(&self, id: EdgeId) -> Option<&E> {
        Graph::edge_weight(self, id)
    }

    fn endpoints(&self, id: EdgeId) -> Option<(NodeId, NodeId)> {
        Graph::endpoints(self, id)
    }

    fn other_endpoint(&self, edge: EdgeId, from: NodeId) -> Option<NodeId> {
        Graph::other_endpoint(self, edge, from)
    }

    fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        Graph::node_ids(self)
    }

    fn edge_ids(&self) -> impl Iterator<Item = EdgeId> + '_ {
        Graph::edge_ids(self)
    }

    fn out_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_ {
        Graph::out_edges(self, id)
    }

    fn in_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_ {
        Graph::in_edges(self, id)
    }

    fn neighbors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        Graph::neighbors(self, id)
    }
}

/// A predicate that determines whether a node is visible in a [`GraphView`].
type NodeFilter<'g, N> = Box<dyn Fn(NodeId, &N) -> bool + 'g>;
/// A predicate that determines whether an edge is visible in a [`GraphView`].
type EdgeFilter<'g, E> = Box<dyn Fn(EdgeId, &E) -> bool + 'g>;

/// A cheap, borrowed, filtered view over a [`Graph`].
pub struct GraphView<'g, N, E> {
    pub(crate) graph: &'g Graph<N, E>,
    pub(crate) node_filter: Option<NodeFilter<'g, N>>,
    pub(crate) edge_filter: Option<EdgeFilter<'g, E>>,
}

impl<'g, N, E> GraphView<'g, N, E> {
    fn node_visible(&self, id: NodeId) -> bool {
        let Some(data) = self.graph.node_weight(id) else { return false };
        match &self.node_filter {
            Some(f) => f(id, data),
            None => true,
        }
    }

    fn edge_visible(&self, id: EdgeId) -> bool {
        let Some(data) = self.graph.edge_weight(id) else { return false };
        let Some((a, b)) = self.graph.endpoints(id) else { return false };
        if !self.node_visible(a) || !self.node_visible(b) {
            return false;
        }
        match &self.edge_filter {
            Some(f) => f(id, data),
            None => true,
        }
    }

    /// Returns a new view, over the same graph, that additionally requires
    /// nodes to satisfy `predicate`. Composes with any node filter already
    /// present (both must pass).
    pub fn filter_nodes(self, predicate: impl Fn(NodeId, &N) -> bool + 'g) -> Self {
        let previous = self.node_filter;
        let combined: NodeFilter<'g, N> = match previous {
            Some(prev) => Box::new(move |id, data| prev(id, data) && predicate(id, data)),
            None => Box::new(predicate),
        };
        GraphView {
            graph: self.graph,
            node_filter: Some(combined),
            edge_filter: self.edge_filter,
        }
    }

    /// Returns a new view, over the same graph, that additionally requires
    /// edges to satisfy `predicate` (on top of the endpoint-visibility
    /// requirement every view already applies). Composes with any edge
    /// filter already present.
    pub fn filter_edges(self, predicate: impl Fn(EdgeId, &E) -> bool + 'g) -> Self {
        let previous = self.edge_filter;
        let combined: EdgeFilter<'g, E> = match previous {
            Some(prev) => Box::new(move |id, data| prev(id, data) && predicate(id, data)),
            None => Box::new(predicate),
        };
        GraphView {
            graph: self.graph,
            node_filter: self.node_filter,
            edge_filter: Some(combined),
        }
    }
}

impl<N, E> sealed::Sealed for &GraphView<'_, N, E> {}

impl<N, E> GraphRef<N, E> for &GraphView<'_, N, E> {
    fn is_directed(&self) -> bool {
        self.graph.is_directed()
    }

    fn node_count(&self) -> usize {
        self.node_ids().count()
    }

    fn edge_count(&self) -> usize {
        self.edge_ids().count()
    }

    fn contains_node(&self, id: NodeId) -> bool {
        self.node_visible(id)
    }

    fn node_weight(&self, id: NodeId) -> Option<&N> {
        if self.node_visible(id) { self.graph.node_weight(id) } else { None }
    }

    fn edge_weight(&self, id: EdgeId) -> Option<&E> {
        if self.edge_visible(id) { self.graph.edge_weight(id) } else { None }
    }

    fn endpoints(&self, id: EdgeId) -> Option<(NodeId, NodeId)> {
        if self.edge_visible(id) { self.graph.endpoints(id) } else { None }
    }

    fn other_endpoint(&self, edge: EdgeId, from: NodeId) -> Option<NodeId> {
        if self.edge_visible(edge) { self.graph.other_endpoint(edge, from) } else { None }
    }

    fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.graph.node_ids().filter(move |&id| self.node_visible(id))
    }

    fn edge_ids(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.graph.edge_ids().filter(move |&id| self.edge_visible(id))
    }

    fn out_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_ {
        self.graph.out_edges(id).filter(move |&e| self.edge_visible(e))
    }

    fn in_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_ {
        self.graph.in_edges(id).filter(move |&e| self.edge_visible(e))
    }

    fn neighbors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.out_edges(id)
            .filter_map(move |e| self.graph.other_endpoint(e, id))
    }
}
