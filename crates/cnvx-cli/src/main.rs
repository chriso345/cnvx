use cnvx::prelude::*;

fn main() -> Result<(), SolveError> {
    println!("CNVX version {}", cnvx::version());

    let mut model = Model::new();

    let x1 = model.add_var().finish();
    let x2 = model.add_var().finish();
    let x3 = model.add_var().finish();

    // Maximize Z = 3*x1 + 5*x2 + 2*x3
    model.add_objective(Objective::maximize(3.0 * x1 + 5.0 * x2 + 2.0 * x3).name("Z"));

    // Constraints:
    model += (x1 + 2.0 * x2 + (-1.0) * x3).eq(4.0);
    model += (2.0 * x1 + (-1.0) * x2 + 3.0 * x3).leq(10.0);
    model += (-1.0 * x1 + x2 + x3).geq(2.0);

    let solver = SimplexSolver::default();
    let sol = solver.solve(&model)?;

    println!("{}", sol);
    println!("x1 = {}, x2 = {}, x3 = {}", sol.value(x1), sol.value(x2), sol.value(x3));

    Ok(())
}
