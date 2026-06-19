pub mod cholesky;
pub mod lu;
pub mod qr;
pub mod triangular;

use crate::matrix::{DenseMatrix, Matrix};

/// MATLAB-style mldivide dispatcher for full matrices
/// See <https://mathworks.com/help/matlab/ref/double.mldivide.html>
pub fn mldivide_dense(a: &DenseMatrix, b: &[f64]) -> Result<Vec<f64>, String> {
    let m = a.rows();
    let n = a.cols();

    // 1. Is matrix square? If not, use QR for least squares.
    if m != n {
        return qr::solve(a, b);
    }

    if b.len() != n {
        return Err("RHS vector length must match matrix rows".to_string());
    }

    // 2. Is it perfectly triangular?
    if let Some(tri_type) = triangular::check_structure(a) {
        return triangular::solve(a, b, tri_type);
    }

    // 3. Is it symmetric?
    if is_symmetric(a) {
        // Try Cholesky decomposition (Symmetric Positive Definite)
        // If it succeeds, we are done. If it fails, fall through to LU.
        if let Ok(x) = cholesky::solve(a, b) {
            return Ok(x);
        }
    }

    // 4. Fallback: LU Decomposition with partial pivoting
    lu::solve(a, b)
}

/// Checks if A is symmetric: A_ij == A_ji
fn is_symmetric(a: &DenseMatrix) -> bool {
    let n = a.rows();
    for i in 0..n {
        for j in (i + 1)..n {
            if (a.get(i, j) - a.get(j, i)).abs() > 1e-12 {
                return false;
            }
        }
    }
    true
}
