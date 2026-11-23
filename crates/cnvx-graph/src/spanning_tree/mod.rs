use cnvx_core::{CnvxError, Status};

use crate::EdgeId;
use crate::common::dispatch::impl_solve;
use crate::common::union_find::UnionFind;
use crate::graph::GraphRef;
use crate::weight::WeightFn;

/// The result of [`Kruskal`], [`Prim`], or [`Boruvka`]: the set of edges
/// forming a minimum spanning forest, and its total weight.
pub struct SpanningForestSolution {
    status: Status,
    objective: f64,
    edges: Vec<EdgeId>,
}

impl SpanningForestSolution {
    /// Always [`Status::Optimal`].
    pub fn status(&self) -> Status {
        self.status
    }

    /// The total weight of the spanning forest.
    pub fn objective(&self) -> f64 {
        self.objective
    }

    /// The edges making up the spanning forest.
    pub fn edges(&self) -> &[EdgeId] {
        &self.edges
    }
}

/// Kruskal's algorithm: sort all edges by weight, then add each in
/// increasing order unless it would close a cycle (checked via
/// union-find). `O(E log E)`.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::spanning_tree::Kruskal;
///
/// let mut g = Graph::<(), f64>::undirected();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, 1.0).unwrap();
/// g.add_edge(b, c, 2.0).unwrap();
/// g.add_edge(a, c, 10.0).unwrap();
///
/// let result = (&g).solve(&Kruskal::new().weight(|w: &f64| *w)).unwrap();
/// assert_eq!(result.objective(), 3.0);
/// assert_eq!(result.edges().len(), 2);
/// ```
pub struct Kruskal<'a, E> {
    weight: Option<WeightFn<'a, E>>,
}

impl<'a, E> Default for Kruskal<'a, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E> Kruskal<'a, E> {
    /// Creates a Kruskal solver. A weight accessor must be supplied via
    /// [`Self::weight`] before solving.
    pub fn new() -> Self {
        Self { weight: None }
    }

    /// Sets the edge-weight accessor.
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }
}

fn kruskal_impl<G, N, E>(
    graph: G,
    solver: &Kruskal<'_, E>,
) -> Result<SpanningForestSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let Some(weight) = &solver.weight else {
        return Err(CnvxError::Numerical(
            "Kruskal: no weight accessor set; call .weight(...)".to_string(),
        ));
    };
    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut edges: Vec<(EdgeId, f64)> = graph
        .edge_ids()
        .filter_map(|e| graph.edge_weight(e).map(|d| (e, weight(d))))
        .collect();
    edges.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("weight must not be NaN"));

    let mut uf = UnionFind::new(n);
    let mut chosen = Vec::new();
    let mut objective = 0.0;
    for (e, w) in edges {
        let Some((a, b)) = graph.endpoints(e) else { continue };
        if uf.union(a.index(), b.index()) {
            chosen.push(e);
            objective += w;
        }
    }
    Ok(SpanningForestSolution { status: Status::Optimal, objective, edges: chosen })
}

impl_solve!(Kruskal<'_, E> => SpanningForestSolution, kruskal_impl);

/// Prim's algorithm: grow a single tree by repeatedly adding the cheapest
/// edge leaving it, using a priority queue (`O(E log V)`); repeated from a
/// new unvisited node whenever the current tree is exhausted, to cover a
/// disconnected graph as a forest.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::spanning_tree::Prim;
///
/// let mut g = Graph::<(), f64>::undirected();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, 1.0).unwrap();
/// g.add_edge(b, c, 2.0).unwrap();
/// g.add_edge(a, c, 10.0).unwrap();
///
/// let result = (&g).solve(&Prim::new().weight(|w: &f64| *w)).unwrap();
/// assert_eq!(result.objective(), 3.0);
/// ```
pub struct Prim<'a, E> {
    weight: Option<WeightFn<'a, E>>,
}

impl<'a, E> Default for Prim<'a, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E> Prim<'a, E> {
    /// Creates a Prim solver.
    pub fn new() -> Self {
        Self { weight: None }
    }

    /// Sets the edge-weight accessor.
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }
}

fn prim_impl<G, N, E>(
    graph: G,
    solver: &Prim<'_, E>,
) -> Result<SpanningForestSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let Some(weight) = &solver.weight else {
        return Err(CnvxError::Numerical(
            "Prim: no weight accessor set; call .weight(...)".to_string(),
        ));
    };
    let nodes: Vec<_> = graph.node_ids().collect();
    let n = nodes.len();
    let mut in_tree = vec![false; n];
    let mut chosen = Vec::new();
    let mut objective = 0.0;

    for &start in &nodes {
        if in_tree[start.index()] {
            continue;
        }
        in_tree[start.index()] = true;
        let mut heap = crate::common::priority_queue::PriorityQueue::new();
        for e in graph.out_edges(start) {
            if let (Some(v), Some(d)) =
                (graph.other_endpoint(e, start), graph.edge_weight(e))
            {
                heap.push((e, v), weight(d));
            }
        }
        while let Some(((e, v), w)) = heap.pop() {
            if in_tree[v.index()] {
                continue;
            }
            in_tree[v.index()] = true;
            chosen.push(e);
            objective += w;
            for e2 in graph.out_edges(v) {
                if let (Some(v2), Some(d)) =
                    (graph.other_endpoint(e2, v), graph.edge_weight(e2))
                    && !in_tree[v2.index()]
                {
                    heap.push((e2, v2), weight(d));
                }
            }
        }
    }
    Ok(SpanningForestSolution { status: Status::Optimal, objective, edges: chosen })
}

impl_solve!(Prim<'_, E> => SpanningForestSolution, prim_impl);

/// Boruvka's algorithm: in rounds, every component simultaneously picks
/// its cheapest outgoing edge, and all such edges are added at once
/// (merging components via union-find); repeats until one component
/// remains per connected piece of the graph. `O(E log V)`, in
/// `O(log V)` rounds.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::spanning_tree::Boruvka;
///
/// let mut g = Graph::<(), f64>::undirected();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, 1.0).unwrap();
/// g.add_edge(b, c, 2.0).unwrap();
/// g.add_edge(a, c, 10.0).unwrap();
///
/// let result = (&g).solve(&Boruvka::new().weight(|w: &f64| *w)).unwrap();
/// assert_eq!(result.objective(), 3.0);
/// ```
pub struct Boruvka<'a, E> {
    weight: Option<WeightFn<'a, E>>,
}

impl<'a, E> Default for Boruvka<'a, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E> Boruvka<'a, E> {
    /// Creates a Boruvka solver.
    pub fn new() -> Self {
        Self { weight: None }
    }

    /// Sets the edge-weight accessor.
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }
}

fn boruvka_impl<G, N, E>(
    graph: G,
    solver: &Boruvka<'_, E>,
) -> Result<SpanningForestSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let Some(weight) = &solver.weight else {
        return Err(CnvxError::Numerical(
            "Boruvka: no weight accessor set; call .weight(...)".to_string(),
        ));
    };
    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut uf = UnionFind::new(n);
    let mut chosen = Vec::new();
    let mut objective = 0.0;
    let all_edges: Vec<(EdgeId, usize, usize, f64)> = graph
        .edge_ids()
        .filter_map(|e| {
            let (a, b) = graph.endpoints(e)?;
            let d = graph.edge_weight(e)?;
            Some((e, a.index(), b.index(), weight(d)))
        })
        .collect();

    loop {
        let mut cheapest: Vec<Option<(EdgeId, f64)>> = vec![None; n];
        for &(e, a, b, w) in &all_edges {
            let ra = uf.find(a);
            let rb = uf.find(b);
            if ra == rb {
                continue;
            }
            for r in [ra, rb] {
                let better = match cheapest[r] {
                    Some((_, cw)) => w < cw,
                    None => true,
                };
                if better {
                    cheapest[r] = Some((e, w));
                }
            }
        }
        let mut merged_any = false;
        for slot in cheapest.into_iter().flatten() {
            let (e, w) = slot;
            let Some((a, b)) = graph.endpoints(e) else { continue };
            if uf.union(a.index(), b.index()) {
                chosen.push(e);
                objective += w;
                merged_any = true;
            }
        }
        if !merged_any {
            break;
        }
    }

    Ok(SpanningForestSolution { status: Status::Optimal, objective, edges: chosen })
}

impl_solve!(Boruvka<'_, E> => SpanningForestSolution, boruvka_impl);
