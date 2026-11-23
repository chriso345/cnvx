use crate::graph::next_generation;
use crate::weight::GraphWeight;
use crate::{EdgeId, Graph, NodeId};

/// How [`Graph::simplify`] should pick a representative edge when
/// collapsing a group of parallel edges (same endpoints) into one.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EdgeSelection {
    /// Keep whichever parallel edge has the smallest
    /// [`GraphWeight::measure`].
    MinWeight,
    /// Keep whichever parallel edge has the largest
    /// [`GraphWeight::measure`].
    MaxWeight,
    /// Keep whichever parallel edge was added first.
    First,
}

impl<N: Clone, E: Clone> Graph<N, E> {
    /// Returns a new graph with every edge's direction reversed (`u -> v`
    /// becomes `v -> u`). For an undirected graph, returns an equivalent
    /// copy.
    ///
    /// # Example
    /// ```
    /// use cnvx_graph::Graph;
    ///
    /// let mut g = Graph::<(), ()>::directed();
    /// let a = g.add_node(());
    /// let b = g.add_node(());
    /// g.add_edge(a, b, ()).unwrap();
    ///
    /// let reversed = g.reverse();
    /// let (ra, rb) =
    ///     (reversed.node_ids().next().unwrap(), reversed.node_ids().nth(1).unwrap());
    /// assert_eq!(reversed.out_degree(rb), 1);
    /// assert_eq!(reversed.out_degree(ra), 0);
    /// ```
    pub fn reverse(&self) -> Graph<N, E> {
        let mut out = if self.directed { Graph::directed() } else { Graph::undirected() };
        let mut map = vec![None; self.nodes.len()];
        for (id, data) in self.nodes() {
            map[id.index()] = Some(out.add_node(data.clone()));
        }
        for (id, data) in self.edges() {
            let (a, b) = self.endpoints(id).unwrap();
            let (a2, b2) = (map[a.index()].unwrap(), map[b.index()].unwrap());
            if self.directed {
                out.add_edge(b2, a2, data.clone()).unwrap();
            } else {
                out.add_edge(a2, b2, data.clone()).unwrap();
            }
        }
        out
    }

    /// Returns a new undirected graph with the same nodes and edges
    /// (direction discarded).
    pub fn to_undirected(&self) -> Graph<N, E> {
        let mut out = Graph::undirected();
        let mut map = vec![None; self.nodes.len()];
        for (id, data) in self.nodes() {
            map[id.index()] = Some(out.add_node(data.clone()));
        }
        for (id, data) in self.edges() {
            let (a, b) = self.endpoints(id).unwrap();
            let (a2, b2) = (map[a.index()].unwrap(), map[b.index()].unwrap());
            out.add_edge(a2, b2, data.clone()).unwrap();
        }
        out
    }

    /// Returns a new graph containing only `nodes` and the edges of
    /// `self` with both endpoints in `nodes`.
    ///
    /// # Example
    /// ```
    /// use cnvx_graph::Graph;
    ///
    /// let mut g = Graph::<&str, ()>::undirected();
    /// let a = g.add_node("a");
    /// let b = g.add_node("b");
    /// let c = g.add_node("c");
    /// g.add_edge(a, b, ()).unwrap();
    /// g.add_edge(b, c, ()).unwrap();
    ///
    /// let sub = g.induced_subgraph(&[a, b]);
    /// assert_eq!(sub.node_count(), 2);
    /// assert_eq!(sub.edge_count(), 1);
    /// ```
    pub fn induced_subgraph(&self, nodes: &[NodeId]) -> Graph<N, E> {
        let mut out = if self.directed { Graph::directed() } else { Graph::undirected() };
        let mut map = vec![None; self.nodes.len()];
        for &id in nodes {
            if let Some(data) = self.node_weight(id) {
                map[id.index()] = Some(out.add_node(data.clone()));
            }
        }
        for (id, data) in self.edges() {
            let (a, b) = self.endpoints(id).unwrap();
            if let (Some(a2), Some(b2)) = (map[a.index()], map[b.index()]) {
                out.add_edge(a2, b2, data.clone()).unwrap();
            }
        }
        out
    }

    /// Removes every edge with `source == target`, in place.
    pub fn remove_self_loops(&mut self) {
        let keep: std::collections::HashSet<EdgeId> = self
            .edges()
            .filter_map(|(id, _)| self.endpoints(id).map(|(a, b)| (id, a != b)))
            .filter(|&(_, keep)| keep)
            .map(|(id, _)| id)
            .collect();
        self.retain_edges_by_id(|id| keep.contains(&id));
    }

    /// Removes parallel edges (same endpoints), keeping one per pair per
    /// `selection`, in place.
    ///
    /// # Example
    /// ```
    /// use cnvx_graph::Graph;
    /// use cnvx_graph::manipulation::EdgeSelection;
    ///
    /// let mut g = Graph::<(), f64>::undirected();
    /// let a = g.add_node(());
    /// let b = g.add_node(());
    /// g.add_edge(a, b, 5.0).unwrap();
    /// g.add_edge(a, b, 1.0).unwrap();
    /// assert_eq!(g.edge_count(), 2);
    ///
    /// g.simplify(EdgeSelection::MinWeight);
    /// assert_eq!(g.edge_count(), 1);
    /// ```
    pub fn simplify(&mut self, selection: EdgeSelection)
    where
        E: GraphWeight,
    {
        let mut best: std::collections::HashMap<(u32, u32), (EdgeId, f64)> =
            std::collections::HashMap::new();
        for (id, data) in self.edges() {
            let (a, b) = self.endpoints(id).unwrap();
            let key = if self.directed || a.index() <= b.index() {
                (a.index() as u32, b.index() as u32)
            } else {
                (b.index() as u32, a.index() as u32)
            };
            let measure = data.measure();
            let entry = best.entry(key);
            match entry {
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert((id, measure));
                }
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    let (_, current_measure) = *o.get();
                    let replace = match selection {
                        EdgeSelection::MinWeight => measure < current_measure,
                        EdgeSelection::MaxWeight => measure > current_measure,
                        EdgeSelection::First => false,
                    };
                    if replace {
                        o.insert((id, measure));
                    }
                }
            }
        }
        let keep: std::collections::HashSet<EdgeId> =
            best.values().map(|&(id, _)| id).collect();
        self.retain_edges_by_id(|id| keep.contains(&id));
    }

    /// Rebuilds `self` in place, keeping only nodes for which `predicate`
    /// returns `true` (and only edges with both endpoints kept). Bumps
    /// the generation counter.
    ///
    /// # Example
    /// ```
    /// use cnvx_graph::Graph;
    ///
    /// let mut g = Graph::<&str, ()>::undirected();
    /// let a = g.add_node("a");
    /// let b = g.add_node("b");
    /// g.add_edge(a, b, ()).unwrap();
    ///
    /// g.retain_nodes(|_, &name| name != "b");
    /// assert_eq!(g.node_count(), 1);
    /// assert_eq!(g.edge_count(), 0);
    /// ```
    pub fn retain_nodes(&mut self, predicate: impl Fn(NodeId, &N) -> bool) {
        let kept: Vec<NodeId> = self
            .nodes()
            .filter(|&(id, data)| predicate(id, data))
            .map(|(id, _)| id)
            .collect();
        let rebuilt = self.induced_subgraph(&kept);
        *self = rebuilt;
        self.generation = next_generation();
    }

    /// Rebuilds `self` in place, keeping only edges for which `predicate`
    /// returns `true`. Bumps the generation counter.
    pub fn retain_edges(&mut self, predicate: impl Fn(EdgeId, &E) -> bool) {
        let keep: std::collections::HashSet<EdgeId> = self
            .edges()
            .filter(|&(id, data)| predicate(id, data))
            .map(|(id, _)| id)
            .collect();
        self.retain_edges_by_id(|id| keep.contains(&id));
    }

    fn retain_edges_by_id(&mut self, keep: impl Fn(EdgeId) -> bool) {
        let mut out = if self.directed { Graph::directed() } else { Graph::undirected() };
        let mut map = vec![None; self.nodes.len()];
        for (id, data) in self.nodes() {
            map[id.index()] = Some(out.add_node(data.clone()));
        }
        for (id, data) in self.edges() {
            if !keep(id) {
                continue;
            }
            let (a, b) = self.endpoints(id).unwrap();
            let (a2, b2) = (map[a.index()].unwrap(), map[b.index()].unwrap());
            out.add_edge(a2, b2, data.clone()).unwrap();
        }
        *self = out;
        self.generation = next_generation();
    }
}

impl<N: Clone, E: GraphWeight> Graph<N, E> {
    /// Merges `edge`'s two endpoints into one node (keeping `source`'s
    /// data, discarding `target`'s), redirecting every edge that touched
    /// `target` to instead touch `source`, and combining `edge` weights.
    ///
    /// # Example
    /// ```
    /// use cnvx_graph::Graph;
    ///
    /// let mut g = Graph::<&str, f64>::undirected();
    /// let a = g.add_node("a");
    /// let b = g.add_node("b");
    /// let e = g.add_edge(a, b, 5.0).unwrap();
    ///
    /// g.contract_edge(e);
    /// assert_eq!(g.node_count(), 1);
    /// assert_eq!(g.edge_count(), 0);
    /// ```
    pub fn contract_edge(&mut self, edge: EdgeId) {
        let Some((source, target)) = self.endpoints(edge) else { return };
        let mut out = if self.directed { Graph::directed() } else { Graph::undirected() };
        let mut map = vec![None; self.nodes.len()];
        for (id, data) in self.nodes() {
            if id == target {
                continue;
            }
            map[id.index()] = Some(out.add_node(data.clone()));
        }
        map[target.index()] = map[source.index()];
        for (id, data) in self.edges() {
            if id == edge {
                continue;
            }
            let (a, b) = self.endpoints(id).unwrap();
            let (Some(a2), Some(b2)) = (map[a.index()], map[b.index()]) else { continue };
            out.add_edge(a2, b2, data.clone()).unwrap();
        }
        *self = out;
        self.generation = next_generation();
    }

    /// Repeatedly contracts every internal node of a degree-2 chain (a
    /// maximal path of nodes with exactly one edge in and one edge out,
    /// ignoring the chain's two endpoints) into a single edge whose
    /// weight is the chain's [`GraphWeight::accumulate`]-combined total.
    ///
    /// # Example
    /// ```
    /// use cnvx_graph::Graph;
    ///
    /// let mut g = Graph::<&str, f64>::undirected();
    /// let a = g.add_node("a");
    /// let b = g.add_node("b");
    /// let c = g.add_node("c");
    /// g.add_edge(a, b, 3.0).unwrap();
    /// g.add_edge(b, c, 4.0).unwrap();
    ///
    /// g.contract_degree_two_chains();
    /// // `b` had degree 2 and got contracted away, leaving one edge
    /// // a-c with the summed weight.
    /// assert_eq!(g.node_count(), 2);
    /// assert_eq!(g.edge_count(), 1);
    /// let only_edge = g.edge_ids().next().unwrap();
    /// assert_eq!(*g.edge_weight(only_edge).unwrap(), 7.0);
    /// ```
    pub fn contract_degree_two_chains(&mut self) {
        loop {
            let mut contracted_any = false;
            for id in self.node_ids().collect::<Vec<_>>() {
                let is_internal = if self.directed {
                    self.in_degree(id) == 1 && self.out_degree(id) == 1
                } else {
                    self.degree(id) == 2
                };
                if !is_internal {
                    continue;
                }
                let neighbors: Vec<NodeId> = self.neighbors(id).collect();
                if neighbors.len() != 2 || neighbors[0] == id || neighbors[1] == id {
                    continue;
                }
                let edges: Vec<EdgeId> =
                    self.out_edges(id).chain(self.in_edges(id)).collect();
                let mut unique_edges = edges.clone();
                unique_edges.sort_by_key(|e| e.index());
                unique_edges.dedup();
                if unique_edges.len() != 2 {
                    continue;
                }
                let (e1, e2) = (unique_edges[0], unique_edges[1]);
                let (a, _) = self.endpoints(e1).unwrap();
                let other1 = self.other_endpoint(e1, id).unwrap();
                let other2 = self.other_endpoint(e2, id).unwrap();
                let _ = a;
                let combined = self
                    .edge_weight(e1)
                    .unwrap()
                    .accumulate(self.edge_weight(e2).unwrap());

                // Rebuild: drop `id` and its two edges, add one edge
                // directly between its two former neighbors.
                let mut out =
                    if self.directed { Graph::directed() } else { Graph::undirected() };
                let mut map = vec![None; self.nodes.len()];
                for (nid, data) in self.nodes() {
                    if nid == id {
                        continue;
                    }
                    map[nid.index()] = Some(out.add_node(data.clone()));
                }
                for (eid, data) in self.edges() {
                    if eid == e1 || eid == e2 {
                        continue;
                    }
                    let (ea, eb) = self.endpoints(eid).unwrap();
                    let (Some(a2), Some(b2)) = (map[ea.index()], map[eb.index()]) else {
                        continue;
                    };
                    out.add_edge(a2, b2, data.clone()).unwrap();
                }
                let (o1, o2) =
                    (map[other1.index()].unwrap(), map[other2.index()].unwrap());
                out.add_edge(o1, o2, combined).unwrap();
                *self = out;
                self.generation = next_generation();
                contracted_any = true;
                break;
            }
            if !contracted_any {
                break;
            }
        }
    }
}
