//! `Matrix::solve`'s MATLAB-style `mldivide` dispatcher.
//!
//! Structure detection ([`check_triangular`], [`is_symmetric`]) is pure
//! Rust and shared by both backends. The actual factorization/substitution
//! work is backend-specific: native LAPACK/BLAS (`lapack`) on non-`wasm32`
//! targets, a pure-Rust reimplementation (`native`) on `wasm32`.

use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

#[cfg(not(target_arch = "wasm32"))]
mod lapack;
#[cfg(target_arch = "wasm32")]
mod native;

mod structure;

pub(crate) use structure::{TriType, check_triangular, is_symmetric};

/// Solves `A x = rhs`. See [`crate::Matrix::solve`] for the dispatch
/// order.
pub(crate) fn solve_dense(a: &Matrix, rhs: &Vector) -> Result<Vector, MathError> {
    let m = a.rows();
    let n = a.cols();

    // 1. Non-square: least squares via QR.
    if m != n {
        return solve_least_squares(a, rhs);
    }

    if rhs.len() != n {
        return Err(MathError::DimensionMismatch(format!(
            "right-hand side has {} entries, matrix has {n} columns",
            rhs.len()
        )));
    }

    // 2. Triangular (or diagonal): substitution fast-path.
    if let Some(tri) = check_triangular(a) {
        return solve_triangular(a, rhs, tri);
    }

    // 3. Symmetric: try Cholesky (i.e. is it also positive-definite?).
    if is_symmetric(a)
        && let Ok(x) = solve_cholesky(a, rhs)
    {
        return Ok(x);
    }

    // 4. General square fallback: LU with partial pivoting.
    solve_lu(a, rhs)
}

#[cfg(not(target_arch = "wasm32"))]
fn solve_triangular(a: &Matrix, rhs: &Vector, tri: TriType) -> Result<Vector, MathError> {
    lapack::triangular::solve(a, rhs, tri)
}
#[cfg(target_arch = "wasm32")]
fn solve_triangular(a: &Matrix, rhs: &Vector, tri: TriType) -> Result<Vector, MathError> {
    native::triangular::solve(a, rhs, tri)
}

#[cfg(not(target_arch = "wasm32"))]
fn solve_cholesky(a: &Matrix, rhs: &Vector) -> Result<Vector, MathError> {
    lapack::cholesky::solve(a, rhs)
}
#[cfg(target_arch = "wasm32")]
fn solve_cholesky(a: &Matrix, rhs: &Vector) -> Result<Vector, MathError> {
    native::cholesky::solve(a, rhs)
}

#[cfg(not(target_arch = "wasm32"))]
fn solve_lu(a: &Matrix, rhs: &Vector) -> Result<Vector, MathError> {
    lapack::lu::solve(a, rhs)
}
#[cfg(target_arch = "wasm32")]
fn solve_lu(a: &Matrix, rhs: &Vector) -> Result<Vector, MathError> {
    native::lu::solve(a, rhs)
}

#[cfg(not(target_arch = "wasm32"))]
fn solve_least_squares(a: &Matrix, rhs: &Vector) -> Result<Vector, MathError> {
    lapack::qr::solve(a, rhs)
}
#[cfg(target_arch = "wasm32")]
fn solve_least_squares(a: &Matrix, rhs: &Vector) -> Result<Vector, MathError> {
    native::qr::solve(a, rhs)
}
