//! # CNVX MILP
//!
//! Mixed-integer linear programming: branch-and-bound on top of
//! [`cnvx_lp`]'s primal simplex.
//!
//! # Examples
//!
//! General branch-and-bound, a small mixed-integer program:
//!
//! ```rust
//! # use cnvx_core::{Model, Sense, Solve};
//! # use cnvx_milp::MilpSolver;
//! let mut model = Model::new("production_plan");
//! let batches = model.add_integer(0.0.., "batches"); // whole batches only
//! let extra = model.add_var(0.0..); // continuous top-up
//!
//! model.set_objective(Sense::Minimize, 5.0 * batches + 8.0 * extra)?;
//! model.add_constraint((10.0 * batches + extra).geq(37.0))?;
//!
//! let solution = model.solve(&MilpSolver::branch_and_bound())?;
//! println!("cost: {}, batches: {}", solution.objective, solution.value(batches));
//! # Ok::<(), cnvx_core::CnvxError>(())
//! ```

pub mod branch_and_bound;
mod method;
mod solution;
mod solve;
mod solver;

pub use method::MilpMethod;
pub use solution::MilpSolution;
pub use solver::MilpSolver;
