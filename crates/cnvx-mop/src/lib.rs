//! # CNVX MOP
//!
//! Multi-objective linear and mixed-integer programming for
//! [`cnvx_core::Model`].
//!
//! `cnvx-mop` has no model type of its own: a model with more than one
//! objective is a [`cnvx_core::Model`], built with
//! [`Model::set_objective`](cnvx_core::Model::set_objective) for the
//! primary objective and
//! [`Model::add_objective`](cnvx_core::Model::add_objective) for every
//! additional one.
//!
//! # Methods
//!
//! See [`MopMethod`] for the full list and what each requires. In
//! summary:
//!
//! - [`MopMethod::Lexicographic`], [`MopMethod::EpsilonConstraint`], and
//!   [`MopMethod::WeightedSum`] each solve to a single [`ParetoPoint`]
//!   ([`MopSolution::Single`]), and work on both continuous and mixed-integer
//!   models.
//! - [`MopMethod::Biobjective`] each trace out a [`ParetoFront`]
//!   ([`MopSolution::Front`]), and only work on models where every variable is
//!   continuous.
//!
//! # Examples
//!
//! ```rust
//! # use cnvx_core::{Model, Sense, Solve};
//! # use cnvx_mop::{MopSolution, MopSolver};
//! let mut model = Model::new("bicriteria_split");
//! let x = model.add_var(0.0..);
//! let y = model.add_var(0.0..);
//! model.add_constraint((x + y).eq(10.0))?;
//!
//! model.set_objective(Sense::Minimize, x.into())?;
//! model.add_objective(Sense::Minimize, y.into(), 0)?;
//!
//! let solution = model.solve(&MopSolver::biobjective())?;
//! if let MopSolution::Front(front) = solution {
//!     println!("found {} efficient points", front.len());
//! }
//! # Ok::<(), cnvx_core::CnvxError>(())
//! ```

mod biobjective;
mod epsilon_constraint;
mod lexicographic;
mod method;
mod solution;
mod solve;
mod solver;
mod util;

pub use method::MopMethod;
pub use solution::{MopPointSolution, MopSolution, ParetoFront, ParetoPoint};
pub use solver::MopSolver;
