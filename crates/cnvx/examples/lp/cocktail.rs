//! Cocktail Blending LP
//!
//! Maximize profit from producing two cocktails:
//! - Mojito
//! - Margarita
//!
//! Subject to limited supplies of rum and tequila.
//!
//! Category: Linear Programming

use cnvx::prelude::*;
use cnvx_core::solver::Solver;
use cnvx_lp::LpSolver;

fn main() {
    let mut model = Model::new();

    let mojito = model.add_var().name("Mojito").lower_bound(0.0).integer().finish();
    let margarita = model.add_var().name("Margarita").lower_bound(0.0).integer().finish();

    // Resource constraints
    //
    // Rum available: 100 units
    // Mojito uses 2 rum
    model += (mojito * 2.0).leq(100.0);

    // Tequila available: 80 units
    // Margarita uses 4 tequila
    model += (margarita * 4.0).leq(80.0);

    // Bartender is only able to make 60 cocktails total
    model += (mojito + margarita).leq(60.0);

    // Profit:
    // Mojito     = $8
    // Margarita  = $10
    model.add_objective(
        Objective::maximize(mojito * 8.0 + margarita * 10.0).name("Profit"),
    );

    let mut solver = LpSolver::new();
    if let Some(name) = solver.selected_for(&model) {
        println!("Selected solver: {name}");
    }

    let solution = solver.solve(&model).unwrap();

    println!("Optimal profit: {}", solution.objective_value.unwrap_or(0.0));
    println!("Mojitos: {}", solution.value(mojito));
    println!("Margaritas: {}", solution.value(margarita));

    // Expected output:
    //
    // Selected solver: primal-simplex
    // Optimal profit: 520
    // Mojitos: 40
    // Margaritas: 20
}
