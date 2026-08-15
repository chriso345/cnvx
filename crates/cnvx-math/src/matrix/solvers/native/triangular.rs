use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

use super::super::TriType;

/// Solves a triangular (or diagonal) system via forward/back substitution.
pub(crate) fn solve(a: &Matrix, b: &Vector, tri: TriType) -> Result<Vector, MathError> {
    let n = a.rows();
    let mut x = b.to_vec();

    for i in 0..n {
        if a.get(i, i).abs() < 1e-12 {
            return Err(MathError::Singular);
        }
    }

    match tri {
        TriType::Diagonal => {
            for i in 0..n {
                x[i] /= a.get(i, i);
            }
        }
        TriType::Lower => {
            // Forward substitution.
            for i in 0..n {
                let mut sum = x[i];
                for k in 0..i {
                    sum -= a.get(i, k) * x[k];
                }
                x[i] = sum / a.get(i, i);
            }
        }
        TriType::Upper => {
            // Backward substitution.
            for i in (0..n).rev() {
                let mut sum = x[i];
                for k in (i + 1)..n {
                    sum -= a.get(i, k) * x[k];
                }
                x[i] = sum / a.get(i, i);
            }
        }
    }

    Ok(Vector::from(x))
}
