//! Knapsack Optimization
//!
//! Maximize the total value of items placed in a knapsack:
//! - Item 0: value 60, weight 10
//! - Item 1: value 100, weight 20
//! - Item 2: value 120, weight 30
//!
//! Subject to:
//! - Each item is either selected or not selected
//! - Total weight must not exceed 50
//!
//! Category: Mixed-Integer Linear Programming

use cnvx_core::{Model, Sense, Solve, sum};
use cnvx_milp::MilpSolver;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("knapsack");

    let values = [60.0, 100.0, 120.0];
    let weights = [10.0, 20.0, 30.0];

    // Each item can either be selected (1) or not selected (0).
    let items: Vec<_> = (0..3).map(|i| model.add_binary(&format!("item{i}"))).collect();

    // Maximize the total value of the selected items.
    let value = sum(items.iter().zip(&values).map(|(&x, &v)| v * x));
    model.set_objective(Sense::Maximize, value)?;

    // The total weight of selected items must not exceed 50.
    let weight = sum(items.iter().zip(&weights).map(|(&x, &w)| w * x));
    model.add_constraint(weight.leq(50.0))?;

    let solution = model.solve(&MilpSolver::binary())?;

    println!("status: {}", solution.status);
    println!("best value: {:.2}", solution.objective);
    println!("Item 0 selected: {:.0}", solution.value(items[0]));
    println!("Item 1 selected: {:.0}", solution.value(items[1]));
    println!("Item 2 selected: {:.0}", solution.value(items[2]));

    // Expected output:
    //
    // status: Optimal
    // best value: 220.00
    // Item 0 selected: 0
    // Item 1 selected: 1
    // Item 2 selected: 1

    Ok(())
}
