//! # CNVX Core
//!
//! This crate provides the core types and abstractions for defining and solving
//! optimization problems. It is independent of any particular solver implementation
//! (e.g., simplex, interior point), and contains the building blocks for variables,
//! constraints, objectives, and solutions.
//!
//! # Modules
//!
//! - [`constraint`]: Defines linear constraints and comparison operators ([`EQ`](Cmp::EQ), [`LEQ`](Cmp::LEQ), [`GEQ`](Cmp::GEQ)).
//! - [`expr`]: Linear expressions ([`LinExpr`]) and terms ([`LinTerm`]) for building objectives and constraints.
//! - [`model`]: The [`Model`] struct, containing variables, constraints, and objectives.
//! - [`objective`]: Objective functions ([`Objective`]) and builder API.
//! - [`solution`]: Solution results ([`Solution`]) and methods for accessing variable values.
//! - [`solver`]: The [`Solver`] trait for solver implementations.
//! - [`status`]: Solver statuses ([`SolveStatus`]) such as [`Optimal`](SolveStatus::Optimal) or [`Infeasible`](SolveStatus::Infeasible).
//! - [`var`]: Variable types ([`Var`], [`VarId`]) and builder API ([`VarBuilder`]).

// TODO: Consider moving LP-specific logic into `cnvx-core/lp`
// to allow non-LP models (e.g., SAT or other problem types) to remain separate.

pub mod problem;
pub mod sense;
pub mod status;

// Re-export all submodules for easy access via `cnvx_core::*`
pub use problem::*;
pub use sense::*;
pub use status::*;
