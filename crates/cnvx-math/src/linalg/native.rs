use super::{
    CholeskyDecomposition, EigenDecomposition, LuDecomposition, QrDecomposition,
    SvdDecomposition,
};
use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

pub(super) fn lu(a: &Matrix) -> Result<LuDecomposition, MathError> {
    let n = a.rows();
    let mut buf = (0..n * n).map(|idx| a.get(idx / n, idx % n)).collect::<Vec<f64>>();
    let mut p: Vec<usize> = (0..n).collect();

    for i in 0..n {
        let mut max_a = 0.0;
        let mut imax = i;
        for k in i..n {
            let abs_a = buf[k * n + i].abs();
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
                buf.swap(i * n + k, imax * n + k);
            }
            p.swap(i, imax);
        }
        for j in (i + 1)..n {
            buf[j * n + i] /= buf[i * n + i];
            for k in (i + 1)..n {
                buf[j * n + k] -= buf[j * n + i] * buf[i * n + k];
            }
        }
    }

    let mut l = Matrix::identity(n);
    let mut u = Matrix::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            let v = buf[i * n + j];
            if i > j {
                l.set(i, j, v);
            } else {
                u.set(i, j, v);
            }
        }
    }
    Ok(LuDecomposition { l, u, p })
}

pub(super) fn qr(a: &Matrix) -> Result<QrDecomposition, MathError> {
    let m = a.rows();
    let n = a.cols();

    // Column-major working buffer for R, and a column-major m x m buffer
    // that accumulates Q by having the same Householder reflectors applied
    // to an identity matrix (see the module doc for the derivation of the
    // `Q[i][j] = q[i * m + j]` indexing this produces).
    let mut r = vec![0.0; m * n];
    for i in 0..m {
        for j in 0..n {
            r[j * m + i] = a.get(i, j);
        }
    }
    let mut q = vec![0.0; m * m];
    for i in 0..m {
        q[i * m + i] = 1.0;
    }

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
            r[k * m + k] = alpha;
            for i in (k + 1)..m {
                r[k * m + i] = 0.0;
            }
            continue;
        }
        for vi in v.iter_mut() {
            *vi /= v_norm;
        }

        for j in k..n {
            let mut dot = 0.0;
            for i in k..m {
                dot += v[i - k] * r[j * m + i];
            }
            for i in k..m {
                r[j * m + i] -= 2.0 * v[i - k] * dot;
            }
        }
        for j in 0..m {
            let mut dot = 0.0;
            for i in k..m {
                dot += v[i - k] * q[j * m + i];
            }
            for i in k..m {
                q[j * m + i] -= 2.0 * v[i - k] * dot;
            }
        }
    }

    let mut r_mat = Matrix::zeros(n, n);
    for i in 0..n {
        for j in i..n {
            r_mat.set(i, j, r[j * m + i]);
        }
    }
    let mut q_mat = Matrix::zeros(m, n);
    for i in 0..m {
        for j in 0..n {
            q_mat.set(i, j, q[i * m + j]);
        }
    }

    Ok(QrDecomposition { q: q_mat, r: r_mat })
}

pub(super) fn cholesky(a: &Matrix) -> Result<CholeskyDecomposition, MathError> {
    let n = a.rows();
    let mut l = Matrix::zeros(n, n);

    for i in 0..n {
        for j in 0..=i {
            let mut sum = a.get(i, j);
            for k in 0..j {
                sum -= l.get(i, k) * l.get(j, k);
            }
            if i == j {
                if sum <= 0.0 {
                    return Err(MathError::NotPositiveDefinite);
                }
                l.set(i, j, sum.sqrt());
            } else {
                l.set(i, j, sum / l.get(j, j));
            }
        }
    }
    Ok(CholeskyDecomposition { l })
}

/// The cyclic Jacobi eigenvalue algorithm for real symmetric matrices.
///
/// Repeatedly zeroes the largest off-diagonal element via a plane
/// rotation until the matrix is diagonal to within `tol`. Converges
/// quadratically for well-separated eigenvalues and is unconditionally
/// stable (every rotation is exactly orthogonal), which is why it's the
/// standard choice when robustness at small-to-medium sizes matters more
/// than the asymptotic speed of a shifted-QR eigensolver.
pub(super) fn eigen_symmetric(a: &Matrix) -> Result<EigenDecomposition, MathError> {
    let n = a.rows();
    let mut mat = a.clone();
    let mut v = Matrix::identity(n);
    let max_sweeps = 100;
    let tol = 1e-13;

    for _ in 0..max_sweeps {
        let mut off_diag_norm = 0.0;
        for p in 0..n {
            for q in (p + 1)..n {
                off_diag_norm += mat.get(p, q) * mat.get(p, q);
            }
        }
        if off_diag_norm.sqrt() < tol {
            break;
        }

        for p in 0..n {
            for q in (p + 1)..n {
                let apq = mat.get(p, q);
                if apq.abs() < 1e-300 {
                    continue;
                }
                let app = mat.get(p, p);
                let aqq = mat.get(q, q);
                let theta = (aqq - app) / (2.0 * apq);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;

                for k in 0..n {
                    let akp = mat.get(k, p);
                    let akq = mat.get(k, q);
                    mat.set(k, p, c * akp - s * akq);
                    mat.set(k, q, s * akp + c * akq);
                }
                for k in 0..n {
                    let apk = mat.get(p, k);
                    let aqk = mat.get(q, k);
                    mat.set(p, k, c * apk - s * aqk);
                    mat.set(q, k, s * apk + c * aqk);
                }
                for k in 0..n {
                    let vkp = v.get(k, p);
                    let vkq = v.get(k, q);
                    v.set(k, p, c * vkp - s * vkq);
                    v.set(k, q, s * vkp + c * vkq);
                }
            }
        }
    }

    let mut values: Vec<f64> = (0..n).map(|i| mat.get(i, i)).collect();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| values[j].partial_cmp(&values[i]).unwrap());
    let sorted_values: Vec<f64> = order.iter().map(|&i| values[i]).collect();
    let mut vectors = Matrix::zeros(n, n);
    for (new_col, &old_col) in order.iter().enumerate() {
        for row in 0..n {
            vectors.set(row, new_col, v.get(row, old_col));
        }
    }
    values.clear();
    values.extend(sorted_values);

    Ok(EigenDecomposition {
        values: Vector::from(values),
        vectors,
        imag_values: None,
    })
}

/// Reduces `a` to upper Hessenberg form `H = Q^T A Q` via Householder
/// reflections, returning `(H, Q)`.
fn hessenberg(a: &Matrix) -> (Matrix, Matrix) {
    let n = a.rows();
    let mut h = a.clone();
    let mut q = Matrix::identity(n);

    for k in 0..n.saturating_sub(2) {
        let m = n - k - 1;
        let mut x = vec![0.0; m];
        for i in 0..m {
            x[i] = h.get(k + 1 + i, k);
        }
        let norm: f64 = x.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm < 1e-300 {
            continue;
        }
        let alpha = if x[0] >= 0.0 { -norm } else { norm };
        let mut v = x.clone();
        v[0] -= alpha;
        let v_norm = v.iter().map(|z| z * z).sum::<f64>().sqrt();
        if v_norm < 1e-300 {
            continue;
        }
        for vi in v.iter_mut() {
            *vi /= v_norm;
        }

        // Apply H = I - 2vv^T from the left to rows k+1..n of h.
        for j in 0..n {
            let mut dot = 0.0;
            for i in 0..m {
                dot += v[i] * h.get(k + 1 + i, j);
            }
            for i in 0..m {
                let new_val = h.get(k + 1 + i, j) - 2.0 * v[i] * dot;
                h.set(k + 1 + i, j, new_val);
            }
        }
        // Apply from the right to columns k+1..n of h.
        for i in 0..n {
            let mut dot = 0.0;
            for j in 0..m {
                dot += v[j] * h.get(i, k + 1 + j);
            }
            for j in 0..m {
                let new_val = h.get(i, k + 1 + j) - 2.0 * v[j] * dot;
                h.set(i, k + 1 + j, new_val);
            }
        }
        // Accumulate into q (from the right, same reflector).
        for i in 0..n {
            let mut dot = 0.0;
            for j in 0..m {
                dot += v[j] * q.get(i, k + 1 + j);
            }
            for j in 0..m {
                let new_val = q.get(i, k + 1 + j) - 2.0 * v[j] * dot;
                q.set(i, k + 1 + j, new_val);
            }
        }
    }
    (h, q)
}

/// General (possibly non-symmetric) real eigenvalues via Hessenberg
/// reduction followed by the (unshifted-except-for-a-Wilkinson-shift)
/// QR algorithm, extracting eigenvalues from the resulting real Schur
/// form's diagonal (1x1 blocks: real eigenvalues; 2x2 blocks:
/// complex-conjugate pairs, solved directly via the block's
/// trace/determinant).
///
/// Eigenvectors are not separately back-transformed for the complex case;
/// `vectors` holds the accumulated real Schur basis (see
/// [`EigenDecomposition`]'s documentation).
pub(super) fn eigen_general(a: &Matrix) -> Result<EigenDecomposition, MathError> {
    let n = a.rows();
    let (mut h, mut q) = hessenberg(a);
    let max_iters = 500 * n.max(1);

    let mut m = n;
    let mut iters_used = 0;
    while m > 1 && iters_used < max_iters {
        // Deflate: check for a negligible subdiagonal entry.
        let mut l = m;
        for i in (1..m).rev() {
            if h.get(i, i - 1).abs()
                < 1e-13 * (h.get(i - 1, i - 1).abs() + h.get(i, i).abs())
            {
                h.set(i, i - 1, 0.0);
                l = i;
                break;
            }
        }
        if l == m {
            // No deflation found this pass; treat the whole active block.
            l = 0;
        }
        if l == m - 1 {
            m -= 1;
            continue;
        }

        // Wilkinson shift from the trailing 2x2 block.
        let a11 = h.get(m - 2, m - 2);
        let a12 = h.get(m - 2, m - 1);
        let a21 = h.get(m - 1, m - 2);
        let a22 = h.get(m - 1, m - 1);
        let tr = a11 + a22;
        let det = a11 * a22 - a12 * a21;
        let disc = tr * tr - 4.0 * det;
        let shift = if disc >= 0.0 {
            let sq = disc.sqrt();
            let e1 = (tr + sq) / 2.0;
            let e2 = (tr - sq) / 2.0;
            if (e1 - a22).abs() < (e2 - a22).abs() { e1 } else { e2 }
        } else {
            a22
        };

        for i in l..m {
            h.set(i, i, h.get(i, i) - shift);
        }

        // QR step on the active submatrix h[l..m, l..m] via Givens rotations.
        let size = m - l;
        let mut cs = vec![(1.0, 0.0); size];
        for i in 0..size - 1 {
            let a_val = h.get(l + i, l + i);
            let b_val = h.get(l + i + 1, l + i);
            let r = (a_val * a_val + b_val * b_val).sqrt();
            let (c, s) = if r < 1e-300 { (1.0, 0.0) } else { (a_val / r, b_val / r) };
            cs[i] = (c, s);
            for j in 0..n {
                let x = h.get(l + i, j);
                let y = h.get(l + i + 1, j);
                h.set(l + i, j, c * x + s * y);
                h.set(l + i + 1, j, -s * x + c * y);
            }
        }
        for i in 0..size - 1 {
            let (c, s) = cs[i];
            for j in 0..n {
                let x = h.get(j, l + i);
                let y = h.get(j, l + i + 1);
                h.set(j, l + i, c * x + s * y);
                h.set(j, l + i + 1, -s * x + c * y);
            }
            for j in 0..n {
                let x = q.get(j, l + i);
                let y = q.get(j, l + i + 1);
                q.set(j, l + i, c * x + s * y);
                q.set(j, l + i + 1, -s * x + c * y);
            }
        }

        for i in l..m {
            h.set(i, i, h.get(i, i) + shift);
        }

        iters_used += 1;
    }

    // Walk the (quasi-triangular) Schur form's diagonal, extracting real
    // eigenvalues from 1x1 blocks and complex-conjugate pairs from 2x2
    // blocks whose off-diagonal product is negative (non-real quadratic).
    let mut real_parts = vec![0.0; n];
    let mut imag_parts = vec![0.0; n];
    let mut i = 0;
    let mut has_complex = false;
    while i < n {
        if i == n - 1 || h.get(i + 1, i).abs() < 1e-9 {
            real_parts[i] = h.get(i, i);
            i += 1;
        } else {
            let a11 = h.get(i, i);
            let a12 = h.get(i, i + 1);
            let a21 = h.get(i + 1, i);
            let a22 = h.get(i + 1, i + 1);
            let tr = a11 + a22;
            let det = a11 * a22 - a12 * a21;
            let disc = tr * tr - 4.0 * det;
            if disc < 0.0 {
                let re = tr / 2.0;
                let im = (-disc).sqrt() / 2.0;
                real_parts[i] = re;
                imag_parts[i] = im;
                real_parts[i + 1] = re;
                imag_parts[i + 1] = -im;
                has_complex = true;
            } else {
                let sq = disc.sqrt();
                real_parts[i] = (tr + sq) / 2.0;
                real_parts[i + 1] = (tr - sq) / 2.0;
            }
            i += 2;
        }
    }

    Ok(EigenDecomposition {
        values: Vector::from(real_parts),
        vectors: q,
        imag_values: if has_complex { Some(Vector::from(imag_parts)) } else { None },
    })
}

/// One-sided Jacobi SVD: repeatedly applies plane rotations to pairs of
/// columns of a working copy of `A` until they're numerically orthogonal,
/// at which point the column norms are the singular values and the
/// normalized columns are `U`. `V` is accumulated as the product of the
/// same rotations applied to an initially-identity matrix.
pub(super) fn svd(a: &Matrix) -> Result<SvdDecomposition, MathError> {
    let m = a.rows();
    let n = a.cols();
    let transposed = m < n;
    let work_source = if transposed { a.transpose() } else { a.clone() };
    let (wm, wn) = (work_source.rows(), work_source.cols());

    let mut work = work_source;
    let mut v = Matrix::identity(wn);
    let max_sweeps = 60;
    let tol = 1e-13;

    for _ in 0..max_sweeps {
        let mut max_off = 0.0f64;
        for p in 0..wn {
            for q in (p + 1)..wn {
                let mut alpha = 0.0;
                let mut beta = 0.0;
                let mut gamma = 0.0;
                for k in 0..wm {
                    let a_kp = work.get(k, p);
                    let a_kq = work.get(k, q);
                    alpha += a_kp * a_kp;
                    beta += a_kq * a_kq;
                    gamma += a_kp * a_kq;
                }
                if gamma.abs() < tol * (alpha * beta).sqrt().max(1e-300) {
                    continue;
                }
                max_off = max_off.max(gamma.abs());

                let zeta = (beta - alpha) / (2.0 * gamma);
                let t = zeta.signum() / (zeta.abs() + (zeta * zeta + 1.0).sqrt());
                let t = if zeta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = c * t;

                for k in 0..wm {
                    let a_kp = work.get(k, p);
                    let a_kq = work.get(k, q);
                    work.set(k, p, c * a_kp - s * a_kq);
                    work.set(k, q, s * a_kp + c * a_kq);
                }
                for k in 0..wn {
                    let v_kp = v.get(k, p);
                    let v_kq = v.get(k, q);
                    v.set(k, p, c * v_kp - s * v_kq);
                    v.set(k, q, s * v_kp + c * v_kq);
                }
            }
        }
        if max_off < tol {
            break;
        }
    }

    let k = wm.min(wn);
    let mut sigmas: Vec<f64> = (0..wn)
        .map(|j| (0..wm).map(|i| work.get(i, j).powi(2)).sum::<f64>().sqrt())
        .collect();
    let mut order: Vec<usize> = (0..wn).collect();
    order.sort_by(|&i, &j| sigmas[j].partial_cmp(&sigmas[i]).unwrap());

    let mut u_full = Matrix::zeros(wm, wm.max(wn));
    let mut v_sorted = Matrix::zeros(wn, wn);
    let mut s_sorted = vec![0.0; k];
    for (new_col, &old_col) in order.iter().enumerate() {
        let sigma = sigmas[old_col];
        if new_col < k {
            s_sorted[new_col] = sigma;
        }
        for row in 0..wn {
            v_sorted.set(row, new_col, v.get(row, old_col));
        }
        if new_col < wm {
            if sigma > 1e-300 {
                for row in 0..wm {
                    u_full.set(row, new_col, work.get(row, old_col) / sigma);
                }
            } else {
                u_full.set(new_col, new_col, 1.0);
            }
        }
    }
    sigmas.clear();

    let mut u = Matrix::zeros(wm, wm);
    for i in 0..wm {
        for j in 0..wm {
            u.set(i, j, u_full.get(i, j));
        }
    }
    let vt = v_sorted.transpose();

    if transposed {
        // A = (A^T)^T, and A^T = U_w S V_w^T means A = V_w S U_w^T.
        Ok(SvdDecomposition {
            u: v_sorted,
            s: Vector::from(s_sorted),
            vt: u.transpose(),
        })
    } else {
        Ok(SvdDecomposition { u, s: Vector::from(s_sorted), vt })
    }
}
