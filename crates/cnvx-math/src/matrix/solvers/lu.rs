use lapacke::{Layout, dgesv};

use crate::matrix::{DenseMatrix, Matrix};

pub fn solve(a: &DenseMatrix, b: &[f64]) -> Result<Vec<f64>, String> {
    let n = a.rows() as i32;

    // LAPACK mutates A to store the LU factors, so we must clone it.
    let mut a_copy = a.data().to_vec();

    // LAPACK mutates b to store the solution x.
    let mut x = b.to_vec();

    // Pivot indices array
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
        return Err("Matrix is singular (U(i,i) is exactly zero)".into());
    } else if info < 0 {
        return Err(format!("Illegal value in argument {}", -info));
    }

    Ok(x)
}
