//! # CNVX LP
//!
//! This crate provides linear programming (LP) functionality built on top of
//! [`cnvx_core`]. It implements LP-specific abstractions and solver algorithms,
//! with a focus on the simplex method.
//!
//! # Features
//!
//! - [`PrimalSimplexSolver`]: Solver implementing the 2-phase primal simplex algorithm for LP problems.
//!
//! # Modules
//!
//! - [`primal_simplex`]: Contains the [`PrimalSimplexSolver`] struct and primal simplex-specific solver logic.

pub mod primal_simplex;
pub mod validate;

pub use primal_simplex::*;
