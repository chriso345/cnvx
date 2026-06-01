use std::collections::BinaryHeap;

use crate::{Entry, Graph, ParetoLabel, ParetoResult, SPType};

/// Label-setting solver over any [`Graph`].
///
/// Single-criterion shortest paths are the special case of a 1-element cost vector.
pub struct LabelSet<'g, G>
where
    G: Graph<Vertex = usize, Edge = usize>,
{
    /// The graph to solve on. Must not be modified during the solve.
    graph: &'g G,

    /// The type of shortest path problem to solve.
    sp_type: SPType,

    /// The cost function is provided at solve time, as a closure mapping edge data to cost vectors.
    cost_fn: Option<Box<dyn Fn(&G::EdgeData) -> Vec<f64> + 'g>>,
}

impl<'g, G> LabelSet<'g, G>
where
    G: Graph<Vertex = usize, Edge = usize>,
{
    pub fn new(graph: &'g G) -> Self {
        Self { graph, sp_type: SPType::None, cost_fn: None }
    }

    pub fn set_sp_type(&mut self, sp_type: SPType) {
        self.sp_type = sp_type;
    }

    pub fn set_cost_function<F>(&mut self, cost_fn: F)
    where
        F: Fn(&G::EdgeData) -> Vec<f64> + 'g,
    {
        self.cost_fn = Some(Box::new(cost_fn));
    }
}

impl<'g, G> LabelSet<'g, G>
where
    G: Graph<Vertex = usize, Edge = usize>,
{
    fn source(&self) -> usize {
        match self.sp_type {
            SPType::OneToOne(s, _) | SPType::OneToAll(s) => s,
            SPType::None => panic!("SPType::None does not have a source node"),
        }
    }

    /// Run the multicriteria label-setting algorithm with a vector cost closure.
    ///
    /// `cost_fn` must return a `Vec<f64>` of the same length for every edge,
    /// with all components non-negative.
    pub fn solve(&self) -> ParetoResult
where {
        let source = self.source();
        let n = self.graph.num_nodes();

        let cost_fn = self
            .cost_fn
            .as_ref()
            .expect("cost function must be provided for non-empty graph");

        let num_criteria = if self.graph.num_edges() > 0 {
            let c = cost_fn(self.graph.edge_data(0));
            assert!(!c.is_empty(), "cost vector must be non-empty");
            c.len()
        } else {
            // No edges: trivial result, source reaches only itself with zero cost.
            let mut labels = vec![vec![]; n];
            labels[source].push(ParetoLabel {
                cost: vec![],
                prev_node: None,
                prev_edge: None,
                prev_label_idx: None,
            });
            return ParetoResult { labels, num_criteria: 0 };
        };

        let mut labels: Vec<Vec<ParetoLabel>> = vec![vec![]; n];

        // Seed source with zero-cost label.
        labels[source].push(ParetoLabel {
            cost: vec![0.0; num_criteria],
            prev_node: None,
            prev_edge: None,
            prev_label_idx: None,
        });

        let mut heap: BinaryHeap<Entry> = BinaryHeap::new();

        // Initialise heap with all arcs out of source.
        for edge_id in self.graph.out_edges(source) {
            let c = cost_fn(self.graph.edge_data(edge_id));
            assert_eq!(c.len(), num_criteria, "all cost vectors must have equal length");
            debug_assert!(c.iter().all(|&x| x >= 0.0), "costs must be non-negative");
            heap.push(Entry {
                cost: c,
                node: self.graph.target(edge_id),
                src_node: source,
                src_edge: edge_id,
                src_label_idx: 0,
            });
        }

        while let Some(entry) = heap.pop() {
            let u = entry.node;

            // Try to settle this label at u.
            let new_label = ParetoLabel {
                cost: entry.cost.clone(),
                prev_node: Some(entry.src_node),
                prev_edge: Some(entry.src_edge),
                prev_label_idx: Some(entry.src_label_idx),
            };

            let inserted_idx = match try_insert(&mut labels[u], new_label) {
                Some(idx) => idx,
                None => continue, // dominated, discard
            };

            // NOTE: For multicriteria we cannot exit early even in OneToOne,
            // because later labels at the target may still be non-dominated.
            // We run to exhaustion.

            // Extend to all outgoing arcs from u.
            for edge_id in self.graph.out_edges(u) {
                let arc_cost = cost_fn(self.graph.edge_data(edge_id));
                assert_eq!(
                    arc_cost.len(),
                    num_criteria,
                    "all cost vectors must have equal length"
                );
                debug_assert!(
                    arc_cost.iter().all(|&x| x >= 0.0),
                    "costs must be non-negative"
                );

                let v = self.graph.target(edge_id);

                let candidate: Vec<f64> =
                    entry.cost.iter().zip(arc_cost.iter()).map(|(a, b)| a + b).collect();

                // Skip if already dominated by a settled label at v.
                if labels[v].iter().any(|lbl| dominates(&lbl.cost, &candidate)) {
                    continue;
                }

                heap.push(Entry {
                    cost: candidate,
                    node: v,
                    src_node: u,
                    src_edge: edge_id,
                    src_label_idx: inserted_idx,
                });
            }
        }

        ParetoResult { labels, num_criteria }
    }
}

/// Returns `true` if `a` strictly dominates `b`:
/// every component of `a` <= `b`, with at least one strictly less.
fn dominates(a: &[f64], b: &[f64]) -> bool {
    debug_assert_eq!(a.len(), b.len());
    let mut strictly_less = false;
    for (x, y) in a.iter().zip(b.iter()) {
        if x > y {
            return false;
        }
        if x < y {
            strictly_less = true;
        }
    }
    strictly_less
}

/// Attempt to insert `new_label` into `frontier`.
///
/// - Discards `new_label` if any existing label dominates it.
/// - Removes any existing labels that `new_label` dominates.
/// - Returns the index of the inserted label, or `None` if discarded.
fn try_insert(frontier: &mut Vec<ParetoLabel>, new_label: ParetoLabel) -> Option<usize> {
    for existing in frontier.iter() {
        if dominates(&existing.cost, &new_label.cost) {
            return None;
        }
    }
    frontier.retain(|existing| !dominates(&new_label.cost, &existing.cost));
    let idx = frontier.len();
    frontier.push(new_label);
    Some(idx)
}
