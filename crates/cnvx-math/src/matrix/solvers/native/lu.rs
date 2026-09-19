use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// Pure-Rust LU decomposition with partial pivoting, followed by
/// forward/back substitution. Functionally equivalent to the native
/// backend's LAPACK `dgesv` call.
pub(crate) fn solve(a: &Matrix, b: &Vector) -> Result<Vector, MathError> {
    let n = a.rows();
    let mut lu = (0..n * n).map(|idx| a.get(idx / n, idx % n)).collect::<Vec<f64>>();
    let mut p: Vec<usize> = (0..n).collect();

    for i in 0..n {
        let mut max_a = 0.0;
        let mut imax = i;

        for k in i..n {
            let abs_a = lu[k * n + i].abs();
            if abs_a > max_a {
                max_a = abs_a;
                imax = k;
            }
        }

        if max_a < 1e-12 {
            return Err(MathError::Singular);
        }

        if imax != i {
            for k in 0..n {
                lu.swap(i * n + k, imax * n + k);
            }
            p.swap(i, imax);
        }

        for j in (i + 1)..n {
            lu[j * n + i] /= lu[i * n + i];
            for k in (i + 1)..n {
                lu[j * n + k] -= lu[j * n + i] * lu[i * n + k];
            }
        }
    }

    // Apply permutation to RHS.
    let mut x = vec![0.0; n];
    for i in 0..n {
        x[i] = b[p[i]];
    }

    // Forward substitution (Ly = Pb).
    for i in 0..n {
        for k in 0..i {
            x[i] -= lu[i * n + k] * x[k];
        }
    }

    // Backward substitution (Ux = y).
    for i in (0..n).rev() {
        for k in (i + 1)..n {
            x[i] -= lu[i * n + k] * x[k];
        }
        x[i] /= lu[i * n + i];
    }

    Ok(Vector::from(x))
}
