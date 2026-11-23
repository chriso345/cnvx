use lapacke::{Layout, dposv};

use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// Solves a symmetric positive-definite system via LAPACK's `dposv`.
pub(crate) fn solve(a: &Matrix, b: &Vector) -> Result<Vector, MathError> {
    let n = a.rows() as i32;

    let mut a_copy = a.data().to_vec();
    let mut x = b.to_vec();

    let info = unsafe {
        dposv(
            Layout::RowMajor,
            b'U', // check the upper triangle
            n,
            1, // nrhs
            &mut a_copy,
            n, // lda
            &mut x,
            1, // ldb
        )
    };

    if info > 0 {
        return Err(MathError::NotPositiveDefinite);
    } else if info < 0 {
        return Err(MathError::InvalidArgument(format!(
            "illegal value in argument {}",
            -info
        )));
    }

    Ok(Vector::from(x))
}
