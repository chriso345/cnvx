use cnvx::prelude::*;

fn main() -> Result<(), SolveError> {
    println!("CNVX version {}", cnvx::version());

    let mut model = Model::new();

    let x = model.add_var().lower_bound(0.0).finish();
    let y = model.add_var().binary().finish();

    model.add_objective(Objective::minimize(3.0 * x + 2.0 * y).name("cost"));

    model += (2.0 * x + y).le(10.0);
    model += (x + 3.0 * y).le(12.0);

    let solver = SimplexSolver::default();
    let sol = solver.solve(&model)?;

    println!("Solution: {:?}", sol);

    Ok(())
}
