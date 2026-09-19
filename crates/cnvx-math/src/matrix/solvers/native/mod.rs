//! Pure-Rust solvers (`wasm32` targets only), functionally equivalent to
//! the `lapack` backend used on native targets.

pub(crate) mod cholesky;
pub(crate) mod lu;
pub(crate) mod qr;
pub(crate) mod triangular;
