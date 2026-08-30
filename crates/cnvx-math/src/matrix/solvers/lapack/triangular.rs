use cblas::{Diagonal, Layout, Part, Transpose, dtrsv};

use super::super::TriType;
use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// Solves a triangular (or diagonal) system via CBLAS's `dtrsv`.
pub(crate) fn solve(a: &Matrix, b: &Vector, tri: TriType) -> Result<Vector, MathError> {
    let n = a.rows() as i32;
    let mut x = b.to_vec();

    // Check for exact zeros on the diagonal to avoid NaN pollution before
    // CBLAS.
    for i in 0..a.rows() {
        if a.get(i, i).abs() < 1e-12 {
            return Err(MathError::Singular);
        }
    }

    let uplo = match tri {
        TriType::Upper => Part::Upper,
        TriType::Lower => Part::Lower,
        TriType::Diagonal => {
            // Pure diagonal division is faster than invoking BLAS.
            for (i, xi) in x.iter_mut().enumerate() {
                *xi /= a.get(i, i);
            }
            return Ok(Vector::from(x));
        }
    };

    unsafe {
        dtrsv(
            Layout::RowMajor,
            uplo,
            Transpose::None,
            Diagonal::Generic,
            n,
            a.data(),
            n, // lda
            &mut x,
            1, // incx
        );
    }
    Ok(Vector::from(x))
}
