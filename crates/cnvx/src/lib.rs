//! # CNVX
//!
//! This crate provides a unified interface for the CNVX optimization library,
//! re-exporting functionality from [`cnvx_core`] and [`cnvx_lp`].
//!
//! `cnvx` allows you to define optimization models, constraints, objectives, and
//! solve linear programming (LP) problems using solvers such as the simplex method,
//! all through a single crate.
//!
//! # Features
//!
//! - Unified interface for core modeling and LP solvers.
//! - Easy access to core types, constraints, variables, and objectives via [`prelude`].
//! - LP solvers accessible via [`solvers`].
//! - Versioning information via [`version`].
//!
//! # Modules
//!
//! - [`prelude`]: Re-exports the main types and functions from [`cnvx_core`] and [`cnvx_lp`] for convenient usage.
//! - [`solvers`]: Contains LP solvers enabled by features, such as the [`PrimalSimplexSolver`](::cnvx_lp::PrimalSimplexSolver) and [`DualSimplexSolver`](::cnvx_lp::DualSimplexSolver).
//!
//! # Examples
//!
//! ```rust
//! use cnvx::prelude::*;
//! use cnvx::solvers::PrimalSimplexSolver;
//!
//! // Create a model
//! let mut model = Model::new();
//! let x = model.add_var().finish();
//! let y = model.add_var().finish();
//!
//! // Add constraints
//! model+= (x + y).leq(5.0);
//! model+= (x + 0.5 * y).geq(10.0);
//!
//!
//! // Set objective
//! model.add_objective(Objective::maximize(x + 2.0 * y).name("Z"));
//!
//! // Solve using the simplex solver
//! let solver = PrimalSimplexSolver::default();
//! let solution = solver.solve(&model).unwrap();
//!
//! println!("Optimal solution: x = {}, y = {}", solution.value(x), solution.value(y));
//! ```
//!
//! # Version
//!
//! Retrieve the current version of the `cnvx` crate using [`version`]:
//!
//! ```rust
//! println!("CNVX version: {}", cnvx::version());
//! ```

pub use cnvx_core as core;
pub mod auto_solver;

#[cfg(feature = "lp")]
pub use cnvx_lp as lp;

pub mod prelude {
    pub use crate::auto_solver::AutoSolver;
    pub use crate::core::*;

    #[cfg(feature = "lp")]
    pub use crate::lp::*;
}

pub mod solvers {
    #[cfg(feature = "lp")]
    pub use crate::lp::{DualSimplexSolver, LpAutoSolver};
}

/// Returns the version of the `cnvx` crate.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
