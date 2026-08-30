use cnvx_core::{CnvxError, Status};
use cnvx_math::Vector;

use crate::common::dispatch::impl_solve;
use crate::graph::GraphRef;
use crate::shortest_path::{Dijkstra, ShortestPathSolution, dijkstra_impl};
use crate::weight::CriteriaFn;
use crate::{EdgeId, NodeId};

/// How a single criterion participates in a lexicographic ordering.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LexPriority {
    /// Minimize this criterion (higher priority = earlier in the list).
    Min,
    /// Maximize this criterion.
    Max,
    /// Not yet implemented.
    SumMin,
    /// Not yet implemented.
    SumMax,
    /// Not yet implemented.
    ThresholdMin,
}

/// The result of a [`LexicographicShortestPath`] search.
pub struct LexicographicSolution {
    inner: ShortestPathSolution,
    criteria_cost: Vec<Option<Vector>>,
}

impl LexicographicSolution {
    /// Always [`Status::Optimal`].
    pub fn status(&self) -> Status {
        self.inner.status()
    }

    /// The true, per-criterion accumulated cost of the lexicographically
    /// optimal path to `node` (unlike the internal big-M scalar used to
    /// find it, this is directly meaningful in the criteria's own units).
    pub fn criteria_to(&self, node: NodeId) -> Option<&Vector> {
        self.criteria_cost.get(node.index())?.as_ref()
    }

    /// The edges on the lexicographically optimal path to `node`.
    pub fn path_to(&self, node: NodeId) -> Option<Vec<EdgeId>> {
        self.inner.path_to(node)
    }
}

/// Lexicographic multi-criteria shortest paths.
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::multi_criteria::{LexPriority, LexicographicShortestPath};
/// use cnvx_math::Vector;
///
/// struct Road {
///     time: f64,
///     cost: f64,
/// }
///
/// let mut g = Graph::<(), Road>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// // Same time, different cost: the cheaper one should win under
/// // lexicographic (time first, then cost) ordering.
/// g.add_edge(a, b, Road { time: 10.0, cost: 5.0 }).unwrap();
/// g.add_edge(a, b, Road { time: 10.0, cost: 2.0 }).unwrap();
///
/// let result = (&g)
///     .solve(
///         &LexicographicShortestPath::new(a)
///             .criteria(|r: &Road| Vector::from_slice(&[r.time, r.cost]))
///             .priorities(vec![LexPriority::Min, LexPriority::Min]),
///     )
///     .unwrap();
/// assert_eq!(result.criteria_to(b).unwrap()[1], 2.0);
/// ```
pub struct LexicographicShortestPath<'a, E> {
    source: NodeId,
    criteria: Option<CriteriaFn<'a, E>>,
    priorities: Option<Vec<LexPriority>>,
}

impl<'a, E> LexicographicShortestPath<'a, E> {
    /// Creates a lexicographic search rooted at `source`.
    pub fn new(source: NodeId) -> Self {
        Self { source, criteria: None, priorities: None }
    }

    /// Sets the criteria accessor.
    pub fn criteria(mut self, f: impl Fn(&E) -> Vector + 'a) -> Self {
        self.criteria = Some(Box::new(f));
        self
    }

    /// Sets the priority order, highest priority first, one entry per
    /// criterion. Defaults to [`LexPriority::Min`] for every criterion if
    /// not set.
    pub fn priorities(mut self, priorities: Vec<LexPriority>) -> Self {
        self.priorities = Some(priorities);
        self
    }
}

fn lexicographic_impl<G, N, E>(
    graph: G,
    solver: &LexicographicShortestPath<'_, E>,
) -> Result<LexicographicSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    if !graph.contains_node(solver.source) {
        return Err(CnvxError::ForeignHandle);
    }
    let Some(criteria) = &solver.criteria else {
        return Err(CnvxError::Numerical(
            "LexicographicShortestPath: no criteria accessor set; call .criteria(...)"
                .to_string(),
        ));
    };

    let k = graph
        .edge_ids()
        .find_map(|e| graph.edge_weight(e).map(|d| criteria(d).len()));
    let Some(k) = k else {
        // No edges: trivial solution, the source alone.
        let sp = dijkstra_impl(graph, &Dijkstra::new(solver.source).weight(|_: &E| 0.0))?;
        let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
        return Ok(LexicographicSolution { inner: sp, criteria_cost: vec![None; n] });
    };
    let priorities =
        solver.priorities.clone().unwrap_or_else(|| vec![LexPriority::Min; k]);
    if priorities.len() != k {
        return Err(CnvxError::Numerical(format!(
            "LexicographicShortestPath: {} priorities given for {k} criteria",
            priorities.len()
        )));
    }
    for p in &priorities {
        if !matches!(p, LexPriority::Min | LexPriority::Max) {
            return Err(CnvxError::Numerical(
                "LexicographicShortestPath: only LexPriority::Min and LexPriority::Max are implemented"
                    .to_string(),
            ));
        }
    }

    let mut max_component = 0.0f64;
    let mut edge_count = 0usize;
    for e in graph.edge_ids() {
        if let Some(d) = graph.edge_weight(e) {
            let v = criteria(d);
            edge_count += 1;
            for i in 0..v.len() {
                max_component = max_component.max(v[i].abs());
            }
        }
    }
    let big = max_component * (edge_count as f64 + 1.0) + 1.0;

    let scalar_weight = move |d: &E| -> f64 {
        let v = criteria(d);
        let mut total = 0.0;
        for i in 0..k {
            let sign = if priorities[i] == LexPriority::Max { -1.0 } else { 1.0 };
            total += sign * big.powi((k - 1 - i) as i32) * v[i];
        }
        total
    };

    let sp = dijkstra_impl(graph, &Dijkstra::new(solver.source).weight(scalar_weight))?;

    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut criteria_cost = vec![None; n];
    for id in graph.node_ids() {
        if let Some(edges) = sp.path_to(id) {
            let mut total = Vector::zeros(k);
            for e in edges {
                if let Some(d) = graph.edge_weight(e) {
                    total = &total + &criteria(d);
                }
            }
            criteria_cost[id.index()] = Some(total);
        }
    }

    Ok(LexicographicSolution { inner: sp, criteria_cost })
}

impl_solve!(LexicographicShortestPath<'_, E> => LexicographicSolution, lexicographic_impl);
