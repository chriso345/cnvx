use crate::Graph;

pub struct AdjacencyGraph<N, E> {
    pub nodes: Vec<N>,
    pub edges: Vec<EdgeRecord<E>>,
    pub out_adj: Vec<Vec<usize>>,
    pub in_adj: Vec<Vec<usize>>,
}

pub struct EdgeRecord<E> {
    pub from: usize,
    pub to: usize,
    pub data: E,
}

impl<N, E> Graph for AdjacencyGraph<N, E> {
    type Vertex = usize;
    type Edge = usize;
    type NodeData = N;
    type EdgeData = E;

    fn add_vertex(&mut self, data: N) -> usize {
        let id = self.nodes.len();
        self.nodes.push(data);
        self.out_adj.push(Vec::new());
        self.in_adj.push(Vec::new());
        id
    }

    fn remove_vertex(&mut self, _v: usize) {
        unimplemented!("vertex removal not yet supported")
    }

    fn add_edge(&mut self, from: usize, to: usize, data: E) -> usize {
        let id = self.edges.len();
        self.edges.push(EdgeRecord { from, to, data });
        self.out_adj[from].push(id);
        self.in_adj[to].push(id);
        id
    }

    fn remove_edge(&mut self, _e: usize) {
        unimplemented!("edge removal not yet supported")
    }

    fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
    fn num_edges(&self) -> usize {
        self.edges.len()
    }
    fn source(&self, e: usize) -> usize {
        self.edges[e].from
    }
    fn target(&self, e: usize) -> usize {
        self.edges[e].to
    }

    fn out_edges(&self, v: usize) -> impl Iterator<Item = usize> {
        self.out_adj[v].iter().copied()
    }

    fn edge_data(&self, e: usize) -> &E {
        &self.edges[e].data
    }
}
