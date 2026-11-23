//! Lexicographic MOP (Weighted Priority)
//!
//! Lexicographically minimize [x, y] for a simple 2D feasible region.
//!
//! Features: `lp`, `mop`

use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("lexicographic_mop");
    let x = model.add_named_var(0.0.., "x");
    let y = model.add_named_var(0.0.., "y");
    model.add_named_constraint((x + y).geq(6.0), "demand")?;
    model.set_objective(Sense::Minimize, x.into())?;
    model.add_objective(Sense::Minimize, y.into(), 0)?;

    let solution = model.solve(&MopSolver::lexicographic())?;

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
    // x=6 y=0
    //
    Ok(())
}
