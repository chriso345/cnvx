use lapacke::{Layout, dgeev, dgeqrf, dgesvd, dgetrf, dorgqr, dpotrf, dsyev};

use super::{
    CholeskyDecomposition, EigenDecomposition, LuDecomposition, QrDecomposition,
    SvdDecomposition,
};
use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

pub(super) fn lu(a: &Matrix) -> Result<LuDecomposition, MathError> {
    let n = a.rows() as i32;
    let mut buf = a.data().to_vec();
    let mut ipiv = vec![0i32; n as usize];

    let info = unsafe { dgetrf(Layout::RowMajor, n, n, &mut buf, n, &mut ipiv) };
    if info > 0 {
        return Err(MathError::Singular);
    } else if info < 0 {
        return Err(MathError::InvalidArgument(format!(
            "illegal value in argument {}",
            -info
        )));
    }

    let dim = n as usize;
    let mut l = Matrix::identity(dim);
    let mut u = Matrix::zeros(dim, dim);
    for i in 0..dim {
        for j in 0..dim {
            let v = buf[i * dim + j];
            if i > j {
                l.set(i, j, v);
            } else {
                u.set(i, j, v);
            }
        }
    }

    // LAPACKE's ipiv is 1-based and describes sequential row swaps: row i
    // (0-based) was swapped with row ipiv[i]-1 during the algorithm. Replay
    // those swaps against the identity permutation to get the final,
    // explicit permutation array (p[i] = original row now in row i).
    let mut p: Vec<usize> = (0..dim).collect();
    for (i, &pivot) in ipiv.iter().enumerate().take(dim) {
        let swap_with = pivot as usize - 1;
        p.swap(i, swap_with);
    }

    Ok(LuDecomposition { l, u, p })
}

pub(super) fn qr(a: &Matrix) -> Result<QrDecomposition, MathError> {
    let m = a.rows() as i32;
    let n = a.cols() as i32;
    let k = m.min(n);

    let mut buf = a.data().to_vec();
    let mut tau = vec![0.0; k as usize];

    let info = unsafe { dgeqrf(Layout::RowMajor, m, n, &mut buf, n, &mut tau) };
    if info != 0 {
        return Err(MathError::InvalidArgument(format!(
            "dgeqrf: illegal value in argument {}",
            -info
        )));
    }

    // Extract R (upper-triangular / upper-trapezoidal) before buf is
    // overwritten with Q's Householder-vector representation.
    let mut r = Matrix::zeros(k as usize, a.cols());
    for i in 0..k as usize {
        for j in i..a.cols() {
            r.set(i, j, buf[i * a.cols() + j]);
        }
    }

    // dorgqr builds the first `k` columns of Q explicitly, in place.
    let info = unsafe { dorgqr(Layout::RowMajor, m, k, k, &mut buf, n, &tau) };
    if info != 0 {
        return Err(MathError::InvalidArgument(format!(
            "dorgqr: illegal value in argument {}",
            -info
        )));
    }

    let mut q = Matrix::zeros(a.rows(), k as usize);
    for i in 0..a.rows() {
        for j in 0..k as usize {
            q.set(i, j, buf[i * a.cols() + j]);
        }
    }

    Ok(QrDecomposition { q, r })
}

pub(super) fn cholesky(a: &Matrix) -> Result<CholeskyDecomposition, MathError> {
    let n = a.rows() as i32;
    let mut buf = a.data().to_vec();

    let info = unsafe { dpotrf(Layout::RowMajor, b'L', n, &mut buf, n) };
    if info > 0 {
        return Err(MathError::NotPositiveDefinite);
    } else if info < 0 {
        return Err(MathError::InvalidArgument(format!(
            "illegal value in argument {}",
            -info
        )));
    }

    let dim = n as usize;
    let mut l = Matrix::zeros(dim, dim);
    for i in 0..dim {
        for j in 0..=i {
            l.set(i, j, buf[i * dim + j]);
        }
    }
    Ok(CholeskyDecomposition { l })
}

pub(super) fn eigen_symmetric(a: &Matrix) -> Result<EigenDecomposition, MathError> {
    let n = a.rows() as i32;
    let mut buf = a.data().to_vec();
    let mut w = vec![0.0; n as usize];

    let info = unsafe { dsyev(Layout::RowMajor, b'V', b'U', n, &mut buf, n, &mut w) };
    if info != 0 {
        return Err(MathError::Unsupported(format!(
            "dsyev failed to converge (info = {info})"
        )));
    }

    // dsyev returns eigenvalues in *ascending* order with eigenvectors as
    // columns of the row-major `buf` (overwritten in place); reverse both
    // to match this crate's documented descending-order contract (shared
    // with the native Jacobi backend, which produces descending order
    // directly).
    let dim = n as usize;
    let mut values = w;
    values.reverse();
    let mut vectors = Matrix::zeros(dim, dim);
    let original = Matrix::from_row_major(dim, dim, buf);
    for (new_col, old_col) in (0..dim).rev().enumerate() {
        for row in 0..dim {
            vectors.set(row, new_col, original.get(row, old_col));
        }
    }

    Ok(EigenDecomposition {
        values: Vector::from(values),
        vectors,
        imag_values: None,
    })
}

pub(super) fn eigen_general(a: &Matrix) -> Result<EigenDecomposition, MathError> {
    let n = a.rows() as i32;
    let mut buf = a.data().to_vec();
    let mut wr = vec![0.0; n as usize];
    let mut wi = vec![0.0; n as usize];
    let mut vl = vec![0.0; (n * n) as usize];
    let mut vr = vec![0.0; (n * n) as usize];

    let info = unsafe {
        dgeev(
            Layout::RowMajor,
            b'N',
            b'V',
            n,
            &mut buf,
            n,
            &mut wr,
            &mut wi,
            &mut vl,
            n,
            &mut vr,
            n,
        )
    };
    if info != 0 {
        return Err(MathError::Unsupported(format!(
            "dgeev failed to converge (info = {info})"
        )));
    }

    let has_complex = wi.iter().any(|&x| x != 0.0);
    Ok(EigenDecomposition {
        values: Vector::from(wr),
        vectors: Matrix::from_row_major(a.rows(), a.rows(), vr),
        imag_values: if has_complex { Some(Vector::from(wi)) } else { None },
    })
}

pub(super) fn svd(a: &Matrix) -> Result<SvdDecomposition, MathError> {
    let m = a.rows() as i32;
    let n = a.cols() as i32;
    let mut buf = a.data().to_vec();
    let k = m.min(n) as usize;
    let mut s = vec![0.0; k];
    let mut u = vec![0.0; (m * m) as usize];
    let mut vt = vec![0.0; (n * n) as usize];
    let mut superb = vec![0.0; k.max(1) - 1 + 1];

    let info = unsafe {
        dgesvd(
            Layout::RowMajor,
            b'A',
            b'A',
            m,
            n,
            &mut buf,
            n,
            &mut s,
            &mut u,
            m,
            &mut vt,
            n,
            &mut superb,
        )
    };
    if info != 0 {
        return Err(MathError::Unsupported(format!(
            "dgesvd failed to converge (info = {info})"
        )));
    }

    Ok(SvdDecomposition {
        u: Matrix::from_row_major(a.rows(), a.rows(), u),
        s: Vector::from(s),
        vt: Matrix::from_row_major(a.cols(), a.cols(), vt),
    })
}
