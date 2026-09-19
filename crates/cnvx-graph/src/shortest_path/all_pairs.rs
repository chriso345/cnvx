use cnvx_core::{CnvxError, Status};

use crate::NodeId;
use crate::common::dispatch::impl_solve;
use crate::common::priority_queue::PriorityQueue;
use crate::graph::GraphRef;
use crate::weight::WeightFn;

/// Which all-pairs algorithm to run. See [`AllPairsShortestPaths::new`]
/// for the default auto-selection when not set explicitly.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AllPairsAlgorithm {
    /// `O(V^3)` time, `O(V^2)` space, regardless of edge count. Simple,
    /// cache-friendly, and the better choice once the graph is dense
    /// enough that `O(V^3)` and `O(V E log V)` are comparable (roughly
    /// `E` close to `V^2`).
    FloydWarshall,
    /// `O(V E log V)` time: one Bellman-Ford pass to detect negative
    /// cycles and compute a reweighting that makes every edge
    /// non-negative, then one Dijkstra per source over the reweighted
    /// graph. The better choice for sparse graphs (`E` much less than
    /// `V^2`), which is the common case for large real-world graphs.
    Johnson,
}

/// The result of [`AllPairsShortestPaths`]: a dense distance matrix and
/// enough information to reconstruct any shortest path.
pub struct AllPairsSolution {
    status: Status,
    n: usize,
    generation: u32,
    distance: Vec<f64>,
    next: Vec<Option<u32>>,
}

impl AllPairsSolution {
    /// [`Status::Optimal`] normally, or [`Status::Infeasible`] if a
    /// negative-weight cycle was found (in which case every distance is
    /// meaningless.
    pub fn status(&self) -> Status {
        self.status
    }

    /// `true` if a negative-weight cycle was found anywhere in the graph.
    pub fn has_negative_cycle(&self) -> bool {
        self.status == Status::Infeasible
    }

    /// The shortest-path distance from `u` to `v`, or `None` if `v` is
    /// unreachable from `u`.
    pub fn distance(&self, u: NodeId, v: NodeId) -> Option<f64> {
        let d = *self.distance.get(u.index() * self.n + v.index())?;
        if d.is_finite() { Some(d) } else { None }
    }

    /// The sequence of nodes on a shortest path from `u` to `v` (`u`
    /// first, `v` last), or `None` if unreachable.
    pub fn path(&self, u: NodeId, v: NodeId) -> Option<Vec<NodeId>> {
        self.distance(u, v)?;
        let mut nodes = vec![u];
        let mut current = u.index();
        let target = v.index();
        while current != target {
            let next = self.next[current * self.n + target]? as usize;
            nodes.push(NodeId { index: next as u32, generation: self.generation });
            current = next;
        }
        Some(nodes)
    }
}

/// All-pairs shortest paths.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::shortest_path::{AllPairsAlgorithm, AllPairsShortestPaths};
///
/// let mut g = Graph::<(), f64>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// let c = g.add_node(());
/// g.add_edge(a, b, 1.0).unwrap();
/// g.add_edge(b, c, 2.0).unwrap();
///
/// let result = (&g)
///     .solve(
///         &AllPairsShortestPaths::new()
///             .weight(|w: &f64| *w)
///             .algorithm(AllPairsAlgorithm::FloydWarshall),
///     )
///     .unwrap();
/// assert_eq!(result.distance(a, c), Some(3.0));
/// assert_eq!(result.path(a, c), Some(vec![a, b, c]));
/// ```
pub struct AllPairsShortestPaths<'a, E> {
    weight: Option<WeightFn<'a, E>>,
    algorithm: Option<AllPairsAlgorithm>,
}

impl<'a, E> Default for AllPairsShortestPaths<'a, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E> AllPairsShortestPaths<'a, E> {
    /// Creates an all-pairs solver. If [`Self::algorithm`] is not called,
    /// [`AllPairsAlgorithm::Johnson`] is used when `edge_count < node_count^2 /
    /// 4` (a sparse graph) and [`AllPairsAlgorithm::FloydWarshall`]
    /// otherwise.
    pub fn new() -> Self {
        Self { weight: None, algorithm: None }
    }

    /// Sets the edge-weight accessor.
    pub fn weight(mut self, f: impl Fn(&E) -> f64 + 'a) -> Self {
        self.weight = Some(Box::new(f));
        self
    }

    /// Forces a specific algorithm instead of auto-selecting.
    pub fn algorithm(mut self, algorithm: AllPairsAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }
}

fn floyd_warshall_impl<G, N, E>(
    graph: G,
    weight: &WeightFn<'_, E>,
) -> Result<AllPairsSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let ids: Vec<NodeId> = graph.node_ids().collect();
    let n = ids.len();
    let generation = ids.first().map(|id| id.generation).unwrap_or(0);
    let index_of = |id: NodeId| ids.iter().position(|&x| x == id).unwrap();
    let mut dist = vec![f64::INFINITY; n * n];
    let mut next: Vec<Option<u32>> = vec![None; n * n];
    for i in 0..n {
        dist[i * n + i] = 0.0;
    }
    for e in graph.edge_ids() {
        let Some((a, b)) = graph.endpoints(e) else { continue };
        let Some(w) = graph.edge_weight(e).map(weight) else { continue };
        let (i, j) = (index_of(a), index_of(b));
        if w < dist[i * n + j] {
            dist[i * n + j] = w;
            next[i * n + j] = Some(j as u32);
        }
    }
    for k in 0..n {
        for i in 0..n {
            if !dist[i * n + k].is_finite() {
                continue;
            }
            for j in 0..n {
                let via = dist[i * n + k] + dist[k * n + j];
                if via < dist[i * n + j] {
                    dist[i * n + j] = via;
                    next[i * n + j] = next[i * n + k];
                }
            }
        }
    }
    let negative_cycle = (0..n).any(|i| dist[i * n + i] < 0.0);
    Ok(AllPairsSolution {
        status: if negative_cycle { Status::Infeasible } else { Status::Optimal },
        n,
        generation,
        distance: dist,
        next,
    })
}

fn johnson_impl<G, N, E>(
    graph: G,
    weight: &WeightFn<'_, E>,
) -> Result<AllPairsSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let ids: Vec<NodeId> = graph.node_ids().collect();
    let n = ids.len();
    let edge_count = graph.edge_ids().map(|e| e.index() + 1).max().unwrap_or(0);
    let mut raw_weight = vec![0.0; edge_count];
    let mut edges: Vec<(usize, usize, usize)> = Vec::new(); // (u_idx, v_idx, edge_idx)
    let index_of = |id: NodeId| ids.iter().position(|&x| x == id).unwrap();
    for e in graph.edge_ids() {
        if let (Some((a, b)), Some(d)) = (graph.endpoints(e), graph.edge_weight(e)) {
            raw_weight[e.index()] = weight(d);
            edges.push((index_of(a), index_of(b), e.index()));
            // Undirected edges are naturally traversed both ways via
            // `out_edges`/`other_endpoint` elsewhere; here we need the
            // explicit directed pair list for the Bellman-Ford
            // reweighting pass below.
            if !graph.is_directed() {
                edges.push((index_of(b), index_of(a), e.index()));
            }
        }
    }

    // Bellman-Ford with an implicit virtual source of distance 0 to every
    // node (equivalent to a real virtual source with zero-weight edges to
    // everywhere, without needing to materialize one).
    let mut h = vec![0.0; n];
    let mut negative_cycle = false;
    'passes: for pass in 0..n {
        let mut improved = false;
        for &(ui, vi, ei) in &edges {
            let candidate = h[ui] + raw_weight[ei];
            if candidate < h[vi] {
                h[vi] = candidate;
                improved = true;
                if pass == n - 1 {
                    negative_cycle = true;
                    break 'passes;
                }
            }
        }
        if !improved {
            break;
        }
    }
    if negative_cycle {
        return Ok(AllPairsSolution {
            status: Status::Infeasible,
            n,
            generation: ids.first().map(|id| id.generation).unwrap_or(0),
            distance: vec![f64::INFINITY; n * n],
            next: vec![None; n * n],
        });
    }

    let mut dist = vec![f64::INFINITY; n * n];
    let mut next: Vec<Option<u32>> = vec![None; n * n];
    for &start in &ids {
        let si = index_of(start);
        let mut d = vec![f64::INFINITY; n];
        let mut finalized = vec![false; n];
        let mut queue = PriorityQueue::new();
        d[si] = 0.0;
        queue.push(si, 0.0);
        let mut pred: Vec<Option<usize>> = vec![None; n];
        while let Some((u, du)) = queue.pop() {
            if finalized[u] {
                continue;
            }
            finalized[u] = true;
            for e in graph.out_edges(ids[u]) {
                let Some(v) = graph.other_endpoint(e, ids[u]) else { continue };
                let vi = index_of(v);
                if finalized[vi] {
                    continue;
                }
                let w = raw_weight[e.index()] + h[u] - h[vi];
                let candidate = du + w;
                if candidate < d[vi] {
                    d[vi] = candidate;
                    pred[vi] = Some(u);
                    queue.push(vi, candidate);
                }
            }
        }
        for (ti, &dv) in d.iter().enumerate() {
            if dv.is_finite() {
                // Undo the reweighting: real_dist = d' - h[source] + h[target].
                dist[si * n + ti] = dv - h[si] + h[ti];
            }
        }
        // Reconstruct `next` pointers by walking `pred` backward from
        // each target, same technique as `path_reconstruction` elsewhere
        // but written directly against this local `pred` array (a plain
        // `Vec<Option<usize>>` here, not a `PredecessorMap`, since this
        // is node-index-based rather than `NodeId`/`EdgeId`-based).
        for ti in 0..n {
            if !d[ti].is_finite() || ti == si {
                continue;
            }
            let mut walk = ti;
            let mut chain = vec![ti];
            while let Some(p) = pred[walk] {
                chain.push(p);
                walk = p;
                if p == si {
                    break;
                }
            }
            chain.reverse();
            for w in chain.windows(2) {
                next[w[0] * n + ti] = Some(w[1] as u32);
            }
        }
    }

    Ok(AllPairsSolution {
        status: Status::Optimal,
        n,
        generation: ids.first().map(|id| id.generation).unwrap_or(0),
        distance: dist,
        next,
    })
}

pub(crate) fn all_pairs_impl<G, N, E>(
    graph: G,
    solver: &AllPairsShortestPaths<'_, E>,
) -> Result<AllPairsSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    let Some(weight) = &solver.weight else {
        return Err(CnvxError::Numerical(
            "AllPairsShortestPaths: no weight accessor set; call .weight(...)"
                .to_string(),
        ));
    };
    let n = graph.node_count();
    let e = graph.edge_count();
    let algorithm = solver.algorithm.unwrap_or(if e * 4 < n * n {
        AllPairsAlgorithm::Johnson
    } else {
        AllPairsAlgorithm::FloydWarshall
    });
    match algorithm {
        AllPairsAlgorithm::FloydWarshall => floyd_warshall_impl(graph, weight),
        AllPairsAlgorithm::Johnson => johnson_impl(graph, weight),
    }
}

impl_solve!(AllPairsShortestPaths<'_, E> => AllPairsSolution, all_pairs_impl);

#[cfg(test)]
mod tests {
    use cnvx_core::Solve;

    use super::*;
    use crate::Graph;

    #[test]
    fn johnson_matches_floyd_warshall_with_negative_edges() {
        let mut g = Graph::<(), f64>::directed();
        let a = g.add_node(());
        let b = g.add_node(());
        let c = g.add_node(());
        let d = g.add_node(());
        g.add_edge(a, b, 3.0).unwrap();
        g.add_edge(a, c, 8.0).unwrap();
        g.add_edge(a, d, -4.0).unwrap();
        g.add_edge(b, d, 1.0).unwrap();
        g.add_edge(c, b, 4.0).unwrap();
        g.add_edge(d, c, 2.0).unwrap();

        let fw = (&g)
            .solve(
                &AllPairsShortestPaths::new()
                    .weight(|w: &f64| *w)
                    .algorithm(AllPairsAlgorithm::FloydWarshall),
            )
            .unwrap();
        let johnson = (&g)
            .solve(
                &AllPairsShortestPaths::new()
                    .weight(|w: &f64| *w)
                    .algorithm(AllPairsAlgorithm::Johnson),
            )
            .unwrap();
        for u in [a, b, c, d] {
            for v in [a, b, c, d] {
                assert_eq!(fw.distance(u, v), johnson.distance(u, v), "{u:?} -> {v:?}");
            }
        }
        assert_eq!(fw.distance(a, c), Some(-2.0));
    }

    #[test]
    fn detects_negative_cycle() {
        let mut g = Graph::<(), f64>::directed();
        let a = g.add_node(());
        let b = g.add_node(());
        g.add_edge(a, b, 1.0).unwrap();
        g.add_edge(b, a, -3.0).unwrap();
        let result = (&g)
            .solve(&AllPairsShortestPaths::new().weight(|w: &f64| *w))
            .unwrap();
        assert!(result.has_negative_cycle());
    }
}
