pub mod adjacency;

pub use adjacency::*;

pub trait Graph {
    type Vertex: Copy + Eq;
    type Edge: Copy + Eq;
    type NodeData;
    type EdgeData;

    fn add_vertex(&mut self, data: Self::NodeData) -> Self::Vertex;
    fn remove_vertex(&mut self, v: Self::Vertex);
    fn add_edge(
        &mut self,
        from: Self::Vertex,
        to: Self::Vertex,
        data: Self::EdgeData,
    ) -> Self::Edge;
    fn remove_edge(&mut self, e: Self::Edge);
    fn num_nodes(&self) -> usize;
    fn num_edges(&self) -> usize;
    fn source(&self, e: Self::Edge) -> Self::Vertex;
    fn target(&self, e: Self::Edge) -> Self::Vertex;

    /// Outgoing edge ids from `v`.
    fn out_edges(&self, v: Self::Vertex) -> impl Iterator<Item = Self::Edge>;

    /// Reference to the data on edge `e`.
    fn edge_data(&self, e: Self::Edge) -> &Self::EdgeData;
}
