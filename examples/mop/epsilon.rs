//! Epsilon-Constraint MOP (Bounded Second Objective)
//!
//! Minimize x subject to y <= 2 and x + y >= 4 (feasible set with cutoff)
//!
//! Features: `lp`, `mop`

use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("epsilon_mop");
    let x = model.add_named_var(0.0.., "x");
    let y = model.add_named_var(0.0.., "y");
    model.add_named_constraint((x + y).geq(4.0), "demand")?;
    model.add_named_constraint(y.leq(2.0), "cap_y")?;

    model.set_objective(Sense::Minimize, x.into())?;
    model.add_objective(Sense::Minimize, y.into(), 0)?;

    let ids: Vec<_> = model.objectives().map(|(id, _)| id).collect();
    let (obj_x, obj_y) = (ids[0], ids[1]);

    let solution =
        model.solve(&MopSolver::epsilon_constraint(obj_x, vec![(obj_y, 2.0)]))?;

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
    // x=2 y=2
    //
    Ok(())
}
