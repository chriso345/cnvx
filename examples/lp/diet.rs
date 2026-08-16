use cnvx_core::{Model, Sense, Solve};
use cnvx_lp::LpSolver;

fn main() -> Result<(), cnvx_core::CnvxError> {
    let mut model = Model::new("diet_problem");

    // Two foods, cost per unit $2 and $3, any nonnegative amount.
    let bread = model.add_named_var(0.0.., "bread");
    let milk = model.add_named_var(0.0.., "milk");

    model.set_objective(Sense::Minimize, 2.0 * bread + 3.0 * milk)?;

    // Need at least 5 units of protein: 1 unit bread + 2 units milk.
    let protein = model.add_named_constraint((bread + 2.0 * milk).geq(5.0), "protein")?;
    // At most 20 units of a shared shelf-space constraint.
    model.add_named_constraint((bread + milk).leq(20.0), "shelf_space")?;

    let solution = model.solve(&LpSolver::primal_simplex())?;

    println!("status: {}", solution.status);
    println!("cost: {}", solution.objective);
    println!("bread: {}", solution.value(bread));
    println!("milk: {}", solution.value(milk));
    println!("protein shadow price: {}", solution.dual(protein));

    Ok(())
}
