use lapacke::{Layout, dgels};

use crate::matrix::{DenseMatrix, Matrix};

/// Solves overdetermined rectangular systems (Least Squares) using QR
/// factorization.
pub fn solve(a: &DenseMatrix, b: &[f64]) -> Result<Vec<f64>, String> {
    let m = a.rows() as i32;
    let n = a.cols() as i32;

    if m < n {
        return Err("Underdetermined systems (m < n) are not yet supported.".into());
    }
    if b.len() != m as usize {
        return Err("RHS vector length must match matrix rows".into());
    }

    let mut a_copy = a.data().to_vec();

    // LAPACK's dgels requires the RHS array to have a size of max(m, n)
    // because it overwrites the RHS with the solution.
    let max_mn = std::cmp::max(m, n) as usize;
    let mut x = vec![0.0; max_mn];
    x[..b.len()].copy_from_slice(b);

    let info = unsafe {
        dgels(
            Layout::RowMajor,
            b'N', // No transpose
            m,
            n,
            1, // nrhs
            &mut a_copy,
            n, // lda
            &mut x,
            1, // ldb
        )
    };

    if info > 0 {
        return Err("Matrix is rank deficient".into());
    } else if info < 0 {
        return Err(format!("Illegal value in argument {}", -info));
    }

    // The first `n` elements contain the least squares solution
    x.truncate(n as usize);
    Ok(x)
}
