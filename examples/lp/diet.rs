//! Diet Problem
//!
//! Minimize the cost of buying bread and milk while satisfying
//! a minimum protein requirement and a shelf-space limit.
//!
//! Category: Linear Programming

use cnvx_core::{Model, Sense, Solve};
use cnvx_lp::LpSolver;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("diet_problem");

    // Two foods:
    // - Bread costs $2 per unit.
    // - Milk costs $3 per unit.
    //
    // Both foods can be purchased in any nonnegative amount.
    let bread = model.add_named_var(0.0.., "bread");
    let milk = model.add_named_var(0.0.., "milk");

    // Minimize the total cost of the diet.
    model.set_objective(Sense::Minimize, 2.0 * bread + 3.0 * milk)?;

    // Protein requirement:
    // - 1 unit of bread provides 1 unit of protein.
    // - 1 unit of milk provides 2 units of protein.
    //
    // The diet must provide at least 5 units of protein.
    let protein = model.add_named_constraint((bread + 2.0 * milk).geq(5.0), "protein")?;

    // Shelf-space limit:
    // Bread and milk together may occupy at most 20 units
    // of shelf space.
    model.add_named_constraint((bread + milk).leq(20.0), "shelf_space")?;

    let solution = model.solve(&LpSolver::primal_simplex())?;

    println!("status: {}", solution.status);
    println!("cost: {}", solution.objective);
    println!("bread: {}", solution.value(bread));
    println!("milk: {}", solution.value(milk));
    println!("protein shadow price: {}", solution.dual(protein));

    // Expected output:
    //
    // status: Optimal
    // cost: 7.5
    // bread: 0
    // milk: 2.5
    // protein shadow price: 1.5

    Ok(())
}
