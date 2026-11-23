//! Weighted Sum MOP (Scalarization)
//!
//! Minimize the weighted sum 2x + y over x + y >= 8, x, y >= 0
//!
//! Features: `lp`, `mop`

use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("weighted_sum_mop");
    let x = model.add_named_var(0.0.., "x");
    let y = model.add_named_var(0.0.., "y");
    model.add_named_constraint((x + y).geq(8.0), "demand")?;
    model.set_objective(Sense::Minimize, 2.0 * x + 1.0 * y)?;
    model.add_objective(Sense::Minimize, y.into(), 0)?;

    let solution = model.solve(&MopSolver::weighted_sum(vec![2.0, 1.0]))?;

    match solution {
        MopSolution::Single { status, point } => {
            if status != Status::Optimal {
                println!("{}", status)
            }

            let vals: Vec<_> = point.objective_values().map(|(_, v)| v).collect();
            println!("x={} y={}", vals[0], vals[1]);
        }
        _ => println!("unexpected solution type"),
    }

    // Expected output:
    //
    // x=8 y=8
    //
    Ok(())
}
