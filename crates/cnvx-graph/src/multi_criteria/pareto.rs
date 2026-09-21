use cnvx_core::{CnvxError, Status};
use cnvx_math::Vector;

use crate::common::dispatch::impl_solve;
use crate::graph::GraphRef;
use crate::weight::CriteriaFn;
use crate::{EdgeId, NodeId};

/// A single Pareto-optimal path: its accumulated criteria vector and the
/// edges traversed to achieve it.
pub struct ParetoPath {
    cost: Vector,
    path: Vec<EdgeId>,
}

impl ParetoPath {
    /// The accumulated cost, one component per criterion, in the order
    /// the [`ParetoShortestPath::criteria`] accessor produces them.
    pub fn cost(&self) -> &Vector {
        &self.cost
    }

    /// The edges traversed, source to destination.
    pub fn path(&self) -> &[EdgeId] {
        &self.path
    }
}

/// Multi-objective shortest paths via LabelSetting
///
/// # Example
/// ```
/// use cnvx_core::Solve;
/// use cnvx_graph::Graph;
/// use cnvx_graph::multi_criteria::ParetoShortestPath;
/// use cnvx_math::Vector;
///
/// struct Road {
///     distance: f64,
///     toll: f64,
/// }
///
/// let mut g = Graph::<(), Road>::directed();
/// let a = g.add_node(());
/// let b = g.add_node(());
/// // Two routes: fast-and-expensive, slow-and-cheap. Neither dominates
/// // the other, so both should appear in the Pareto front.
/// g.add_edge(a, b, Road { distance: 10.0, toll: 5.0 }).unwrap();
/// g.add_edge(a, b, Road { distance: 20.0, toll: 1.0 }).unwrap();
///
/// let result = (&g)
///     .solve(
///         &ParetoShortestPath::new(a)
///             .criteria(|r: &Road| Vector::from_slice(&[r.distance, r.toll])),
///     )
///     .unwrap();
/// assert_eq!(result.pareto_front(b).len(), 2);
/// ```
pub struct ParetoShortestPath<'a, E> {
    source: NodeId,
    target: Option<NodeId>,
    criteria: Option<CriteriaFn<'a, E>>,
}

impl<'a, E> ParetoShortestPath<'a, E> {
    /// Creates a Pareto search rooted at `source`. A criteria accessor
    /// must be supplied via [`Self::criteria`] before solving.
    pub fn new(source: NodeId) -> Self {
        Self { source, target: None, criteria: None }
    }

    /// Sets the criteria accessor, projecting each edge's data to a
    /// vector of non-negative criteria values.
    pub fn criteria(mut self, f: impl Fn(&E) -> Vector + 'a) -> Self {
        self.criteria = Some(Box::new(f));
        self
    }

    pub fn target(mut self, target: NodeId) -> Self {
        self.target = Some(target);
        self
    }
}

/// One generated label: a Pareto-candidate partial-path cost arriving at
/// `node`, with enough information (`pred`) to reconstruct the path.
struct Label {
    node: NodeId,
    cost: Vector,
    pred: Option<(usize, EdgeId)>,
}

/// The result of a [`ParetoShortestPath`] search: the Pareto front of
/// non-dominated arrival costs at every node.
pub struct ParetoShortestPathSolution {
    status: Status,
    labels: Vec<Label>,
    /// Indices into `labels` of the finalized (Pareto-optimal) labels at
    /// each node.
    fronts: Vec<Vec<usize>>,
}

impl ParetoShortestPathSolution {
    /// Always [`Status::Optimal`].
    pub fn status(&self) -> Status {
        self.status
    }

    /// Every Pareto-optimal path from the source to `node`.
    pub fn pareto_front(&self, node: NodeId) -> Vec<ParetoPath> {
        let Some(indices) = self.fronts.get(node.index()) else { return Vec::new() };
        indices.iter().map(|&i| self.reconstruct(i)).collect()
    }

    fn reconstruct(&self, mut label_index: usize) -> ParetoPath {
        let cost = self.labels[label_index].cost.clone();
        let mut edges = Vec::new();
        loop {
            let label = &self.labels[label_index];
            match label.pred {
                Some((prev, edge)) => {
                    edges.push(edge);
                    label_index = prev;
                }
                None => break,
            }
        }
        edges.reverse();
        ParetoPath { cost, path: edges }
    }
}

struct HeapEntry {
    label_index: usize,
    cost: Vector,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cost.sum() == other.cost.sum()
    }
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .sum()
            .partial_cmp(&self.cost.sum())
            .expect("criteria must not be NaN")
    }
}

fn pareto_impl<G, N, E>(
    graph: G,
    solver: &ParetoShortestPath<'_, E>,
) -> Result<ParetoShortestPathSolution, CnvxError>
where
    G: GraphRef<N, E> + Copy,
{
    if !graph.contains_node(solver.source) {
        return Err(CnvxError::ForeignHandle);
    }
    let Some(criteria) = &solver.criteria else {
        return Err(CnvxError::Numerical(
            "ParetoShortestPath: no criteria accessor set; call .criteria(...)"
                .to_string(),
        ));
    };

    let n = graph.node_ids().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut labels: Vec<Label> = Vec::new();
    let mut fronts: Vec<Vec<usize>> = vec![Vec::new(); n];
    // Labels generated for a node but not yet finalized (a superset of
    // what will end up in `fronts`; used only for dominance pruning
    // before an entry is popped).
    let mut pending: Vec<Vec<usize>> = vec![Vec::new(); n];

    let mut heap = std::collections::BinaryHeap::new();

    labels.push(Label {
        node: solver.source,
        cost: Vector::zeros(0),
        pred: None,
    });
    pending[solver.source.index()].push(0);
    heap.push(HeapEntry { label_index: 0, cost: Vector::zeros(0) });

    while let Some(HeapEntry { label_index, .. }) = heap.pop() {
        let node = labels[label_index].node;
        let cost = labels[label_index].cost.clone();

        // Dominance check against already-finalized labels at this node.
        if fronts[node.index()]
            .iter()
            .any(|&fi| labels[fi].cost.dominates_or_equal(&cost) && fi != label_index)
        {
            continue;
        }
        fronts[node.index()].push(label_index);

        if Some(node) == solver.target {
            continue;
        }

        for e in graph.out_edges(node) {
            let Some(v) = graph.other_endpoint(e, node) else { continue };
            let Some(edge_cost) = graph.edge_weight(e).map(criteria) else { continue };
            if edge_cost.is_empty() {
                return Err(CnvxError::Numerical(
                    "ParetoShortestPath: criteria accessor returned an empty vector"
                        .to_string(),
                ));
            }
            for i in 0..edge_cost.len() {
                if edge_cost[i] < 0.0 {
                    return Err(CnvxError::Numerical(
                        "ParetoShortestPath requires every criterion to be non-negative"
                            .to_string(),
                    ));
                }
            }
            let base = if label_index == 0 && cost.is_empty() {
                Vector::zeros(edge_cost.len())
            } else {
                cost.clone()
            };
            let new_cost = &base + &edge_cost;

            // Prune against the destination's already-finalized front.
            if fronts[v.index()]
                .iter()
                .any(|&fi| labels[fi].cost.dominates_or_equal(&new_cost))
            {
                continue;
            }
            // Prune against the destination's other pending labels too,
            // and drop any pending labels the new one dominates (kept
            // list stays a genuine antichain).
            if pending[v.index()]
                .iter()
                .any(|&pi| labels[pi].cost.dominates_or_equal(&new_cost))
            {
                continue;
            }

            let new_index = labels.len();
            labels.push(Label {
                node: v,
                cost: new_cost.clone(),
                pred: Some((label_index, e)),
            });
            pending[v.index()].push(new_index);
            heap.push(HeapEntry { label_index: new_index, cost: new_cost });
        }
    }

    Ok(ParetoShortestPathSolution { status: Status::Optimal, labels, fronts })
}

impl_solve!(ParetoShortestPath<'_, E> => ParetoShortestPathSolution, pareto_impl);
