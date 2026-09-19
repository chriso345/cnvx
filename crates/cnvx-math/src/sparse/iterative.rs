use super::preconditioner::{Preconditioner, Prepared};
use crate::error::MathError;
use crate::sparse::SparseMatrix;
use crate::vector::Vector;

#[derive(Debug, Clone, PartialEq)]
pub struct IterativeOptions {
    /// Stop once the relative residual `||b - A x|| / ||b||` drops below
    /// this value.
    pub tol: f64,
    /// The maximum number of iterations.
    pub max_iter: u32,
    /// Which preconditioner to apply.
    pub preconditioner: Preconditioner,
}

impl Default for IterativeOptions {
    fn default() -> Self {
        Self {
            tol: 1e-8,
            max_iter: 1000,
            preconditioner: Preconditioner::None,
        }
    }
}

/// The outcome of a converged iterative solve.
#[derive(Debug, Clone, PartialEq)]
pub struct IterativeResult {
    /// The approximate solution.
    pub x: Vector,
    /// The number of iterations actually performed.
    pub iterations: u32,
    /// The final relative residual `||b - A x|| / ||b||`.
    pub residual_norm: f64,
}

/// The conjugate gradient method, for symmetric positive-definite `A`.
///
/// The standard choice for SPD systems: guaranteed to converge in at
/// most `n` iterations in exact arithmetic (and usually far fewer with a
/// good preconditioner), using only one matrix-vector product per
/// iteration and a handful of vector operations.
///
/// # Errors
/// Returns [`MathError::DimensionMismatch`] if `rhs.len() != a.rows()`,
/// and [`MathError::NotConverged`] if `options.max_iter` is reached
/// without the residual dropping below `options.tol`.
pub fn conjugate_gradient(
    a: &SparseMatrix,
    rhs: &Vector,
    options: &IterativeOptions,
) -> Result<IterativeResult, MathError> {
    if rhs.len() != a.rows() {
        return Err(MathError::DimensionMismatch(format!(
            "matrix has {} rows, right-hand side has {} entries",
            a.rows(),
            rhs.len()
        )));
    }
    let n = a.rows();
    let precond = Prepared::build(options.preconditioner, a)?;
    let b_norm = rhs.norm().max(1e-300);

    let mut x = vec![0.0; n];
    let mut r = rhs.clone();
    let mut z = precond.apply(&r);
    let mut p = z.clone();
    let mut rz_old = r.dot(&z);

    let mut iterations = 0;
    let mut residual_norm = r.norm() / b_norm;
    if residual_norm < options.tol {
        return Ok(IterativeResult { x: Vector::from(x), iterations, residual_norm });
    }

    for iter in 1..=options.max_iter {
        iterations = iter;
        let ap = a.mul_vec(&p);
        let denom = p.dot(&ap);
        if denom.abs() < 1e-300 {
            return Err(MathError::Singular);
        }
        let alpha = rz_old / denom;
        for i in 0..n {
            x[i] += alpha * p[i];
        }
        r = &r - &(&ap * alpha);
        residual_norm = r.norm() / b_norm;
        if residual_norm < options.tol {
            break;
        }
        z = precond.apply(&r);
        let rz_new = r.dot(&z);
        let beta = rz_new / rz_old;
        p = &z + &(&p * beta);
        rz_old = rz_new;
    }

    if residual_norm >= options.tol {
        return Err(MathError::NotConverged(options.max_iter));
    }
    Ok(IterativeResult { x: Vector::from(x), iterations, residual_norm })
}

/// GMRES (generalized minimal residual method), for general (possibly
/// non-symmetric) `A`, restarted every `restart` iterations (or every
/// `n` iterations, whichever is smaller, capped at 50 to bound memory)
/// to keep the Krylov basis from growing without limit.
///
/// # Errors
/// Returns [`MathError::DimensionMismatch`] if `rhs.len() != a.rows()`,
/// and [`MathError::NotConverged`] if `options.max_iter` total iterations
/// are reached without the residual dropping below `options.tol`.
pub fn gmres(
    a: &SparseMatrix,
    rhs: &Vector,
    options: &IterativeOptions,
) -> Result<IterativeResult, MathError> {
    if rhs.len() != a.rows() {
        return Err(MathError::DimensionMismatch(format!(
            "matrix has {} rows, right-hand side has {} entries",
            a.rows(),
            rhs.len()
        )));
    }
    let n = a.rows();
    let precond = Prepared::build(options.preconditioner, a)?;
    let restart = n.clamp(1, 50);
    let b_norm = rhs.norm().max(1e-300);

    let mut x = Vector::zeros(n);
    let mut total_iters = 0u32;
    let mut residual_norm;

    loop {
        let r0 = rhs - &a.mul_vec(&x);
        let beta = r0.norm();
        residual_norm = beta / b_norm;
        if residual_norm < options.tol {
            return Ok(IterativeResult { x, iterations: total_iters, residual_norm });
        }

        // Arnoldi process (modified Gram-Schmidt) building an
        // orthonormal Krylov basis `q_1..q_m`, with the Hessenberg
        // matrix reduced to upper-triangular via Givens rotations as
        // each column is produced (so the least-squares problem is
        // solved incrementally rather than only at the end).
        let m = restart.min((options.max_iter - total_iters) as usize);
        if m == 0 {
            break;
        }
        let mut q: Vec<Vector> = Vec::with_capacity(m + 1);
        q.push(&r0 * (1.0 / beta));
        let mut h = vec![vec![0.0; m]; m + 1];
        let mut cs = vec![0.0; m];
        let mut sn = vec![0.0; m];
        let mut g = vec![0.0; m + 1];
        g[0] = beta;

        let mut k_used = 0;
        for k in 0..m {
            k_used = k + 1;
            total_iters += 1;
            let y = precond.apply(&q[k]);
            let mut qk1 = a.mul_vec(&y);
            for i in 0..=k {
                h[i][k] = q[i].dot(&qk1);
                qk1 = &qk1 - &(&q[i] * h[i][k]);
            }
            h[k + 1][k] = qk1.norm();
            let arnoldi_beta = h[k + 1][k];

            // Apply previous Givens rotations to the new Hessenberg column.
            for i in 0..k {
                let temp = cs[i] * h[i][k] + sn[i] * h[i + 1][k];
                h[i + 1][k] = -sn[i] * h[i][k] + cs[i] * h[i + 1][k];
                h[i][k] = temp;
            }
            let denom = (h[k][k].powi(2) + h[k + 1][k].powi(2)).sqrt();
            if denom < 1e-300 {
                cs[k] = 1.0;
                sn[k] = 0.0;
            } else {
                cs[k] = h[k][k] / denom;
                sn[k] = h[k + 1][k] / denom;
            }
            h[k][k] = cs[k] * h[k][k] + sn[k] * h[k + 1][k];
            h[k + 1][k] = 0.0;

            let temp = cs[k] * g[k];
            g[k + 1] = -sn[k] * g[k];
            g[k] = temp;

            residual_norm = g[k + 1].abs() / b_norm;
            if arnoldi_beta.abs() > 1e-300 {
                q.push(&qk1 * (1.0 / arnoldi_beta));
            } else {
                // Exact breakdown: the Krylov subspace is invariant under
                // A (no new orthogonal direction to add). The current
                // solution already lies in the span of q[0..=k], so
                // there's nothing more this restart cycle can do.
                break;
            }
            if residual_norm < options.tol || total_iters >= options.max_iter {
                break;
            }
        }

        // Back-substitute for y in the (k_used x k_used) upper-triangular
        // system `h y = g`, then form the update `x += sum(y_i * (M^-1 q_i))`.
        let mut y = vec![0.0; k_used];
        for i in (0..k_used).rev() {
            let mut sum = g[i];
            for j in (i + 1)..k_used {
                sum -= h[i][j] * y[j];
            }
            y[i] = sum / h[i][i];
        }
        let mut update = Vector::zeros(n);
        for (i, &yi) in y.iter().enumerate() {
            update = &update + &(&precond.apply(&q[i]) * yi);
        }
        x = &x + &update;

        if residual_norm < options.tol || total_iters >= options.max_iter {
            break;
        }
    }

    if residual_norm >= options.tol {
        return Err(MathError::NotConverged(options.max_iter));
    }
    Ok(IterativeResult { x, iterations: total_iters, residual_norm })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spd_test_matrix(n: usize) -> SparseMatrix {
        let mut triplets = Vec::new();
        for i in 0..n {
            triplets.push((i, i, 2.0));
            if i > 0 {
                triplets.push((i, i - 1, -1.0));
            }
            if i + 1 < n {
                triplets.push((i, i + 1, -1.0));
            }
        }
        SparseMatrix::from_triplets(n, n, &triplets)
    }

    fn nonsymmetric_test_matrix() -> SparseMatrix {
        // Diagonally dominant but non-symmetric.
        SparseMatrix::from_triplets(
            3,
            3,
            &[
                (0, 0, 4.0),
                (0, 1, 1.0),
                (1, 0, 2.0),
                (1, 1, 5.0),
                (1, 2, 1.0),
                (2, 1, 1.0),
                (2, 2, 6.0),
                (2, 0, 1.0),
            ],
        )
    }

    #[test]
    fn cg_solves_spd_system() {
        let a = spd_test_matrix(20);
        let x_true = Vector::from((0..20).map(|i| (i as f64) * 0.1).collect::<Vec<_>>());
        let b = a.mul_vec(&x_true);
        let result = conjugate_gradient(&a, &b, &IterativeOptions::default()).unwrap();
        for i in 0..20 {
            assert!((result.x[i] - x_true[i]).abs() < 1e-6, "index {i}");
        }
    }

    #[test]
    fn cg_with_jacobi_preconditioner_converges_faster() {
        let a = spd_test_matrix(30);
        let x_true = Vector::ones(30);
        let b = a.mul_vec(&x_true);
        let none = conjugate_gradient(&a, &b, &IterativeOptions::default()).unwrap();
        let jacobi = conjugate_gradient(
            &a,
            &b,
            &IterativeOptions {
                preconditioner: Preconditioner::Jacobi,
                ..Default::default()
            },
        )
        .unwrap();
        // Jacobi preconditioning shouldn't ever need *more* iterations
        // than no preconditioning here.
        assert!(jacobi.iterations <= none.iterations);
    }

    #[test]
    fn cg_with_incomplete_cholesky_converges() {
        let a = spd_test_matrix(15);
        let x_true = Vector::from((0..15).map(|i| i as f64).collect::<Vec<_>>());
        let b = a.mul_vec(&x_true);
        let result = conjugate_gradient(
            &a,
            &b,
            &IterativeOptions {
                preconditioner: Preconditioner::IncompleteCholesky,
                ..Default::default()
            },
        )
        .unwrap();
        for i in 0..15 {
            assert!((result.x[i] - x_true[i]).abs() < 1e-5, "index {i}");
        }
    }

    #[test]
    fn gmres_solves_nonsymmetric_system() {
        let a = nonsymmetric_test_matrix();
        let x_true = Vector::from_slice(&[1.0, 2.0, 3.0]);
        let b = a.mul_vec(&x_true);
        let result = gmres(&a, &b, &IterativeOptions::default()).unwrap();
        for i in 0..3 {
            assert!((result.x[i] - x_true[i]).abs() < 1e-6, "index {i}");
        }
    }

    #[test]
    fn gmres_solves_larger_spd_system() {
        let a = spd_test_matrix(25);
        let x_true = Vector::from((0..25).map(|i| (i as f64).sin()).collect::<Vec<_>>());
        let b = a.mul_vec(&x_true);
        let result = gmres(&a, &b, &IterativeOptions::default()).unwrap();
        for i in 0..25 {
            assert!((result.x[i] - x_true[i]).abs() < 1e-5, "index {i}");
        }
    }

    #[test]
    fn dimension_mismatch_is_reported() {
        let a = spd_test_matrix(5);
        let b = Vector::zeros(3);
        assert!(conjugate_gradient(&a, &b, &IterativeOptions::default()).is_err());
        assert!(gmres(&a, &b, &IterativeOptions::default()).is_err());
    }
}
