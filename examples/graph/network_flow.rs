//! Network Flow Optimisation
//!
//! Features: `graph`

use cnvx::prelude::*;

fn main() {
    let mut g = Graph::<&str, f64>::directed();
    let source = g.add_node("Reservoir");
    let a = g.add_node("Station A");
    let b = g.add_node("Station B");
    let sink = g.add_node("City");

    g.add_edge(source, a, 10.0).unwrap();
    g.add_edge(source, b, 5.0).unwrap();
    g.add_edge(a, b, 15.0).unwrap();
    g.add_edge(a, sink, 10.0).unwrap();
    g.add_edge(b, sink, 10.0).unwrap();

    let result = (&g)
        .solve(&MaxFlow::new(source, sink).capacity(|c: &f64| *c))
        .unwrap();
    println!("Maximum flow from Reservoir to City: {:.0} units/hour", result.objective());

    println!("\nMinimum cut (the bottleneck pipes):");
    for e in g.edge_ids() {
        let (u, v) = g.endpoints(e).unwrap();
        if result.is_source_side(u) && !result.is_source_side(v) {
            let (uname, vname) = (g.node_weight(u).unwrap(), g.node_weight(v).unwrap());
            println!(
                "  {uname} -> {vname} (capacity {:.0}, fully used)",
                g.edge_weight(e).unwrap()
            );
        }
    }

    // Expected output:
    //
    // Maximum flow from Reservoir to City: 15 units/hour
    //
    // Minimum cut (the bottleneck pipes):
    //   Reservoir -> Station A (capacity 10, fully used)
    //   Reservoir -> Station B (capacity 5, fully used)
}
