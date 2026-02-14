//! # CNVX LP
//!
//! This crate provides linear programming (LP) functionality built on top of
//! [`cnvx_core`]. It implements LP-specific abstractions and solver algorithms,
//! with a focus on the simplex method.
//!
//! # Features
//!
//! - [`LpAutoSolver`]: Automatically selects the appropriate solver based on the problem characteristics. (TODO)
//! - [`DualSimplexSolver`]: Solver implementing the dual simplex algorithm for LP problems. (TODO)
//! - [`PrimalSimplexSolver`]: Solver implementing the 2-phase primal simplex algorithm for LP problems.
//!
//! # Modules
//!
//! - [`auto`]: Contains the [`LpAutoSolver`] struct, which automatically selects the appropriate solver based on the problem characteristics.
//! - [`dual_simplex`]: Contains the [`DualSimplexSolver`] struct and dual
//! - [`primal_simplex`]: Contains the [`PrimalSimplexSolver`] struct and primal simplex-specific solver logic.

pub mod auto;
pub mod dual_simplex;
pub mod primal_simplex;
pub mod validate;

pub use auto::*;
pub use dual_simplex::*;
pub use primal_simplex::*;
