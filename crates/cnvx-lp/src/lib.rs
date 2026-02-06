//! # CNVX LP
//!
//! This crate provides linear programming (LP) functionality built on top of
//! [`cnvx_core`]. It implements LP-specific abstractions and solver algorithms,
//! with a focus on the simplex method.
//!
//! # Features
//!
//! - [`SimplexSolver`]: Solver implementing the simplex algorithm for LP problems.
//!
//! # Modules
//!
//! - [`simplex`]: Contains the [`SimplexSolver`] struct and simplex-specific solver logic.

pub mod simplex;
pub mod validate;

pub use simplex::*;
