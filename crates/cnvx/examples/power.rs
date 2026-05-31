//! Power grid dispatch optimization (PURE LINEAR PROGRAM).
//!
//! Minimize cost of producing electricity while meeting demand:
//! - Gas plant (flexible, medium cost)
//! - Coal plant (high emissions, higher cost)
//! - Wind farm (free but limited capacity)
//!
//! Subject to:
//! - Exact demand satisfaction
//! - Emissions cap
//! - Minimum thermal generation requirement

use cnvx::prelude::*;
use cnvx_core::solver::Solver;
use cnvx_lp::LpSolver;

fn main() {
    let mut model = Model::new();

    let gas = model
        .add_var()
        .name("Gas")
        .lower_bound(0.0)
        .upper_bound(200.0)
        .finish();

    let coal = model
        .add_var()
        .name("Coal")
        .lower_bound(0.0)
        .upper_bound(180.0)
        .finish();

    let wind = model
        .add_var()
        .name("Wind")
        .lower_bound(0.0)
        .upper_bound(120.0)
        .finish();

    // Total electricity generation must exactly match demand (300 MW)
    model += (gas + coal + wind).eq(300.0);

    // Coal produces 2 units of emissions per MW, capped at 200
    model += (coal * 2.0).leq(200.0);

    // At least 150 MW must come from thermal generation (gas + coal)
    model += (gas + coal).geq(150.0);

    // Gas = $50/MW, Coal = $80/MW, Wind = $0/MW
    model.add_objective(
        Objective::minimize(gas * 50.0 + coal * 80.0 + wind * 0.0).name("TotalCost"),
    );

    // -------------------------
    // Solve LP
    // -------------------------
    let mut solver = LpSolver::new();

    if let Some(name) = solver.selected_for(&model) {
        println!("Selected solver: {name}");
    }

    let solution = solver.solve(&model).unwrap();

    println!("Optimal cost: {}", solution.objective_value.unwrap_or(0.0));
    println!("Gas generation: {}", solution.value(gas));
    println!("Coal generation: {}", solution.value(coal));
    println!("Wind generation: {}", solution.value(wind));

    // Expected behavior:
    //
    // - Wind is used as much as possible (free resource)
    // - Gas and coal satisfy minimum thermal constraint
    // - Coal is limited by emissions cap
}
