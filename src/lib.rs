//! # CNVX
//!
//! This crate provides a unified interface for the CNVX optimization library,
//! re-exporting functionality from [`cnvx_core`] and [`cnvx_lp`].
//!
//! `cnvx` allows you to define optimization models, constraints, objectives,
//! and solve linear programming (LP) problems using solvers such as the simplex
//! method, all through a single crate.
//!
//! # Examples
//!
//! ```rust
//! use cnvx::prelude::*;
//!
//! // Create a model
//! let mut model = Model::new("Z");
//! let x = model.add_var(..);
//! let y = model.add_var(..);
//!
//! // Add constraints
//! model.add_constraint((x + y).leq(5.0)).unwrap();
//! model.add_constraint((x + 0.5 * y).geq(10.0)).unwrap();
//!
//! // Set objective
//! model.set_objective(Sense::Maximize, x + 2.0 * y).unwrap();
//!
//! // Solve using the simplex solver
//! let solution = model.solve(&LpSolver::default()).unwrap();
//!
//! println!("Optimal objective: {}", solution.objective);
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
#[cfg(feature = "lp")]
pub use cnvx_lp as lp;
pub use cnvx_math as math;

pub mod prelude {
    pub use crate::core::*;
    #[cfg(feature = "lp")]
    pub use crate::lp::*;
    pub use crate::math::*;
}

/// Returns the version of the `cnvx` crate.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
