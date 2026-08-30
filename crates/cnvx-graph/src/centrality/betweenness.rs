use cnvx_core::{CnvxError, Status};

use super::CentralityScores;
use crate::NodeId;
use crate::common::dispatch::impl_solve;
use crate::common::priority_queue::PriorityQueue;
use crate::graph::GraphRef;
use crate::weight::WeightFn;

/// Betweenness centrality.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::centrality::Betweenness;
///
/// // a - b - c: every shortest path between a and c passes through b.
/// let mut g = Graph::<(), ()>::undirected();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, ()).unwrap();
/// g.add_edge(b, c, ()).unwrap();
///
/// let result = (&g).solve(&Betweenness::new()).unwrap();
/// assert!(result.score(b) > result.score(a));
/// assert_eq!(result.score(a), 0.0);
/// ```
pub struct Betweenness<'a, E> {
    weight: Option<WeightFn<'a, E>>,
    normalized: bool,
}

impl<'a, E> Default for Betweenness<'a, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E> Betweenness<'a, E> {
    /// Creates an unweighted betweenness solver.
    pub fn new() -> Self {
        Self { weight: None, normalized: false }
    }

    /// Uses Dijkstra with this weight accessor instead of unweighted BFS.
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }

    /// Normalizes scores by `(n-1)(n-2)` (directed) or `(n-1)(n-2)/2`
    /// (undirected).
    pub fn normalized(mut self) -> Self {
        self.normalized = true;
        self
    }
}

fn betweenness_impl<G, N, E>(
    graph: G,
    solver: &Betweenness<'_, E>,
) -> Result<CentralityScores, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let ids: Vec<NodeId> = graph.node_ids().collect();
    let n = ids.len();
    let index_of = |id: NodeId| ids.iter().position(|&x| x == id).unwrap();
    let mut betweenness = vec![0.0; n];

    for &s in &ids {
        let si = index_of(s);
        // `predecessors[w]`: every node on some shortest path from `s` to
        // `w`'s immediate predecessor set; `sigma[w]`: number of shortest
        // paths from `s` to `w`; `order`: nodes in non-decreasing distance
        // from `s` (so the dependency-accumulation pass can run in
        // reverse of this order).
        let mut sigma = vec![0.0f64; n];
        let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut order = Vec::with_capacity(n);
        sigma[si] = 1.0;

        if let Some(weight) = &solver.weight {
            let mut dist = vec![f64::INFINITY; n];
            dist[si] = 0.0;
            let mut finalized = vec![false; n];
            let mut queue = PriorityQueue::new();
            queue.push(si, 0.0);
            while let Some((u, du)) = queue.pop() {
                if finalized[u] {
                    continue;
                }
                finalized[u] = true;
                order.push(u);
                for e in graph.out_edges(ids[u]) {
                    let Some(v) = graph.other_endpoint(e, ids[u]) else { continue };
                    let vi = index_of(v);
                    if finalized[vi] {
                        continue;
                    }
                    let Some(w) = graph.edge_weight(e).map(weight) else {
                        continue;
                    };
                    let candidate = du + w;
                    if candidate < dist[vi] {
                        dist[vi] = candidate;
                        sigma[vi] = sigma[u];
                        predecessors[vi] = vec![u];
                        queue.push(vi, candidate);
                    } else if (candidate - dist[vi]).abs() < 1e-12 {
                        sigma[vi] += sigma[u];
                        predecessors[vi].push(u);
                    }
                }
            }
        } else {
            let mut dist: Vec<Option<u32>> = vec![None; n];
            dist[si] = Some(0);
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(si);
            while let Some(u) = queue.pop_front() {
                order.push(u);
                let du = dist[u].unwrap();
                for e in graph.out_edges(ids[u]) {
                    let Some(v) = graph.other_endpoint(e, ids[u]) else { continue };
                    let vi = index_of(v);
                    match dist[vi] {
                        None => {
                            dist[vi] = Some(du + 1);
                            sigma[vi] = sigma[u];
                            predecessors[vi] = vec![u];
                            queue.push_back(vi);
                        }
                        Some(d) if d == du + 1 => {
                            sigma[vi] += sigma[u];
                            predecessors[vi].push(u);
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut delta = vec![0.0f64; n];
        for &w in order.iter().rev() {
            for &v in &predecessors[w] {
                if sigma[w] > 0.0 {
                    delta[v] += (sigma[v] / sigma[w]) * (1.0 + delta[w]);
                }
            }
            if w != si {
                betweenness[w] += delta[w];
            }
        }
    }

    if !graph.is_directed() {
        for b in &mut betweenness {
            *b /= 2.0;
        }
    }
    if solver.normalized && n > 2 {
        let denom = if graph.is_directed() {
            (n - 1) as f64 * (n - 2) as f64
        } else {
            (n - 1) as f64 * (n - 2) as f64 / 2.0
        };
        for b in &mut betweenness {
            *b /= denom;
        }
    }

    Ok(CentralityScores { status: Status::Optimal, score: betweenness })
}

impl_solve!(Betweenness<'_, E> => CentralityScores, betweenness_impl);
