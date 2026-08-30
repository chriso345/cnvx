use cnvx_core::{CnvxError, Status};

use crate::common::dispatch::impl_solve;
use crate::graph::GraphRef;
use crate::weight::WeightFn;
use crate::{EdgeId, NodeId};

struct Arc {
    to: usize,
    cap: f64,
    cost: f64,
    original_edge: Option<u32>,
}

/// Minimum-cost flow from `source` to `sink`.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::flow::MinCostFlow;
///
/// struct Pipe {
///     capacity: f64,
///     cost: f64,
/// }
///
/// let mut g = Graph::<(), Pipe>::directed();
/// let s = g.add_node(());
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let t = g.add_node(());
/// g.add_edge(s, a, Pipe { capacity: 2.0, cost: 1.0 }).unwrap();
/// g.add_edge(s, b, Pipe { capacity: 2.0, cost: 5.0 }).unwrap();
/// g.add_edge(a, t, Pipe { capacity: 2.0, cost: 1.0 }).unwrap();
/// g.add_edge(b, t, Pipe { capacity: 2.0, cost: 1.0 }).unwrap();
///
/// let result = (&g)
///     .solve(
///         &MinCostFlow::new(s, t)
///             .capacity(|p: &Pipe| p.capacity)
///             .cost(|p: &Pipe| p.cost)
///             .target_flow(2.0),
///     )
///     .unwrap();
/// // Cheapest way to send 2 units: both through s -> a -> t (cost 1 each).
/// assert_eq!(result.objective(), 4.0);
/// ```
pub struct MinCostFlow<'a, E> {
    source: NodeId,
    sink: NodeId,
    capacity: Option<WeightFn<'a, E>>,
    cost: Option<WeightFn<'a, E>>,
    target_flow: Option<f64>,
}

impl<'a, E> MinCostFlow<'a, E> {
    /// Creates a min-cost flow solver from `source` to `sink`.
    pub fn new(source: NodeId, sink: NodeId) -> Self {
        Self {
            source,
            sink,
            capacity: None,
            cost: None,
            target_flow: None,
        }
    }

    /// Sets the edge-capacity accessor (must be non-negative).
    pub fn capacity(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.capacity = Some(Box::new(f));
        self
    }

    /// Sets the edge-cost accessor.
    pub fn cost(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.cost = Some(Box::new(f));
        self
    }

    /// Requires exactly this much flow, rather than the maximum possible.
    pub fn target_flow(mut self, target: f64) -> Self {
        self.target_flow = Some(target);
        self
    }
}

/// The result of [`MinCostFlow`].
pub struct MinCostFlowSolution {
    status: Status,
    objective: f64,
    flow_value: f64,
    flow_per_edge: Vec<f64>,
}

impl MinCostFlowSolution {
    /// [`Status::Optimal`] if the requested flow (maximum, or
    /// [`MinCostFlow::target_flow`] if set) was achieved,
    /// [`Status::Infeasible`] if a specific target flow was requested but
    /// could not be reached.
    pub fn status(&self) -> Status {
        self.status
    }

    /// The total cost of the flow found.
    pub fn objective(&self) -> f64 {
        self.objective
    }

    /// The total amount of flow sent from source to sink.
    pub fn flow_value(&self) -> f64 {
        self.flow_value
    }

    /// The flow carried on `edge`.
    pub fn flow(&self, edge: EdgeId) -> f64 {
        self.flow_per_edge.get(edge.index()).copied().unwrap_or(0.0)
    }
}

fn min_cost_flow_impl<G, N, E>(
    graph: G,
    solver: &MinCostFlow<'_, E>,
) -> Result<MinCostFlowSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    if !graph.contains_node(solver.source) || !graph.contains_node(solver.sink) {
        return Err(CnvxError::ForeignHandle);
    }
    let Some(capacity) = &solver.capacity else {
        return Err(CnvxError::Numerical(
            "MinCostFlow: no capacity accessor set; call .capacity(...)".to_string(),
        ));
    };
    let Some(cost) = &solver.cost else {
        return Err(CnvxError::Numerical(
            "MinCostFlow: no cost accessor set; call .cost(...)".to_string(),
        ));
    };
    if solver.source == solver.sink {
        return Err(CnvxError::Numerical(
            "MinCostFlow: source and sink must differ".to_string(),
        ));
    }

    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut arcs: Vec<Arc> = Vec::new();
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];
    let add_arc = |arcs: &mut Vec<Arc>,
                   adjacency: &mut Vec<Vec<usize>>,
                   from: usize,
                   to: usize,
                   cap: f64,
                   cost: f64,
                   original_edge: Option<u32>| {
        let fwd = arcs.len();
        arcs.push(Arc { to, cap, cost, original_edge });
        adjacency[from].push(fwd);
        let bwd = arcs.len();
        arcs.push(Arc {
            to: from,
            cap: 0.0,
            cost: -cost,
            original_edge: None,
        });
        adjacency[to].push(bwd);
    };
    for e in graph.edge_ids() {
        let Some((a, b)) = graph.endpoints(e) else { continue };
        let Some(d) = graph.edge_weight(e) else { continue };
        let (c, cst) = (capacity(d), cost(d));
        add_arc(
            &mut arcs,
            &mut adjacency,
            a.index(),
            b.index(),
            c,
            cst,
            Some(e.index() as u32),
        );
        if !graph.is_directed() {
            add_arc(
                &mut arcs,
                &mut adjacency,
                b.index(),
                a.index(),
                c,
                cst,
                Some(e.index() as u32),
            );
        }
    }

    let s = solver.source.index();
    let t = solver.sink.index();
    let mut total_cost = 0.0;
    let mut total_flow = 0.0;
    let target = solver.target_flow;

    loop {
        if let Some(target) = target
            && total_flow >= target - 1e-12
        {
            break;
        }
        // Bellman-Ford shortest path by cost (tolerates the negative
        // costs on reverse residual arcs).
        let mut dist = vec![f64::INFINITY; n];
        let mut in_queue = vec![false; n];
        let mut pred_arc: Vec<Option<usize>> = vec![None; n];
        dist[s] = 0.0;
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(s);
        in_queue[s] = true;
        while let Some(u) = queue.pop_front() {
            in_queue[u] = false;
            for &ai in &adjacency[u] {
                let arc = &arcs[ai];
                if arc.cap > 1e-12 && dist[u] + arc.cost < dist[arc.to] - 1e-12 {
                    let new_dist = dist[u] + arc.cost;
                    let v = arc.to;
                    // (index recomputed after the mutable borrow above ends)
                    if new_dist < dist[v] {
                        dist[v] = new_dist;
                        pred_arc[v] = Some(ai);
                        if !in_queue[v] {
                            in_queue[v] = true;
                            queue.push_back(v);
                        }
                    }
                }
            }
        }
        if !dist[t].is_finite() {
            break;
        }
        // Bottleneck capacity along the found path.
        let mut bottleneck = f64::INFINITY;
        let mut v = t;
        while v != s {
            let ai = pred_arc[v].expect("path exists");
            bottleneck = bottleneck.min(arcs[ai].cap);
            v = arcs[ai ^ 1].to;
        }
        if let Some(target) = target {
            bottleneck = bottleneck.min(target - total_flow);
        }
        if bottleneck <= 1e-12 {
            break;
        }
        let mut v = t;
        while v != s {
            let ai = pred_arc[v].expect("path exists");
            arcs[ai].cap -= bottleneck;
            arcs[ai ^ 1].cap += bottleneck;
            v = arcs[ai ^ 1].to;
        }
        total_cost += bottleneck * dist[t];
        total_flow += bottleneck;
    }

    let status = match target {
        Some(target) if (total_flow - target).abs() > 1e-9 => Status::Infeasible,
        _ => Status::Optimal,
    };

    let mut flow_per_edge =
        vec![0.0; graph.edge_ids().map(|e| e.index() + 1).max().unwrap_or(0)];
    let mut seen: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();
    for chunk in arcs.chunks(2) {
        if let Some(edge_index) = chunk[0].original_edge {
            *seen.entry(edge_index).or_insert(0.0) += chunk[1].cap;
        }
    }
    for (edge_index, used) in seen {
        if (edge_index as usize) < flow_per_edge.len() {
            flow_per_edge[edge_index as usize] = used;
        }
    }

    Ok(MinCostFlowSolution {
        status,
        objective: total_cost,
        flow_value: total_flow,
        flow_per_edge,
    })
}

impl_solve!(MinCostFlow<'_, E> => MinCostFlowSolution, min_cost_flow_impl);
