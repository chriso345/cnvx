use cnvx_core::{CnvxError, Status};

use crate::NodeId;
use crate::common::dispatch::impl_solve;
use crate::graph::GraphRef;

/// Breadth-first search from a single source.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::traversal::Bfs;
///
/// let mut g = Graph::<&str, ()>::directed();
/// let a = g.add_node("a");
/// let b = g.add_node("b");
/// let c = g.add_node("c");
/// g.add_edge(a, b, ()).unwrap();
/// g.add_edge(b, c, ()).unwrap();
///
/// let result = (&g).solve(&Bfs::new(a)).unwrap();
/// assert_eq!(result.depth(c), Some(2));
/// assert_eq!(result.order(), &[a, b, c]);
/// ```
pub struct Bfs {
    source: NodeId,
}

impl Bfs {
    /// Creates a BFS search rooted at `source`.
    pub fn new(source: NodeId) -> Self {
        Self { source }
    }
}

/// The result of a [`Bfs`] search: each reached node's BFS depth (edge
/// count from the source) and discovery order.
pub struct BfsSolution {
    status: Status,
    depth: Vec<Option<u32>>,
    order: Vec<NodeId>,
    source: NodeId,
}

impl BfsSolution {
    /// Always [`Status::Optimal`], as BFS is guaranteed to complete
    /// successfully on any graph.
    pub fn status(&self) -> Status {
        self.status
    }

    /// The BFS depth (number of edges) from the source to `node`, or
    /// `None` if `node` is unreached.
    pub fn depth(&self, node: NodeId) -> Option<u32> {
        self.depth.get(node.index()).copied().flatten()
    }

    /// `true` if `node` was reached.
    pub fn is_reachable(&self, node: NodeId) -> bool {
        self.depth(node).is_some()
    }

    /// Every reached node, in BFS discovery order (the source first).
    pub fn order(&self) -> &[NodeId] {
        &self.order
    }

    /// The source node this search was rooted at.
    pub fn source(&self) -> NodeId {
        self.source
    }
}

pub(crate) fn bfs_impl<G, N, E>(graph: G, solver: &Bfs) -> Result<BfsSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    if !graph.contains_node(solver.source) {
        return Err(CnvxError::ForeignHandle);
    }
    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut depth: Vec<Option<u32>> = vec![None; n];
    let mut order = Vec::new();
    let mut queue = std::collections::VecDeque::new();

    depth[solver.source.index()] = Some(0);
    queue.push_back(solver.source);
    while let Some(u) = queue.pop_front() {
        order.push(u);
        let d = depth[u.index()].unwrap();
        for v in graph.neighbors(u) {
            if depth[v.index()].is_none() {
                depth[v.index()] = Some(d + 1);
                queue.push_back(v);
            }
        }
    }

    Ok(BfsSolution {
        status: Status::Optimal,
        depth,
        order,
        source: solver.source,
    })
}

impl_solve!(Bfs => BfsSolution, bfs_impl);

/// Depth-first search from a single source.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::traversal::Dfs;
///
/// let mut g = Graph::<&str, ()>::directed();
/// let a = g.add_node("a");
/// let b = g.add_node("b");
/// g.add_edge(a, b, ()).unwrap();
///
/// let result = (&g).solve(&Dfs::new(a)).unwrap();
/// assert!(result.is_reachable(b));
/// ```
pub struct Dfs {
    source: NodeId,
}

impl Dfs {
    /// Creates a DFS search rooted at `source`.
    pub fn new(source: NodeId) -> Self {
        Self { source }
    }
}

/// The result of a [`Dfs`] search.
pub struct DfsSolution {
    status: Status,
    order: Vec<NodeId>,
    reachable: Vec<bool>,
}

impl DfsSolution {
    /// Always [`Status::Optimal`].
    pub fn status(&self) -> Status {
        self.status
    }

    /// `true` if `node` was reached.
    pub fn is_reachable(&self, node: NodeId) -> bool {
        self.reachable.get(node.index()).copied().unwrap_or(false)
    }

    /// Every reached node, in DFS discovery (preorder) order.
    pub fn order(&self) -> &[NodeId] {
        &self.order
    }
}

fn dfs_impl<G, N, E>(graph: G, solver: &Dfs) -> Result<DfsSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    if !graph.contains_node(solver.source) {
        return Err(CnvxError::ForeignHandle);
    }
    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut reachable = vec![false; n];
    let mut order = Vec::new();
    // Explicit stack, iterative, to avoid unbounded recursion depth on a
    // long path/cycle-free chain.
    let mut stack = vec![solver.source];
    reachable[solver.source.index()] = true;
    while let Some(u) = stack.pop() {
        order.push(u);
        for v in graph.neighbors(u) {
            if !reachable[v.index()] {
                reachable[v.index()] = true;
                stack.push(v);
            }
        }
    }
    Ok(DfsSolution { status: Status::Optimal, order, reachable })
}

impl_solve!(Dfs => DfsSolution, dfs_impl);
