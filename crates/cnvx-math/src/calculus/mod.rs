//! Numerical calculus: finite-difference derivative approximations,
//! scalar root-finding, and quadrature.

mod finite_diff;
mod quadrature;
mod root_finding;

pub use finite_diff::{DEFAULT_STEP, gradient, hessian, jacobian};
pub use quadrature::{simpson, trapezoidal};
pub use root_finding::{bisection, newton};
