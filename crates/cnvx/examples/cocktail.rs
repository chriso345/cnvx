//! Simple cocktail blending LP.
//!
//! Maximize profit from producing two cocktails:
//! - Mojito
//! - Margarita
//!
//! Subject to limited supplies of rum and tequila.

use cnvx::prelude::*;

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

    // Profit:
    // Mojito     = $8
    // Margarita  = $10
    model.add_objective(
        Objective::maximize(mojito * 8.0 + margarita * 10.0).name("Profit"),
    );

    let mut solver = PrimalSimplexSolver::new(&model);
    let solution = solver.solve().unwrap();

    println!("Optimal profit: {}", solution.objective_value.unwrap_or(0.0));

    println!("Mojitos: {}", solution.value(mojito));
    println!("Margaritas: {}", solution.value(margarita));

    // Expected output:
    //
    // Optimal profit: 600
    // Mojitos: 50
    // Margaritas: 20
}
