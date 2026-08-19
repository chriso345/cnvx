//! Traveling Salesperson Problem
//!
//! Find the minimum-cost tour that:
//! - Visits every city exactly once
//! - Returns to the starting city
//! - Minimizes the total travel distance
//!
//! Subject to:
//! - Exactly one edge leaves each city
//! - Exactly one edge enters each city
//! - Subtours are eliminated using MTZ constraints
//!
//! Category: Mixed-Integer Linear Programming

use cnvx_core::{Model, Sense, Solve, sum};
use cnvx_milp::MilpSolver;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("tsp");

    // Distance matrix between the five cities.
    //
    //       A   B   C   D   E
    // A     0  12  10  19   8
    // B    12   0   3   7   6
    // C    10   3   0   6   9
    // D    19   7   6   0   5
    // E     8   6   9   5   0
    let distances = [
        [0.0, 12.0, 10.0, 19.0, 8.0],
        [12.0, 0.0, 3.0, 7.0, 6.0],
        [10.0, 3.0, 0.0, 6.0, 9.0],
        [19.0, 7.0, 6.0, 0.0, 5.0],
        [8.0, 6.0, 9.0, 5.0, 0.0],
    ];

    let cities = ["A", "B", "C", "D", "E"];
    let n = cities.len();

    // x[i][j] = 1 if the tour travels directly from city i to city j.
    let mut x = Vec::with_capacity(n);

    for i in 0..n {
        let mut row = Vec::with_capacity(n);

        for j in 0..n {
            row.push(model.add_binary(&format!("x_{i}_{j}")));
        }

        x.push(row);
    }

    // A city cannot travel directly to itself.
    for (i, row) in x.iter().enumerate() {
        model.add_named_constraint(row[i].eq(0.0), &format!("no_self_loop_{i}"))?;
    }

    // Exactly one edge must leave every city.
    for (i, row) in x.iter().enumerate() {
        let outgoing =
            sum(row.iter().enumerate().filter(|&(j, _)| j != i).map(|(_, &xij)| xij));

        model.add_named_constraint(outgoing.eq(1.0), &format!("one_departure_{i}"))?;
    }

    // Exactly one edge must enter every city.
    for (j, _) in x.iter().enumerate().take(n) {
        let incoming = sum((0..n).filter(|&i| i != j).map(|i| x[i][j]));

        model.add_named_constraint(incoming.eq(1.0), &format!("one_arrival_{j}"))?;
    }

    // MTZ ordering variables.
    //
    // City A is the fixed starting city.
    // The remaining cities receive positions 1..n-1.
    let mut order = Vec::with_capacity(n);

    order.push(model.add_named_var(0.0..=0.0, "u_0"));

    for i in 1..n {
        order.push(model.add_named_var(1.0..=(n - 1) as f64, &format!("u_{i}")));
    }

    // MTZ subtour elimination constraints.
    //
    // For every pair of non-starting cities i and j:
    //
    //     u_i - u_j + (n - 1) * x[i][j] <= n - 2
    //
    // If the tour travels from i to j, then j must appear after i
    // in the ordering. This prevents disconnected subtours.
    for i in 1..n {
        for j in 1..n {
            if i != j {
                model.add_named_constraint(
                    (order[i] - order[j] + (n as f64 - 1.0) * x[i][j])
                        .leq(n as f64 - 2.0),
                    &format!("mtz_{i}_{j}"),
                )?;
            }
        }
    }

    // Minimize the total distance traveled.
    let mut objective_terms = Vec::new();

    for i in 0..n {
        for j in 0..n {
            if i != j {
                objective_terms.push(distances[i][j] * x[i][j]);
            }
        }
    }

    model.set_objective(Sense::Minimize, sum(objective_terms))?;

    let solution = model.solve(&MilpSolver::branch_and_bound())?;

    println!("status: {}", solution.status);
    println!("total distance: {:.2}", solution.objective);

    // Reconstruct the tour starting from city A.
    let xv: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| solution.value(x[i][j])).collect())
        .collect();

    let mut current = 0;
    let mut tour = vec![current];
    let mut visited = vec![false; n];

    visited[current] = true;

    while tour.len() < n {
        let next = (0..n)
            .find(|&j| !visited[j] && xv[current][j] > 0.5)
            .expect("TSP solution should contain a valid tour");

        tour.push(next);
        visited[next] = true;
        current = next;
    }

    tour.push(0);

    print!("tour: ");

    for (index, &city) in tour.iter().enumerate() {
        if index > 0 {
            print!(" -> ");
        }

        print!("{}", cities[city]);
    }

    println!();

    // Expected output:
    //
    // status: Optimal
    // total distance: 33.00
    // tour: A -> E -> D -> B -> C -> A

    Ok(())
}
