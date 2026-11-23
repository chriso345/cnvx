//! Multi-criteria routing
//!
//! Features: `graph`

use cnvx::prelude::*;

struct Road {
    distance_km: f64,
    toll_dollars: f64,
}

fn main() {
    let mut g = Graph::<&str, Road>::directed();
    let start = g.add_node("Start");
    let end = g.add_node("End");

    // Three genuinely different routes: none dominates the other two.
    g.add_edge(start, end, Road { distance_km: 100.0, toll_dollars: 0.0 })
        .unwrap(); // free, long
    g.add_edge(start, end, Road { distance_km: 60.0, toll_dollars: 12.0 })
        .unwrap(); // fast, pricey
    g.add_edge(start, end, Road { distance_km: 80.0, toll_dollars: 4.0 })
        .unwrap(); // a middle option

    // 1. Full Pareto front: every non-dominated route.
    let pareto = (&g)
        .solve(
            &ParetoShortestPath::new(start).criteria(|r: &Road| {
                Vector::from_slice(&[r.distance_km, r.toll_dollars])
            }),
        )
        .unwrap();
    println!("Pareto-optimal routes (distance km, toll $):");
    for path in pareto.pareto_front(end) {
        println!("  {:?}", path.cost().to_vec());
    }

    // 2. Weighted-sum scalarization: pick a single trade-off in advance.
    let weight = scalarize_weighted_sum(
        |r: &Road| Vector::from_slice(&[r.distance_km, r.toll_dollars]),
        Vector::from_slice(&[1.0, 5.0]), // a dollar of toll "costs" as much as 5 km
    );
    let scalarized = (&g).solve(&Dijkstra::new(start).weight(weight)).unwrap();
    println!(
        "\nWeighted-sum optimal combined cost: {:.1}",
        scalarized.distance_to(end).unwrap()
    );

    // 3. Lexicographic: minimize toll first; among equal-toll routes,
    // minimize distance.
    let lexicographic = (&g)
        .solve(
            &LexicographicShortestPath::new(start)
                .criteria(|r: &Road| Vector::from_slice(&[r.toll_dollars, r.distance_km]))
                .priorities(vec![LexPriority::Min, LexPriority::Min]),
        )
        .unwrap();
    println!(
        "\nLexicographic (toll first) optimal: {:?}",
        lexicographic.criteria_to(end).unwrap().to_vec()
    );

    // Expected output:
    //
    // Pareto-optimal routes (distance km, toll $):
    //   [60.0, 12.0]
    //   [80.0, 4.0]
    //   [100.0, 0.0]
    //
    // Weighted-sum optimal combined cost: 100.0
    //
    // Lexicographic (toll first) optimal: [0.0, 100.0]
}
