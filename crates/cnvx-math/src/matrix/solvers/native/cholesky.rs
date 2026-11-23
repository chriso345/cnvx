use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// Pure-Rust Cholesky factorization (`A = L L^T`) followed by forward/back
/// substitution. Functionally equivalent to the native backend's
/// LAPACK `dposv` call.
pub(crate) fn solve(a: &Matrix, b: &Vector) -> Result<Vector, MathError> {
    let n = a.rows();
    let mut l = vec![0.0; n * n];

    for i in 0..n {
        for j in 0..=i {
            let mut sum = a.get(i, j);
            for k in 0..j {
                sum -= l[i * n + k] * l[j * n + k];
            }
            if i == j {
                if sum <= 0.0 {
                    return Err(MathError::NotPositiveDefinite);
                }
                l[i * n + j] = sum.sqrt();
            } else {
                l[i * n + j] = sum / l[j * n + j];
            }
        }
    }

    // Forward substitution: L y = b
    let mut y = vec![0.0; n];
    for i in 0..n {
        let mut sum = b[i];
        for k in 0..i {
            sum -= l[i * n + k] * y[k];
        }
        y[i] = sum / l[i * n + i];
    }

    // Back substitution: L^T x = y
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = y[i];
        for k in (i + 1)..n {
            sum -= l[k * n + i] * x[k];
        }
        x[i] = sum / l[i * n + i];
    }

    Ok(Vector::from(x))
}
