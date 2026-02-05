use cnvx::prelude::*;

fn main() -> Result<(), SolveError> {
    println!("CNVX version {}", cnvx::version());

    let mut model = Model::new();

    // let x = model.add_var().lower_bound(0.0).finish();
    // let y = model.add_var().binary().finish();
    let x = model.add_var().finish();
    let y = model.add_var().finish();

    model.add_objective(Objective::maximize(3.0 * x + 2.0 * y).name("profit"));

    model += (x + y).eq(4.0);
    model += (2.0 * x + 3.0 * y).eq(9.0);

    let solver = SimplexSolver::default();
    let sol = solver.solve(&model)?;

    println!(
        "Solution: x = {}, y = {}, objective = {}",
        sol.value(x),
        sol.value(y),
        sol.objective_value.unwrap_or_default()
    );

    Ok(())
}
