use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// Pure-Rust least-squares solve for overdetermined (`rows >= cols`)
/// systems via QR factorization. Functionally equivalent to
/// the native backend's LAPACK `dgels` call.
pub(crate) fn solve(a: &Matrix, b: &Vector) -> Result<Vector, MathError> {
    let m = a.rows();
    let n = a.cols();

    if m < n {
        return Err(MathError::Unsupported(
            "underdetermined systems (rows < cols) are not yet supported".into(),
        ));
    }
    if b.len() != m {
        return Err(MathError::DimensionMismatch(format!(
            "right-hand side has {} entries, matrix has {m} rows",
            b.len()
        )));
    }

    // Work on a column-major copy: `r[col * m + row]`. Householder
    // reflections are applied column-by-column, so column-major storage
    // keeps each reflected column contiguous.
    let mut r = vec![0.0; m * n];
    for i in 0..m {
        for j in 0..n {
            r[j * m + i] = a.get(i, j);
        }
    }
    let mut qtb = b.to_vec();

    for k in 0..n {
        let mut norm = 0.0f64;
        for i in k..m {
            norm += r[k * m + i] * r[k * m + i];
        }
        norm = norm.sqrt();
        if norm < 1e-14 {
            return Err(MathError::RankDeficient);
        }

        let alpha = if r[k * m + k] >= 0.0 { -norm } else { norm };
        let mut v = vec![0.0; m - k];
        v[0] = r[k * m + k] - alpha;
        for i in (k + 1)..m {
            v[i - k] = r[k * m + i];
        }
        let v_norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if v_norm < 1e-14 {
            // Column is already zero below the diagonal; nothing to reflect.
            r[k * m + k] = alpha;
            for i in (k + 1)..m {
                r[k * m + i] = 0.0;
            }
            continue;
        }
        for vi in v.iter_mut() {
            *vi /= v_norm;
        }

        // Apply the reflection (I - 2vv^T) to columns k..n of R.
        for j in k..n {
            let mut dot = 0.0;
            for i in k..m {
                dot += v[i - k] * r[j * m + i];
            }
            for i in k..m {
                r[j * m + i] -= 2.0 * v[i - k] * dot;
            }
        }

        // Apply the same reflection to Q^T b.
        let mut dot = 0.0;
        for i in k..m {
            dot += v[i - k] * qtb[i];
        }
        for i in k..m {
            qtb[i] -= 2.0 * v[i - k] * dot;
        }
    }

    // Back-substitute R[0..n, 0..n] x = (Q^T b)[0..n].
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let diag = r[i * m + i];
        if diag.abs() < 1e-12 {
            return Err(MathError::RankDeficient);
        }
        let mut sum = qtb[i];
        for k in (i + 1)..n {
            sum -= r[k * m + i] * x[k];
        }
        x[i] = sum / diag;
    }

    Ok(Vector::from(x))
}
