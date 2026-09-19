use cnvx_core::{CnvxError, Status};

use crate::common::dispatch::impl_solve;
use crate::common::union_find::UnionFind;
use crate::graph::GraphRef;
use crate::{EdgeId, NodeId};

/// Connected components of an undirected graph, or weakly
/// connected components of a directed graph.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::connectivity::ConnectedComponents;
///
/// let mut g = Graph::<(), ()>::undirected();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, ()).unwrap();
///
/// let result = (&g).solve(&ConnectedComponents).unwrap();
/// assert_eq!(result.component_of(a), result.component_of(b));
/// assert_ne!(result.component_of(a), result.component_of(c));
/// assert_eq!(result.count(), 2);
/// ```
pub struct ConnectedComponents;

/// The result of [`ConnectedComponents`] (and, identically shaped, of
/// [`WeaklyConnectedComponents`]).
pub struct ComponentsSolution {
    status: Status,
    component: Vec<u32>,
    count: u32,
}

impl ComponentsSolution {
    /// Always [`Status::Optimal`].
    pub fn status(&self) -> Status {
        self.status
    }

    /// The component index (`0..count()`) containing `node`. Two nodes
    /// are in the same component iff this is equal.
    pub fn component_of(&self, node: NodeId) -> u32 {
        self.component[node.index()]
    }

    /// The number of components.
    pub fn count(&self) -> u32 {
        self.count
    }
}

fn connected_components_impl<G, N, E>(
    graph: G,
    _solver: &ConnectedComponents,
) -> Result<ComponentsSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut uf = UnionFind::new(n);
    for e in graph.edge_ids() {
        if let Some((a, b)) = graph.endpoints(e) {
            uf.union(a.index(), b.index());
        }
    }
    let mut labels = vec![u32::MAX; n];
    let mut next_label = 0u32;
    let mut component = vec![0u32; n];
    for id in graph.node_ids() {
        let root = uf.find(id.index());
        let label = if labels[root] == u32::MAX {
            let l = next_label;
            labels[root] = l;
            next_label += 1;
            l
        } else {
            labels[root]
        };
        component[id.index()] = label;
    }
    Ok(ComponentsSolution {
        status: Status::Optimal,
        component,
        count: next_label,
    })
}

impl_solve!(ConnectedComponents => ComponentsSolution, connected_components_impl);

/// Weakly connected components of a directed graph: identical to
/// [`ConnectedComponents`], provided as a separately named type purely so
/// call sites read as "I know this graph is directed and I'm explicitly
/// ignoring direction" rather than looking like a mistake.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::connectivity::WeaklyConnectedComponents;
///
/// let mut g = Graph::<(), ()>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// g.add_edge(a, b, ()).unwrap();
///
/// let result = (&g).solve(&WeaklyConnectedComponents).unwrap();
/// assert_eq!(result.component_of(a), result.component_of(b));
/// ```
pub struct WeaklyConnectedComponents;

fn weakly_connected_components_impl<G, N, E>(
    graph: G,
    _solver: &WeaklyConnectedComponents,
) -> Result<ComponentsSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    connected_components_impl(graph, &ConnectedComponents)
}

impl_solve!(WeaklyConnectedComponents => ComponentsSolution, weakly_connected_components_impl);

/// Strongly connected components of a directed graph: maximal sets of
/// nodes each reachable from every other node in the same set. Uses
/// Tarjan's algorithm (a single DFS pass, `O(V + E)`).
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::connectivity::StronglyConnectedComponents;
///
/// let mut g = Graph::<(), ()>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, ()).unwrap();
/// g.add_edge(b, a, ()).unwrap();
/// g.add_edge(b, c, ()).unwrap();
///
/// let result = (&g).solve(&StronglyConnectedComponents).unwrap();
/// assert_eq!(result.component_of(a), result.component_of(b));
/// assert_ne!(result.component_of(a), result.component_of(c));
/// ```
pub struct StronglyConnectedComponents;

fn strongly_connected_components_impl<G, N, E>(
    graph: G,
    _solver: &StronglyConnectedComponents,
) -> Result<ComponentsSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let ids: Vec<NodeId> = graph.node_ids().collect();
    let n = ids.len();
    let index_of = |id: NodeId| ids.iter().position(|&x| x == id).unwrap();

    let mut index = vec![None; n];
    let mut lowlink = vec![0u32; n];
    let mut on_stack = vec![false; n];
    let mut stack = Vec::new();
    let mut next_index = 0u32;
    let mut component = vec![u32::MAX; n];
    let mut next_component = 0u32;

    enum Frame {
        Enter(usize),
        Exit(usize),
    }
    let mut call_stack = Vec::new();
    let mut child_cursor: Vec<usize> = vec![0; n];

    for start in 0..n {
        if index[start].is_some() {
            continue;
        }
        call_stack.push(Frame::Enter(start));
        while let Some(frame) = call_stack.pop() {
            match frame {
                Frame::Enter(v) => {
                    if index[v].is_some() {
                        continue;
                    }
                    index[v] = Some(next_index);
                    lowlink[v] = next_index;
                    next_index += 1;
                    stack.push(v);
                    on_stack[v] = true;
                    call_stack.push(Frame::Exit(v));
                    let neighbors: Vec<usize> = graph
                        .out_edges(ids[v])
                        .filter_map(|e| graph.other_endpoint(e, ids[v]))
                        .map(index_of)
                        .collect();
                    while child_cursor[v] < neighbors.len() {
                        let w = neighbors[child_cursor[v]];
                        child_cursor[v] += 1;
                        if index[w].is_none() {
                            call_stack.push(Frame::Enter(w));
                            break;
                        } else if on_stack[w] {
                            lowlink[v] = lowlink[v].min(index[w].unwrap());
                        }
                    }
                    if child_cursor[v] < neighbors.len() {
                        call_stack.push(Frame::Enter(v));
                    }
                }
                Frame::Exit(v) => {
                    let neighbors: Vec<usize> = graph
                        .out_edges(ids[v])
                        .filter_map(|e| graph.other_endpoint(e, ids[v]))
                        .map(index_of)
                        .collect();
                    for w in neighbors {
                        if on_stack[w] {
                            lowlink[v] = lowlink[v].min(lowlink[w]);
                        }
                    }
                    if lowlink[v] == index[v].unwrap() {
                        loop {
                            let w = stack.pop().unwrap();
                            on_stack[w] = false;
                            component[w] = next_component;
                            if w == v {
                                break;
                            }
                        }
                        next_component += 1;
                    }
                }
            }
        }
    }

    Ok(ComponentsSolution {
        status: Status::Optimal,
        component,
        count: next_component,
    })
}

impl_solve!(StronglyConnectedComponents => ComponentsSolution, strongly_connected_components_impl);

/// Articulation points (cut vertices) of an undirected graph: nodes whose
/// removal increases the number of connected components. Computed via a
/// single DFS pass tracking discovery time and lowpoint, `O(V + E)`.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::connectivity::ArticulationPoints;
///
/// // a - b - c  (b is a cut vertex; removing it disconnects a from c)
/// let mut g = Graph::<(), ()>::undirected();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, ()).unwrap();
/// g.add_edge(b, c, ()).unwrap();
///
/// let result = (&g).solve(&ArticulationPoints).unwrap();
/// assert!(result.is_articulation_point(b));
/// assert!(!result.is_articulation_point(a));
/// ```
pub struct ArticulationPoints;

/// The result of [`ArticulationPoints`].
pub struct ArticulationPointsSolution {
    status: Status,
    is_cut: Vec<bool>,
}

impl ArticulationPointsSolution {
    /// Always [`Status::Optimal`].
    pub fn status(&self) -> Status {
        self.status
    }

    /// `true` if `node` is an articulation point.
    pub fn is_articulation_point(&self, node: NodeId) -> bool {
        self.is_cut.get(node.index()).copied().unwrap_or(false)
    }
}

/// Bridges (cut edges) of an undirected graph: edges whose removal
/// increases the number of connected components. Computed with the same
/// DFS pass as [`ArticulationPoints`].
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::connectivity::Bridges;
///
/// let mut g = Graph::<(), ()>::undirected();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let e = g.add_edge(a, b, ()).unwrap();
///
/// let result = (&g).solve(&Bridges).unwrap();
/// assert!(result.is_bridge(e));
/// ```
pub struct Bridges;

/// The result of [`Bridges`].
pub struct BridgesSolution {
    status: Status,
    bridge: Vec<bool>,
}

impl BridgesSolution {
    /// Always [`Status::Optimal`].
    pub fn status(&self) -> Status {
        self.status
    }

    /// `true` if `edge` is a bridge.
    pub fn is_bridge(&self, edge: EdgeId) -> bool {
        self.bridge.get(edge.index()).copied().unwrap_or(false)
    }
}

/// Shared low-link DFS state for [`ArticulationPoints`] and [`Bridges`]
/// (both are computed by the same DFS, differing only in what is recorded
/// at each step, so both are driven by one traversal rather than
/// duplicating it).
struct LowLinkDfs {
    is_cut: Vec<bool>,
    bridge: Vec<bool>,
}

fn low_link_dfs<G, N, E>(graph: G, n: usize) -> LowLinkDfs
where
    G: GraphRef<N, E> + Copy,
{
    let ids: Vec<NodeId> = graph.node_ids().collect();
    let index_of = |id: NodeId| ids.iter().position(|&x| x == id).unwrap();
    let mut discovery: Vec<Option<u32>> = vec![None; n];
    let mut low = vec![0u32; n];
    let mut is_cut = vec![false; n];
    let edge_count = graph.edge_ids().map(|e| e.index() + 1).max().unwrap_or(0);
    let mut bridge_edges = vec![false; edge_count];
    let mut timer = 0u32;

    struct StackFrame {
        v: usize,
        parent_edge: Option<usize>,
        is_root: bool,
        root_children: u32,
        edges: Vec<(usize, usize)>, // (neighbor index, edge index)
        cursor: usize,
    }

    for start in 0..n {
        if discovery[start].is_some() {
            continue;
        }
        let edges: Vec<(usize, usize)> = graph
            .out_edges(ids[start])
            .filter_map(|e| {
                graph.other_endpoint(e, ids[start]).map(|v| (index_of(v), e.index()))
            })
            .collect();
        discovery[start] = Some(timer);
        low[start] = timer;
        timer += 1;
        let mut stack = vec![StackFrame {
            v: start,
            parent_edge: None,
            is_root: true,
            root_children: 0,
            edges,
            cursor: 0,
        }];

        while let Some(frame) = stack.last_mut() {
            if frame.cursor >= frame.edges.len() {
                let v = frame.v;
                let parent_edge = frame.parent_edge;
                let v_low = low[v];
                stack.pop();
                if let Some(parent_frame) = stack.last_mut() {
                    let p = parent_frame.v;
                    low[p] = low[p].min(v_low);
                    if v_low > discovery[p].unwrap()
                        && let Some(pe) = parent_edge
                    {
                        bridge_edges[pe] = true;
                    }
                    if parent_frame.is_root {
                        parent_frame.root_children += 1;
                        if parent_frame.root_children > 1 {
                            is_cut[p] = true;
                        }
                    } else if v_low >= discovery[p].unwrap() {
                        is_cut[p] = true;
                    }
                }
                continue;
            }
            let (w, edge_index) = frame.edges[frame.cursor];
            frame.cursor += 1;
            if Some(edge_index) == frame.parent_edge {
                continue;
            }
            if let Some(dw) = discovery[w] {
                low[frame.v] = low[frame.v].min(dw);
                continue;
            }
            discovery[w] = Some(timer);
            low[w] = timer;
            timer += 1;
            let child_edges: Vec<(usize, usize)> = graph
                .out_edges(ids[w])
                .filter_map(|e| {
                    graph.other_endpoint(e, ids[w]).map(|v| (index_of(v), e.index()))
                })
                .collect();
            stack.push(StackFrame {
                v: w,
                parent_edge: Some(edge_index),
                is_root: false,
                root_children: 0,
                edges: child_edges,
                cursor: 0,
            });
        }
    }

    LowLinkDfs { is_cut, bridge: bridge_edges }
}

fn articulation_points_impl<G, N, E>(
    graph: G,
    _solver: &ArticulationPoints,
) -> Result<ArticulationPointsSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let dfs = low_link_dfs(graph, n);
    Ok(ArticulationPointsSolution { status: Status::Optimal, is_cut: dfs.is_cut })
}

impl_solve!(ArticulationPoints => ArticulationPointsSolution, articulation_points_impl);

fn bridges_impl<G, N, E>(
    graph: G,
    _solver: &Bridges,
) -> Result<BridgesSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let dfs = low_link_dfs(graph, n);
    Ok(BridgesSolution { status: Status::Optimal, bridge: dfs.bridge })
}

impl_solve!(Bridges => BridgesSolution, bridges_impl);
