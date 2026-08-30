use crate::common::distance_map::PredecessorMap;
use crate::{EdgeId, NodeId};

pub(crate) fn reconstruct_edge_path(
    pred: &PredecessorMap,
    source: NodeId,
    target: NodeId,
) -> Option<Vec<EdgeId>> {
    if source == target {
        return Some(Vec::new());
    }
    let mut edges = Vec::new();
    let mut current = target;
    loop {
        let (prev, edge) = pred.get(current)?;
        edges.push(edge);
        if prev == source {
            edges.reverse();
            return Some(edges);
        }
        current = prev;

        if edges.len() > pred_len(pred) {
            return None;
        }
    }
}

fn pred_len(pred: &PredecessorMap) -> usize {
    pred.len()
}
