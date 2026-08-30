//! Production Planning Optimization
//!
//! Minimize the cost of meeting a production requirement:
//! - Production batches are whole units and cost $5 each
//! - Additional production can be purchased continuously at $8 per unit
//!
//! Subject to:
//! - Total production must be at least 37 units
//! - Batches must be whole numbers
//! - Additional production must be non-negative
//!
//! Features: `lp`, `milp`

use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("production_plan");

    // Production must be purchased in whole batches.
    let batches = model.add_integer(0.0.., "Batches");

    // Continuous production can be used to top up the requirement.
    let extra = model.add_named_var(0.0.., "Extra");

    // Whole batches cost $5 each, while additional production costs $8 per unit.
    model.set_objective(Sense::Minimize, 5.0 * batches + 8.0 * extra)?;

    // Total production must meet the required 37 units.
    model.add_named_constraint(
        (10.0 * batches + extra).geq(37.0),
        "production_requirement",
    )?;

    let solution = model.solve(&MilpSolver::branch_and_bound())?;

    println!("status: {}", solution.status);
    println!("cost: {:.2}", solution.objective);
    println!("Batches: {:.0}", solution.value(batches));
    println!("Extra production: {:.2}", solution.value(extra));

    // Expected output:
    //
    // status: Optimal
    // cost: 20.00
    // Batches: 4
    // Extra production: 0.00

    Ok(())
}
