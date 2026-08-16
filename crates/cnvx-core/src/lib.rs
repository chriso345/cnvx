//! # CNVX Core
//!
//! This crate provides the core types and abstractions for defining and solving
//! optimization problems. It is independent of any particular solver
//! implementation (e.g., simplex, interior point, shortest path problems), and
//! contains the building blocks for variables, constraints, objectives, and
//! solutions.
//!
//! # Modules
//!
//! - [`var`]: Handle types ([`Var`], [`Con`]) and variable typing
//!   ([`VarKind`]).
//! - [`expr`]: Linear expressions ([`Expression`]), built with operator
//!   overloading, plus the [`sum`] helper.
//! - [`constraint`]: Comparisons on an [`Expression`] ([`Constraint`],
//!   [`ConstraintKind`]).
//! - [`model`]: [`Model`], the pure problem-definition type.
//! - [`solve`]: [`Solve`], the trait every problem/solver pair in the workspace
//!   implements.
//! - [`bounds`]: Variable bounds ([`Bounds`]).
//! - [`sense`]: Optimization senses ([`Sense`]) such as
//!   [`Minimize`](Sense::Minimize) or [`Maximize`](Sense::Maximize).
//! - [`status`]: Solver statuses ([`Status`]) such as
//!   [`Optimal`](Status::Optimal) or [`Infeasible`](Status::Infeasible).
//! - [`mod@env`]: Shared solve configuration ([`Env`], [`Params`]).
//! - [`error`]: The shared error type ([`CnvxError`]).

pub mod bounds;
pub mod constraint;
pub mod env;
pub mod error;
pub mod expr;
pub mod model;
pub mod sense;
pub mod solve;
pub mod status;
pub mod var;

// Re-export all submodules for easy access via `cnvx_core::*`
pub use bounds::*;
pub use constraint::*;
pub use env::*;
pub use error::*;
pub use expr::*;
pub use model::*;
pub use sense::*;
pub use solve::*;
pub use status::*;
pub use var::*;
