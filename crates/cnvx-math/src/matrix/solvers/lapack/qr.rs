use lapacke::{Layout, dgels};

use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// Solves overdetermined rectangular systems (least squares) via LAPACK's
/// `dgels` (QR factorization).
pub(crate) fn solve(a: &Matrix, b: &Vector) -> Result<Vector, MathError> {
    let m = a.rows() as i32;
    let n = a.cols() as i32;

    if m < n {
        return Err(MathError::Unsupported(
            "underdetermined systems (rows < cols) are not yet supported".into(),
        ));
    }
    if b.len() != m as usize {
        return Err(MathError::DimensionMismatch(format!(
            "right-hand side has {} entries, matrix has {m} rows",
            b.len()
        )));
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
            b'N', // no transpose
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
        return Err(MathError::RankDeficient);
    } else if info < 0 {
        return Err(MathError::InvalidArgument(format!(
            "illegal value in argument {}",
            -info
        )));
    }

    // The first `n` elements contain the least-squares solution.
    x.truncate(n as usize);
    Ok(Vector::from(x))
}
