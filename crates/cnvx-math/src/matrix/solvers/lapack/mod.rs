//! Native BLAS/LAPACK-backed solvers (non-`wasm32` targets only).

pub(crate) mod cholesky;
pub(crate) mod lu;
pub(crate) mod qr;
pub(crate) mod triangular;
