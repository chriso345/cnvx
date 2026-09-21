use crate::NodeId;

/// A dense, `NodeId`-indexed distance map, defaulting every node to `+INF`
/// (unreached) until relaxed.
#[allow(dead_code)]
pub(crate) struct DistanceMap {
    dist: Vec<f64>,
}

#[allow(dead_code)]
impl DistanceMap {
    pub(crate) fn new(n: usize) -> Self {
        Self { dist: vec![f64::INFINITY; n] }
    }

    pub(crate) fn get(&self, id: NodeId) -> f64 {
        self.dist[id.index()]
    }

    pub(crate) fn set(&mut self, id: NodeId, d: f64) {
        self.dist[id.index()] = d;
    }

    /// Relaxes the distance to `id` down to `candidate` if it is an
    /// improvement. Returns `true` if it improved.
    pub(crate) fn relax(&mut self, id: NodeId, candidate: f64) -> bool {
        if candidate < self.dist[id.index()] {
            self.dist[id.index()] = candidate;
            true
        } else {
            false
        }
    }

    pub(crate) fn into_vec(self) -> Vec<f64> {
        self.dist
    }
}

/// A dense, `NodeId`-indexed predecessor map: for each node, the edge that
/// last improved its distance (if any), used for path reconstruction.
pub(crate) struct PredecessorMap {
    pred: Vec<Option<(NodeId, crate::EdgeId)>>,
}

impl PredecessorMap {
    pub(crate) fn new(n: usize) -> Self {
        Self { pred: vec![None; n] }
    }

    pub(crate) fn set(&mut self, id: NodeId, via: (NodeId, crate::EdgeId)) {
        self.pred[id.index()] = Some(via);
    }

    pub(crate) fn get(&self, id: NodeId) -> Option<(NodeId, crate::EdgeId)> {
        self.pred[id.index()]
    }

    pub(crate) fn len(&self) -> usize {
        self.pred.len()
    }
}
