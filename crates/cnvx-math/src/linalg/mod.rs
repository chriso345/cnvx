//! Dense matrix decompositions: LU, QR, Cholesky, symmetric/general
//! eigendecomposition, and SVD.
//!
//! Every decomposition here follows the same split as the existing
//! [`crate::Matrix::solve`]: a native BLAS/LAPACK-backed implementation on
//! non-`wasm32` targets, and a functionally equivalent
//! pure-Rust implementation on `wasm32`. Both are exercised by
//! the same public methods on [`crate::Matrix`], so callers never choose a
//! backend explicitly -- `cfg(target_arch = "wasm32")` does.

use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

#[cfg(not(target_arch = "wasm32"))]
mod lapack;
#[cfg(target_arch = "wasm32")]
mod native;

/// The result of an LU decomposition with partial pivoting: `P A = L U`.
///
/// `l` is unit lower-triangular, `u` is upper-triangular, and `p` is the
/// row permutation applied to `A` (i.e. `p[i]` is the original row that
/// ended up in row `i`). Exposed directly (beyond what [`Matrix::solve`]
/// already does internally) for callers who want to reuse the same
/// factorization across multiple right-hand sides without re-factorizing.
#[derive(Debug, Clone, PartialEq)]
pub struct LuDecomposition {
    pub l: Matrix,
    pub u: Matrix,
    pub p: Vec<usize>,
}

/// The result of a QR decomposition: `A = Q R`, with `Q` orthogonal and `R`
/// upper-triangular (upper-trapezoidal for rectangular `A`).
#[derive(Debug, Clone, PartialEq)]
pub struct QrDecomposition {
    pub q: Matrix,
    pub r: Matrix,
}

/// The result of a Cholesky decomposition: `A = L L^T`, valid only for
/// symmetric positive-definite `A`.
#[derive(Debug, Clone, PartialEq)]
pub struct CholeskyDecomposition {
    pub l: Matrix,
}

/// The result of an eigendecomposition: `A v_i = value_i * v_i` for each
/// column `v_i` of `vectors`.
///
/// For [`Matrix::eigen_symmetric`], `values` and `vectors` are always real
/// and `imag_values` is always `None`. For [`Matrix::eigen_general`],
/// non-symmetric matrices can have complex-conjugate eigenvalue pairs;
/// `imag_values`, when `Some`, holds the imaginary part matching `values`'
/// real part (a zero entry means that eigenvalue is real). When complex
/// eigenvalues are present, `vectors` holds the real Schur basis rather
/// than a complex eigenvector matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct EigenDecomposition {
    pub values: Vector,
    pub vectors: Matrix,
    pub imag_values: Option<Vector>,
}

/// The result of a singular value decomposition: `A = U * diag(s) * V^T`.
///
/// `s` is sorted in descending order, matching LAPACK's `dgesvd`
/// convention.
#[derive(Debug, Clone, PartialEq)]
pub struct SvdDecomposition {
    pub u: Matrix,
    pub s: Vector,
    pub vt: Matrix,
}

pub(crate) fn lu(a: &Matrix) -> Result<LuDecomposition, MathError> {
    if a.rows() != a.cols() {
        return Err(MathError::DimensionMismatch(format!(
            "Matrix::lu requires a square matrix, got {}x{}",
            a.rows(),
            a.cols()
        )));
    }
    #[cfg(not(target_arch = "wasm32"))]
    return lapack::lu(a);
    #[cfg(target_arch = "wasm32")]
    return native::lu(a);
}

pub(crate) fn qr(a: &Matrix) -> Result<QrDecomposition, MathError> {
    if a.rows() < a.cols() {
        return Err(MathError::Unsupported("Matrix::qr requires rows >= cols".into()));
    }
    #[cfg(not(target_arch = "wasm32"))]
    return lapack::qr(a);
    #[cfg(target_arch = "wasm32")]
    return native::qr(a);
}

pub(crate) fn cholesky(a: &Matrix) -> Result<CholeskyDecomposition, MathError> {
    if a.rows() != a.cols() {
        return Err(MathError::DimensionMismatch(format!(
            "Matrix::cholesky requires a square matrix, got {}x{}",
            a.rows(),
            a.cols()
        )));
    }
    #[cfg(not(target_arch = "wasm32"))]
    return lapack::cholesky(a);
    #[cfg(target_arch = "wasm32")]
    return native::cholesky(a);
}

pub(crate) fn eigen_symmetric(a: &Matrix) -> Result<EigenDecomposition, MathError> {
    if a.rows() != a.cols() {
        return Err(MathError::DimensionMismatch(format!(
            "Matrix::eigen_symmetric requires a square matrix, got {}x{}",
            a.rows(),
            a.cols()
        )));
    }
    #[cfg(not(target_arch = "wasm32"))]
    return lapack::eigen_symmetric(a);
    #[cfg(target_arch = "wasm32")]
    return native::eigen_symmetric(a);
}

pub(crate) fn eigen_general(a: &Matrix) -> Result<EigenDecomposition, MathError> {
    if a.rows() != a.cols() {
        return Err(MathError::DimensionMismatch(format!(
            "Matrix::eigen_general requires a square matrix, got {}x{}",
            a.rows(),
            a.cols()
        )));
    }
    #[cfg(not(target_arch = "wasm32"))]
    return lapack::eigen_general(a);
    #[cfg(target_arch = "wasm32")]
    return native::eigen_general(a);
}

pub(crate) fn svd(a: &Matrix) -> Result<SvdDecomposition, MathError> {
    #[cfg(not(target_arch = "wasm32"))]
    return lapack::svd(a);
    #[cfg(target_arch = "wasm32")]
    return native::svd(a);
}
