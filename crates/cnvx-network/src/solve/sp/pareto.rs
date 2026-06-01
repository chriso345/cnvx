use std::cmp::Ordering;

/// A single Pareto-optimal label at a node.
#[derive(Debug, Clone)]
pub struct ParetoLabel {
    /// Accumulated cost vector from the source to this node.
    pub cost: Vec<f64>,
    /// Predecessor node, or `None` at the source.
    pub prev_node: Option<usize>,
    /// Edge used to arrive here, or `None` at the source.
    pub prev_edge: Option<usize>,
    /// Index into `ParetoResult.labels[prev_node]` of the predecessor label,
    /// or `None` at the source.
    pub prev_label_idx: Option<usize>,
}

/// Result from [`LabelSet`].
///
/// For single-criterion solves, each node has exactly one label.
/// For multicriteria solves, each node has zero or more Pareto-optimal labels.
#[derive(Debug, Clone)]
pub struct ParetoResult {
    /// `labels[v]` = all Pareto-optimal labels settled at node `v`.
    pub labels: Vec<Vec<ParetoLabel>>,

    /// Number of criteria in the cost vectors.
    pub num_criteria: usize,
}

impl ParetoResult {
    /// All Pareto-optimal labels at `node`.
    pub fn labels_at(&self, node: usize) -> &[ParetoLabel] {
        &self.labels[node]
    }

    /// Whether `target` is reachable from the source.
    pub fn is_reachable(&self, target: usize) -> bool {
        !self.labels[target].is_empty()
    }

    /// Shortest cost vector at `target` (for single-criterion solves,
    /// this is the unique label). Returns `None` if unreachable.
    pub fn best_cost(&self, target: usize) -> Option<&Vec<f64>> {
        self.labels[target].first().map(|l| &l.cost)
    }

    /// Ordered node ids on the path corresponding to label `label_idx` at `target`.
    ///
    /// Returns an empty vec if `label_idx` is out of range or the path
    /// does not originate at `source`.
    pub fn node_path(
        &self,
        source: usize,
        target: usize,
        label_idx: usize,
    ) -> Vec<usize> {
        let mut path = vec![target];
        let mut cur_node = target;
        let mut cur_idx = label_idx;

        loop {
            let lbl = &self.labels[cur_node][cur_idx];
            match lbl.prev_node {
                None => break,
                Some(pn) => {
                    path.push(pn);
                    cur_node = pn;
                    cur_idx = lbl.prev_label_idx.unwrap();
                }
            }
        }

        path.reverse();
        if path.first() != Some(&source) {
            return vec![];
        }
        path
    }

    /// Ordered edge ids on the path corresponding to label `label_idx` at `target`.
    pub fn edge_path(
        &self,
        source: usize,
        target: usize,
        label_idx: usize,
    ) -> Vec<usize> {
        let mut edges = Vec::new();
        let mut cur_node = target;
        let mut cur_idx = label_idx;

        loop {
            let lbl = &self.labels[cur_node][cur_idx];
            match lbl.prev_edge {
                None => break,
                Some(e) => {
                    edges.push(e);
                    cur_node = lbl.prev_node.unwrap();
                    cur_idx = lbl.prev_label_idx.unwrap();
                }
            }
        }

        edges.reverse();
        let _ = source;
        edges
    }
}

#[derive(Clone)]
pub(crate) struct Entry {
    pub(crate) cost: Vec<f64>,
    pub(crate) node: usize,
    pub(crate) src_node: usize,
    pub(crate) src_edge: usize,
    pub(crate) src_label_idx: usize,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.node == other.node
    }
}
impl Eq for Entry {}

// Lexicographic min-heap.
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        for (a, b) in other.cost.iter().zip(self.cost.iter()) {
            match a.partial_cmp(b).unwrap_or(Ordering::Equal) {
                Ordering::Equal => continue,
                other => return other,
            }
        }
        Ordering::Equal
    }
}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
