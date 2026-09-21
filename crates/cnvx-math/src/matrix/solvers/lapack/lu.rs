use lapacke::{Layout, dgesv};

use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// General square solve via LAPACK's `dgesv` (LU with partial pivoting).
pub(crate) fn solve(a: &Matrix, b: &Vector) -> Result<Vector, MathError> {
    let n = a.rows() as i32;

    // LAPACK mutates A to store the LU factors, so we must clone it.
    let mut a_copy = a.data().to_vec();
    // LAPACK mutates b to store the solution x.
    let mut x = b.to_vec();
    let mut ipiv = vec![0; n as usize];

    let info = unsafe {
        dgesv(
            Layout::RowMajor,
            n,
            1, // nrhs
            &mut a_copy,
            n, // lda
            &mut ipiv,
            &mut x,
            1, // ldb
        )
    };

    if info > 0 {
        return Err(MathError::Singular);
    } else if info < 0 {
        return Err(MathError::InvalidArgument(format!(
            "illegal value in argument {}",
            -info
        )));
    }

    Ok(Vector::from(x))
}
