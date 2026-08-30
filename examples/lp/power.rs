//! Power Grid Dispatch Optimization
//!
//! Minimize the cost of producing electricity while meeting demand:
//! - Gas plant (flexible, medium cost)
//! - Coal plant (high emissions, higher cost)
//! - Wind farm (free but limited capacity)
//!
//! Subject to:
//! - Exact demand satisfaction
//! - Emissions cap
//! - Minimum thermal generation requirement
//!
//! Features: `lp`

use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("power_grid_dispatch");

    let gas = model.add_named_var(0.0..=200.0, "Gas");
    let coal = model.add_named_var(0.0..=180.0, "Coal");
    let wind = model.add_named_var(0.0..=120.0, "Wind");

    // Total electricity generation must exactly match demand: 300 MW.
    model.add_named_constraint((gas + coal + wind).eq(300.0), "demand")?;

    // Gas emissions must be less than or equal to half of coal emissions.
    model.add_named_constraint(gas.leq(0.5 * coal), "emissions")?;

    // At least 150 MW must come from thermal generation.
    model.add_named_constraint((gas + coal).geq(150.0), "minimum_thermal")?;

    // Gas = $50/MW, Coal = $80/MW, Wind = $0/MW.
    model.set_objective(Sense::Minimize, 50.0 * gas + 80.0 * coal + 0.0 * wind)?;

    let solution =
        model.solve(&LpSolver::primal_simplex().time_limit(1.5).max_iterations(10))?;

    println!("status: {}", solution.status);
    println!("cost: {:.2}", solution.objective);
    println!("Gas generation: {:.2}", solution.value(gas));
    println!("Coal generation: {:.2}", solution.value(coal));
    println!("Wind generation: {:.2}", solution.value(wind));

    // Expected output:
    //
    // status: Optimal
    // cost: 12600.00
    // Gas generation: 60.00
    // Coal generation: 120.00
    // Wind generation: 120.00

    Ok(())
}
