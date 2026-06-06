//! # CNVX Core
//!
//! This crate provides the core types and abstractions for defining and solving
//! optimization problems. It is independent of any particular solver implementation
//! (e.g., simplex, interior point, shortest path problems), and contains the building
//! blocks for variables, constraints, objectives, and solutions.
//!
//! # Modules
//!
//! - [`sense`]: Optimization senses ([`Sense`]) such as [`Minimize`](Sense::Minimize) or [`Maximize`](Sense::Maximize).
//! - [`status`]: Solver statuses ([`SolveStatus`]) such as [`Optimal`](SolveStatus::Optimal) or [`Infeasible`](SolveStatus::Infeasible).

pub mod sense;
pub mod status;

// Re-export all submodules for easy access via `cnvx_core::*`
pub use sense::*;
pub use status::*;
