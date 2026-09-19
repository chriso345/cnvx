use cnvx_core::{CnvxError, Status};

use crate::common::augmenting_path::bfs_layers;
use crate::common::dispatch::impl_solve;
use crate::graph::GraphRef;
use crate::weight::WeightFn;
use crate::{EdgeId, NodeId};

/// A residual arc: `to` is the arc's head, `cap` its remaining capacity,
/// and `original_edge` the real edge it represents (`None` for a reverse
/// pseudo-arc, which exists only to let flow be "undone").
struct Arc {
    to: usize,
    cap: f64,
    original_edge: Option<u32>,
}

struct ResidualGraph {
    arcs: Vec<Arc>,
    adjacency: Vec<Vec<usize>>, // per node, indices into `arcs`
}

impl ResidualGraph {
    fn add_arc(&mut self, from: usize, to: usize, cap: f64, original_edge: Option<u32>) {
        let forward = self.arcs.len();
        self.arcs.push(Arc { to, cap, original_edge });
        self.adjacency[from].push(forward);
        let backward = self.arcs.len();
        self.arcs.push(Arc { to: from, cap: 0.0, original_edge: None });
        self.adjacency[to].push(backward);
    }
}

fn build_residual<G, N, E>(
    graph: G,
    capacity: &WeightFn<'_, E>,
    n: usize,
) -> ResidualGraph
where
    G: GraphRef<N, E> + Copy,
{
    let mut residual = ResidualGraph { arcs: Vec::new(), adjacency: vec![Vec::new(); n] };
    for e in graph.edge_ids() {
        let Some((a, b)) = graph.endpoints(e) else { continue };
        let Some(c) = graph.edge_weight(e).map(capacity) else { continue };
        residual.add_arc(a.index(), b.index(), c, Some(e.index() as u32));
        if !graph.is_directed() {
            residual.add_arc(b.index(), a.index(), c, Some(e.index() as u32));
        }
    }
    residual
}

/// Maximum flow from `source` to `sink`, and the corresponding minimum
/// cut.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::flow::MaxFlow;
///
/// let mut g = Graph::<(), f64>::directed();
/// let s = g.add_node(());
/// let a = g.add_node(());
/// let t = g.add_node(());
/// g.add_edge(s, a, 5.0).unwrap();
/// g.add_edge(a, t, 3.0).unwrap();
///
/// let result = (&g).solve(&MaxFlow::new(s, t).capacity(|c: &f64| *c)).unwrap();
/// assert_eq!(result.objective(), 3.0);
/// ```
pub struct MaxFlow<'a, E> {
    source: NodeId,
    sink: NodeId,
    capacity: Option<WeightFn<'a, E>>,
}

impl<'a, E> MaxFlow<'a, E> {
    /// Creates a max-flow solver from `source` to `sink`.
    pub fn new(source: NodeId, sink: NodeId) -> Self {
        Self { source, sink, capacity: None }
    }

    /// Sets the edge-capacity accessor (must be non-negative).
    pub fn capacity(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.capacity = Some(Box::new(f));
        self
    }
}

/// The result of [`MaxFlow`].
pub struct MaxFlowSolution {
    status: Status,
    objective: f64,
    flow_per_edge: Vec<f64>,
    source_side: Vec<bool>,
}

impl MaxFlowSolution {
    /// Always [`Status::Optimal`].
    pub fn status(&self) -> Status {
        self.status
    }

    /// The maximum flow value.
    pub fn objective(&self) -> f64 {
        self.objective
    }

    /// The flow carried on `edge` (in its stored direction; `0` if it
    /// carries none).
    pub fn flow(&self, edge: EdgeId) -> f64 {
        self.flow_per_edge.get(edge.index()).copied().unwrap_or(0.0)
    }

    /// `true` if `node` is on the source side of the minimum cut (i.e.
    /// still reachable from the source in the final residual graph).
    pub fn is_source_side(&self, node: NodeId) -> bool {
        self.source_side.get(node.index()).copied().unwrap_or(false)
    }
}

fn max_flow_impl<G, N, E>(
    graph: G,
    solver: &MaxFlow<'_, E>,
) -> Result<MaxFlowSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    if !graph.contains_node(solver.source) || !graph.contains_node(solver.sink) {
        return Err(CnvxError::ForeignHandle);
    }
    let Some(capacity) = &solver.capacity else {
        return Err(CnvxError::Numerical(
            "MaxFlow: no capacity accessor set; call .capacity(...)".to_string(),
        ));
    };
    if solver.source == solver.sink {
        return Err(CnvxError::Numerical(
            "MaxFlow: source and sink must differ".to_string(),
        ));
    }

    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut residual = build_residual(graph, capacity, n);
    let s = solver.source.index();
    let t = solver.sink.index();
    let mut total_flow = 0.0;

    loop {
        let level = bfs_layers(n, &[s], |u, out| {
            for &ai in &residual.adjacency[u] {
                if residual.arcs[ai].cap > 1e-12 {
                    out.push(residual.arcs[ai].to);
                }
            }
        });
        if level[t] == -1 {
            break;
        }
        let mut cursor = vec![0usize; n];
        loop {
            let pushed = dfs_blocking_flow(
                &mut residual,
                &level,
                &mut cursor,
                s,
                t,
                f64::INFINITY,
            );
            if pushed <= 1e-12 {
                break;
            }
            total_flow += pushed;
        }
    }

    let mut flow_per_edge =
        vec![0.0; graph.edge_ids().map(|e| e.index() + 1).max().unwrap_or(0)];

    let mut seen_forward: std::collections::HashMap<u32, f64> =
        std::collections::HashMap::new();
    for chunk in residual.arcs.chunks(2) {
        let arc = &chunk[0];
        if let Some(edge_index) = arc.original_edge {
            // `cap` has been decremented by usage; original capacity is
            // recovered from the paired reverse arc's final cap (which
            // started at 0 and now holds exactly the used amount).
            let used = chunk[1].cap;
            *seen_forward.entry(edge_index).or_insert(0.0) += used;
        }
    }
    for (edge_index, used) in seen_forward {
        if (edge_index as usize) < flow_per_edge.len() {
            flow_per_edge[edge_index as usize] = used;
        }
    }

    let reach = bfs_layers(n, &[s], |u, out| {
        for &ai in &residual.adjacency[u] {
            if residual.arcs[ai].cap > 1e-12 {
                out.push(residual.arcs[ai].to);
            }
        }
    });
    let source_side: Vec<bool> = reach.iter().map(|&l| l != -1).collect();

    Ok(MaxFlowSolution {
        status: Status::Optimal,
        objective: total_flow,
        flow_per_edge,
        source_side,
    })
}

fn dfs_blocking_flow(
    residual: &mut ResidualGraph,
    level: &[i32],
    cursor: &mut [usize],
    u: usize,
    t: usize,
    limit: f64,
) -> f64 {
    if u == t {
        return limit;
    }
    while cursor[u] < residual.adjacency[u].len() {
        let ai = residual.adjacency[u][cursor[u]];
        let v = residual.arcs[ai].to;
        if residual.arcs[ai].cap > 1e-12 && level[v] == level[u] + 1 {
            let pushed = dfs_blocking_flow(
                residual,
                level,
                cursor,
                v,
                t,
                limit.min(residual.arcs[ai].cap),
            );
            if pushed > 1e-12 {
                residual.arcs[ai].cap -= pushed;
                let rev = ai ^ 1;
                residual.arcs[rev].cap += pushed;
                return pushed;
            }
        }
        cursor[u] += 1;
    }
    0.0
}

impl_solve!(MaxFlow<'_, E> => MaxFlowSolution, max_flow_impl);

#[cfg(test)]
mod tests {
    use cnvx_core::Solve;

    use super::*;
    use crate::Graph;

    #[test]
    fn max_flow_equals_min_cut_capacity() {
        let mut g = Graph::<(), f64>::directed();
        let s = g.add_node(());
        let a = g.add_node(());
        let b = g.add_node(());
        let t = g.add_node(());
        let _e1 = g.add_edge(s, a, 10.0).unwrap();
        g.add_edge(s, b, 5.0).unwrap();
        g.add_edge(a, b, 15.0).unwrap();
        let _e4 = g.add_edge(a, t, 10.0).unwrap();
        let e5 = g.add_edge(b, t, 10.0).unwrap();

        let result = (&g).solve(&MaxFlow::new(s, t).capacity(|c: &f64| *c)).unwrap();
        assert_eq!(result.objective(), 15.0);

        // Cut capacity: sum of capacities of edges crossing from the
        // source side to the sink side should equal the max flow value
        // (max-flow-min-cut theorem).
        let mut cut_capacity = 0.0;
        for e in g.edge_ids() {
            let (u, v) = g.endpoints(e).unwrap();
            if result.is_source_side(u) && !result.is_source_side(v) {
                cut_capacity += *g.edge_weight(e).unwrap();
            }
        }
        assert_eq!(cut_capacity, 15.0);
        let _ = e5;
    }
}
