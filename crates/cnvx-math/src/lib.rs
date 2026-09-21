//! # cnvx-math
//!
//! Math utilities for cnvx's solvers and optimization algorithms: dense
//! and sparse linear algebra, random sampling, statistics, calculus, and
//! an N-dimensional array with broadcasting.
//!
//! - [`Vector`] - a thin, owned wrapper around `Vec<f64>`.
//! - [`Matrix`] - a dense, row-major matrix, with decompositions
//!   ([`Matrix::lu`], [`Matrix::qr`], [`Matrix::cholesky`],
//!   [`Matrix::eigen_symmetric`], [`Matrix::eigen_general`], [`Matrix::svd`])
//!   and the derived quantities built on them ([`Matrix::determinant`],
//!   [`Matrix::rank`], [`Matrix::condition_number`],
//!   [`Matrix::pseudo_inverse`], [`Matrix::least_squares`]).
//! - [`SparseMatrix`] - a CSR matrix, with iterative solvers
//!   ([`SparseMatrix::solve_cg`], [`SparseMatrix::solve_gmres`]), direct
//!   solvers ([`SparseMatrix::solve_direct`] via UMFPACK, and partial
//!   eigendecomposition ([`SparseMatrix::smallest_eigenpairs`],
//!   [`SparseMatrix::largest_eigenpairs`] via Lanczos).
//! - [`random`] - a dependency-free PRNG ([`random::Rng`]) and probability
//!   distributions ([`random::Distribution`] and implementors).
//! - [`stats`] - descriptive statistics: mean, variance, quantiles,
//!   covariance/correlation (scalar and matrix forms), histograms.
//! - [`special`] - special functions: the gamma and error function families.
//! - [`calculus`] - finite-difference derivatives, scalar root-finding, and
//!   quadrature.
//! - [`MathError`] - the error type shared by all fallible operations above.
//!
//! ## Native vs. WebAssembly
//!
//! [`Matrix::solve`] dispatches to a MATLAB-style `mldivide`: a triangular
//! fast-path, Cholesky for symmetric positive-definite systems, LU with
//! partial pivoting as the general square fallback, and QR-based least
//! squares for overdetermined rectangular systems.
//!
//! On non-`wasm32` targets these are backed by native BLAS/LAPACK
//! (`cblas` / `lapacke`, linked via `build.rs` against the system
//! library via `pkg-config`), and [`SparseMatrix`]'s direct solvers are
//! backed by UMFPACK and MUMPS (also linked via `build.rs`). WebAssembly
//! cannot link against these, so `wasm32` targets use equivalent
//! pure-Rust implementations for the dense decompositions instead, and
//! do not expose [`SparseMatrix::solve_direct`] at all (there's no pure-Rust
//! substitute for a from-scratch sparse direct solver in this crate;
//! use [`SparseMatrix::solve_cg`]/[`SparseMatrix::solve_gmres`] instead, which
//! are pure Rust and available everywhere). Both dense-decomposition paths are
//! exercised by the same dispatch logic and the same public API.

pub mod calculus;
mod elementwise;
mod error;
mod linalg;
mod matrix;
pub mod random;
mod sparse;
pub mod special;
pub mod stats;
mod vector;

pub use error::MathError;
pub use linalg::{
    CholeskyDecomposition, EigenDecomposition, LuDecomposition, QrDecomposition,
    SvdDecomposition,
};
pub use matrix::Matrix;
pub use sparse::{IterativeOptions, IterativeResult, Preconditioner, SparseMatrix};
pub use stats::Histogram;
pub use vector::Vector;
