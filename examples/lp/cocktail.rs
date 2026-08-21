//! Cocktail Blending LP
//!
//! Maximize profit from producing two cocktails:
//! - Mojito
//! - Margarita
//!
//! Subject to limited supplies of rum and tequila and a bartender
//! capacity limit.
//!
//! Features: `lp`

use cnvx_core::{Model, Sense, Solve};
use cnvx_lp::LpSolver;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("cocktail_blending");

    // Mojitos and margaritas can be produced in any nonnegative amount.
    let mojito = model.add_named_var(0.0.., "Mojito");
    let margarita = model.add_named_var(0.0.., "Margarita");

    // Resource constraints.
    //
    // Rum available: 100 units.
    // Each mojito uses 2 units of rum.
    model.add_named_constraint((2.0 * mojito).leq(100.0), "rum")?;

    // Tequila available: 80 units.
    // Each margarita uses 4 units of tequila.
    model.add_named_constraint((4.0 * margarita).leq(80.0), "tequila")?;

    // The bartender can make at most 60 cocktails total.
    model.add_named_constraint((mojito + margarita).leq(60.0), "bartender_capacity")?;

    // Profit:
    // Mojito     = $8
    // Margarita  = $10
    model.set_objective(Sense::Maximize, 8.0 * mojito + 10.0 * margarita)?;

    let solution = model.solve(&LpSolver::primal_simplex())?;

    println!("status: {}", solution.status);
    println!("profit: {}", solution.objective);
    println!("Mojitos: {}", solution.value(mojito));
    println!("Margaritas: {}", solution.value(margarita));

    // Expected output:
    //
    // status: Optimal
    // profit: 520
    // Mojitos: 40
    // Margaritas: 20

    Ok(())
}
