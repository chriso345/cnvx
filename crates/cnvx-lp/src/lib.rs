//! # CNVX LP
//!
//! This crate provides linear programming (LP) functionality built on top of
//! [`cnvx_core`]. It implements LP-specific abstractions and solver algorithms,
//! with a focus on the simplex method.
//!
//! # Features
//!
//! - [`LpSolver`]: A high-level solver that automatically selects the appropriate LP algorithm based on the problem characteristics.
//! - [`DualSimplexSolver`]: Solver implementing the dual simplex algorithm for LP problems. (TODO)
//! - [`PrimalSimplexSolver`]: Solver implementing the 2-phase primal simplex algorithm for LP problems.
//!
//! # Modules
//!
//! - [`lp_solver`]: Contains the [`LpSolver`] struct, which automatically selects the appropriate LP solver based on the problem characteristics.
//! - [`dual_simplex`]: Contains the [`DualSimplexSolver`] struct and dual
//! - [`primal_simplex`]: Contains the [`PrimalSimplexSolver`] struct and primal simplex-specific solver logic.

pub mod dual_simplex;
pub mod lp_solver;
pub mod primal_simplex;
pub mod validate;

pub use dual_simplex::*;
pub use lp_solver::*;
pub use primal_simplex::*;

pub mod core;
pub use core::*;
