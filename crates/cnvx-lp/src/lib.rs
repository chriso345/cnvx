//! # CNVX LP
//!
//! Linear programming solvers for [`cnvx_core::Model`].
//!
//! # Status
//!
//! - [`LpMethod::PrimalSimplex`] (a two-phase, bounded-variable-aware tableau
//!   simplex) is fully implemented; it is what [`LpSolver::default`] dispatches
//!   to.
//! - [`LpMethod::DualSimplex`] and [`LpMethod::InteriorPoint`] are not yet
//!   implemented; solving with either returns
//!   `Err(cnvx_core::CnvxError::Numerical)`.
//!
//! # Examples
//!
//! ```rust
//! # use cnvx_core::{Model, Solve};
//! # use cnvx_lp::LpSolver;
//! let mut model = Model::new("diet_problem");
//! let x = model.add_var(0.0..);
//! let y = model.add_var(0.0..=10.0);
//! model.set_objective(cnvx_core::Sense::Minimize, 2.0 * x + 3.0 * y)?;
//! model.add_constraint((x + y).geq(5.0))?;
//!
//! let solution = model.solve(&LpSolver::default())?;
//! println!("optimal objective: {}", solution.objective);
//! # Ok::<(), cnvx_core::CnvxError>(())
//! ```

mod basis;
mod method;
mod simplex;
mod solution;
mod solve;
mod solver;
mod standard_form;

pub use basis::{Basis, VarStatus};
pub use method::LpMethod;
pub use solution::{CostRange, LpSolution, RhsRange};
pub use solver::LpSolver;
