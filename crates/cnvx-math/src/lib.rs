//! # cnvx-math
//!
//! Linear algebra primitives for cnvx's solvers.
//!
//! - [`Vector`] - a thin, owned wrapper around `Vec<f64>`.
//! - [`Matrix`] - a dense, row-major matrix.
//! - [`SparseMatrix`] - a CSR matrix.
//! - [`MathError`] - the error type shared by all fallible operations
//!   above.
//!
//! ## Native vs. WebAssembly
//!
//! [`Matrix::solve`] dispatches to a MATLAB-style `mldivide`: a triangular
//! fast-path, Cholesky for symmetric positive-definite systems, LU with
//! partial pivoting as the general square fallback, and QR-based least
//! squares for overdetermined rectangular systems.
//!
//! On non-`wasm32` targets this is backed by native BLAS/LAPACK (`cblas` /
//! `lapacke` / `openblas-src`, linked via `build.rs`). WebAssembly cannot
//! link against a system BLAS/LAPACK, so `wasm32` targets use an
//! equivalent pure-Rust implementation instead. Both paths are exercised
//! by the same dispatch logic and the same public API.

#[cfg(not(target_arch = "wasm32"))]
extern crate openblas_src;

mod error;
mod matrix;
mod sparse;
mod vector;

pub use error::MathError;
pub use matrix::Matrix;
pub use sparse::SparseMatrix;
pub use vector::Vector;
