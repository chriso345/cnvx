//! Biobjective MOP (Pareto Front)
//!
//! Minimize two objectives over a simple 2D region.
//!
//! Features: `lp`, `mop`

use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("biobjective_example");
    // Variables x, y >= 0
    let x = model.add_named_var(0.0.., "x");
    let y = model.add_named_var(0.0.., "y");
    // x + y >= 3 (forces at least some nonzero, simple feasible region)
    model.add_named_constraint((x + y).geq(3.0), "demand")?;

    // First objective: Minimize x
    model.set_objective(Sense::Minimize, x.into())?;
    // Second objective: Minimize y
    model.add_objective(Sense::Minimize, y.into(), 0)?;

    let solution = model.solve(&MopSolver::biobjective())?;

    match solution {
        MopSolution::Front(front) => {
            println!("front length: {}", front.len());
            for (i, point) in front.points().iter().enumerate() {
                let fv: Vec<_> = point.objective_values().map(|(_, v)| v).collect();
                println!("point {}: x={} y={}", i, fv[0], fv[1]);
            }
        }
        _ => println!("unexpected solution type"),
    }

    // Expected output:
    //
    // front length: 2
    // point 0: x=0 y=3
    // point 1: x=3 y=0
    Ok(())
}
