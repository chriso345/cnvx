use cnvx::prelude::*;

#[derive(Debug, Clone)]
pub struct EdgeData {
    pub costs: [f64; 3], // 3 objectives per edge
}

#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: usize,
}

fn main() {
    let mut g = AdjacencyGraph::<NodeData, EdgeData> {
        nodes: Vec::new(),
        edges: Vec::new(),
        out_adj: Vec::new(),
        in_adj: Vec::new(),
    };

    for i in 0..5 {
        g.nodes.push(NodeData { id: i });
        g.out_adj.push(Vec::new());
        g.in_adj.push(Vec::new());
    }

    let edges = vec![
        (0, 1, [10, 5, 2]),
        (0, 2, [3, 8, 6]),
        (1, 2, [4, 4, 1]),
        (1, 3, [7, 2, 9]),
        (2, 3, [6, 6, 3]),
        (3, 4, [1, 9, 5]),
        (4, 0, [8, 3, 7]),
    ];

    for (from, to, costs) in edges {
        let edge_id = g.edges.len();

        g.edges.push(EdgeRecord {
            from,
            to,
            data: EdgeData { costs: costs.map(|c| c as f64) },
        });

        g.out_adj[from].push(edge_id);
        g.in_adj[to].push(edge_id);
    }

    println!("Nodes: {}", g.nodes.len());
    println!("Edges: {}", g.edges.len());

    let mut solver = LabelSet::new(&g);
    solver.set_sp_type(SPType::OneToAll(0));
    solver.set_cost_function(|e: &EdgeData| e.costs.to_vec());
    let result = solver.solve();

    for n in 0..g.num_nodes() {
        println!("Node {n}:");
        for (i, label) in result.labels_at(n).iter().enumerate() {
            println!(
                "  path {i}: cost={:?}  nodes={:?}",
                label.cost,
                result.node_path(0, n, i)
            );
        }
    }

    // Expected Output:
    //
    // Nodes: 5
    // Edges: 7
    // Node 0:
    //   path 0: cost=[0.0, 0.0, 0.0]  nodes=[0]
    // Node 1:
    //   path 0: cost=[10.0, 5.0, 2.0]  nodes=[0, 1]
    // Node 2:
    //   path 0: cost=[3.0, 8.0, 6.0]  nodes=[0, 2]
    //   path 1: cost=[14.0, 9.0, 3.0]  nodes=[0, 1, 2]
    // Node 3:
    //   path 0: cost=[9.0, 14.0, 9.0]  nodes=[0, 2, 3]
    //   path 1: cost=[17.0, 7.0, 11.0]  nodes=[0, 1, 3]
    //   path 2: cost=[20.0, 15.0, 6.0]  nodes=[0, 1, 2, 3]
    // Node 4:
    //   path 0: cost=[10.0, 23.0, 14.0]  nodes=[0, 2, 3, 4]
    //   path 1: cost=[18.0, 16.0, 16.0]  nodes=[0, 1, 3, 4]
    //   path 2: cost=[21.0, 24.0, 11.0]  nodes=[0, 1, 2, 3, 4]
}
