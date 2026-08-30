//! Dijkstra's algorithm for shortest paths, applied to a simple road network.
//!
//! Features: `graph`

use cnvx_core::Solve;
use cnvx_graph::Graph;
use cnvx_graph::shortest_path::Dijkstra;

struct Road {
    distance_km: f64,
    time_minutes: f64,
    toll_dollars: f64,
}

fn main() {
    let mut g = Graph::<&str, Road>::directed();
    let springfield = g.add_node("Springfield");
    let shelbyville = g.add_node("Shelbyville");
    let capital_city = g.add_node("Capital City");
    let ogdenville = g.add_node("Ogdenville");

    g.add_edge(
        springfield,
        shelbyville,
        Road {
            distance_km: 40.0,
            time_minutes: 35.0,
            toll_dollars: 0.0,
        },
    )
    .unwrap();
    g.add_edge(
        shelbyville,
        capital_city,
        Road {
            distance_km: 90.0,
            time_minutes: 60.0,
            toll_dollars: 8.0,
        },
    )
    .unwrap();
    g.add_edge(
        springfield,
        ogdenville,
        Road {
            distance_km: 55.0,
            time_minutes: 40.0,
            toll_dollars: 0.0,
        },
    )
    .unwrap();
    g.add_edge(
        ogdenville,
        capital_city,
        Road {
            distance_km: 60.0,
            time_minutes: 90.0,
            toll_dollars: 0.0,
        },
    )
    .unwrap();

    // The same graph, queried three different ways -- only the weight
    // accessor closure changes.
    let by_distance = (&g)
        .solve(&Dijkstra::new(springfield).weight(|r: &Road| r.distance_km))
        .unwrap();
    let by_time = (&g)
        .solve(&Dijkstra::new(springfield).weight(|r: &Road| r.time_minutes))
        .unwrap();
    let by_toll = (&g)
        .solve(&Dijkstra::new(springfield).weight(|r: &Road| r.toll_dollars))
        .unwrap();

    println!("Springfield -> Capital City:");
    println!(
        "  shortest by distance: {:.0} km",
        by_distance.distance_to(capital_city).unwrap()
    );
    println!(
        "  shortest by time:     {:.0} minutes",
        by_time.distance_to(capital_city).unwrap()
    );
    println!(
        "  cheapest by toll:     ${:.0}",
        by_toll.distance_to(capital_city).unwrap()
    );

    let path = by_distance.node_path_to(&g, capital_city).unwrap();
    let names: Vec<&str> = path.iter().map(|&n| *g.node_weight(n).unwrap()).collect();
    println!("  fastest-distance route: {}", names.join(" -> "));

    // Expected output:
    //
    // Springfield -> Capital City:
    //   shortest by distance: 115 km
    //   shortest by time:     95 minutes
    //   cheapest by toll:     $0
    //   fastest-distance route: Springfield -> Ogdenville -> Capital City
}
