use crate::matrix::{DenseMatrix, Matrix};
use lapacke::{Layout, dposv};

pub fn solve(a: &DenseMatrix, b: &[f64]) -> Result<Vec<f64>, String> {
    let n = a.rows() as i32;

    let mut a_copy = a.data().to_vec();
    let mut x = b.to_vec();

    let info = unsafe {
        dposv(
            Layout::RowMajor,
            b'U', // Check Upper triangle
            n,
            1, // nrhs
            &mut a_copy,
            n, // lda
            &mut x,
            1, // ldb
        )
    };

    if info > 0 {
        return Err("Matrix is not strictly positive definite".into());
    } else if info < 0 {
        return Err(format!("Illegal value in argument {}", -info));
    }

    Ok(x)
}
