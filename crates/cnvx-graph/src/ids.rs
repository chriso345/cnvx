/// A handle to a node in a [`Graph`](crate::Graph).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

impl NodeId {
    /// The handle's position within the graph that created it.
    pub fn index(&self) -> usize {
        self.index as usize
    }
}

/// A handle to an edge in a [`Graph`](crate::Graph).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct EdgeId {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

impl EdgeId {
    /// The handle's position within the graph that created it.
    pub fn index(&self) -> usize {
        self.index as usize
    }
}
