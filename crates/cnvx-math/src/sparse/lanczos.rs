use super::IterativeOptions;
use crate::error::MathError;
use crate::linalg::EigenDecomposition;
use crate::matrix::Matrix;
use crate::sparse::SparseMatrix;
use crate::vector::Vector;

/// Runs `steps` iterations of the Lanczos algorithm on symmetric `a`
/// starting from an arbitrary initial vector, returning the tridiagonal Lanczos
/// matrix's diagonal (`alpha`) and off-diagonal (`beta`, length `steps - 1`)
/// together with the orthonormal Lanczos basis vectors (as columns of a dense
/// `n x steps` `Matrix`).
///
/// Full reorthogonalization (against every previous basis vector, not
/// just the immediately preceding two) is used each step: plain Lanczos
/// loses orthogonality after enough iterations due to floating-point
/// error, which without correction shows up as spurious duplicate
/// ("ghost") Ritz values. Reorthogonalizing costs `O(steps)` extra work
/// per step, but `steps` is small relative to `n` for the intended use
/// case (a handful of eigenpairs from a large sparse matrix).
fn lanczos_tridiagonalize(
    a: &SparseMatrix,
    steps: usize,
) -> (Vec<f64>, Vec<f64>, Matrix) {
    let n = a.rows();
    let mut v_prev = Vector::zeros(n);
    let mut basis: Vec<Vector> = Vec::with_capacity(steps);

    // A fixed, deterministic starting vector (rather than a random one)
    // so results are reproducible across runs without needing an `Rng`.
    let mut v: Vector = (0..n).map(|i| (i as f64 + 1.0).sin()).collect();
    let v_norm = v.norm();
    v = &v * (1.0 / v_norm.max(1e-300));

    let mut alpha = Vec::with_capacity(steps);
    let mut beta = Vec::with_capacity(steps.saturating_sub(1));
    let mut beta_prev = 0.0;

    for _ in 0..steps {
        let mut w = a.mul_vec(&v);
        let a_j = v.dot(&w);
        w = &w - &(&v * a_j);
        if !basis.is_empty() {
            w = &w - &(&v_prev * beta_prev);
        }
        // Full reorthogonalization against every prior basis vector.
        for prior in &basis {
            let proj = prior.dot(&w);
            w = &w - &(prior * proj);
        }

        alpha.push(a_j);
        basis.push(v.clone());

        let b_j = w.norm();
        if b_j < 1e-12 {
            break;
        }
        beta.push(b_j);
        v_prev = v;
        v = &w * (1.0 / b_j);
        beta_prev = b_j;
    }

    let m = basis.len();
    let mut basis_mat = Matrix::zeros(n, m);
    for (col, bv) in basis.iter().enumerate() {
        for row in 0..n {
            basis_mat.set(row, col, bv[row]);
        }
    }
    (alpha, beta, basis_mat)
}

/// Builds the dense `m x m` tridiagonal matrix with diagonal `alpha` and
/// off-diagonal `beta`.
fn tridiagonal_matrix(alpha: &[f64], beta: &[f64]) -> Matrix {
    let m = alpha.len();
    let mut t = Matrix::zeros(m, m);
    for i in 0..m {
        t.set(i, i, alpha[i]);
        if i + 1 < m {
            t.set(i, i + 1, beta[i]);
            t.set(i + 1, i, beta[i]);
        }
    }
    t
}

/// Computes `k` extreme eigenpairs of symmetric sparse `a` via Lanczos,
/// taking the `k` Ritz values with the largest or smallest magnitude sign
/// (`smallest = true` for the `k` smallest, `false` for the `k` largest)
/// from a tridiagonal projection built over `options.max_iter` steps.
///
/// Ritz vectors are formed by projecting the tridiagonal matrix's
/// eigenvectors back through the Lanczos basis (`vectors = basis *
/// tridiagonal_eigenvectors`). They approximate true eigenvectors of `A`
/// to the same accuracy the corresponding Ritz value approximates the
/// true eigenvalue.
///
/// # Errors
/// Returns [`MathError::DimensionMismatch`] if `a` isn't square, and
/// [`MathError::Unsupported`] if `k` exceeds the number of Lanczos steps
/// actually completed (e.g. because the Krylov subspace was exhausted
/// early on a low-rank or small matrix).
fn extreme_eigenpairs(
    a: &SparseMatrix,
    k: usize,
    options: &IterativeOptions,
    smallest: bool,
) -> Result<EigenDecomposition, MathError> {
    if a.rows() != a.cols() {
        return Err(MathError::DimensionMismatch(
            "Lanczos requires a square matrix".into(),
        ));
    }
    let steps = options.max_iter as usize;
    let (alpha, beta, basis) = lanczos_tridiagonalize(a, steps.max(k + 1).min(a.rows()));
    let m = alpha.len();
    if k > m {
        return Err(MathError::Unsupported(format!(
            "requested {k} eigenpairs but the Lanczos process only produced {m} steps"
        )));
    }

    let t = tridiagonal_matrix(&alpha, &beta);
    let eig = t.eigen_symmetric()?; // Already sorted descending.

    let indices: Vec<usize> =
        if smallest { (0..m).rev().take(k).collect() } else { (0..m).take(k).collect() };

    let mut values = vec![0.0; k];
    let mut vectors = Matrix::zeros(a.rows(), k);
    for (out_col, &src_col) in indices.iter().enumerate() {
        values[out_col] = eig.values[src_col];
        // Ritz vector: basis * (tridiagonal eigenvector column src_col).
        let coeffs = eig.vectors.get_col(src_col);
        let ritz = basis.mul_vec(&coeffs);
        for row in 0..a.rows() {
            vectors.set(row, out_col, ritz[row]);
        }
    }

    Ok(EigenDecomposition {
        values: Vector::from(values),
        vectors,
        imag_values: None,
    })
}

/// The `k` smallest eigenpairs of symmetric sparse `a`, via Lanczos.
///
/// # Errors
/// See [`extreme_eigenpairs`].
pub fn smallest_eigenpairs(
    a: &SparseMatrix,
    k: usize,
    options: &IterativeOptions,
) -> Result<EigenDecomposition, MathError> {
    extreme_eigenpairs(a, k, options, true)
}

/// The `k` largest eigenpairs of symmetric sparse `a`, via Lanczos.
///
/// # Errors
/// See [`extreme_eigenpairs`].
pub fn largest_eigenpairs(
    a: &SparseMatrix,
    k: usize,
    options: &IterativeOptions,
) -> Result<EigenDecomposition, MathError> {
    extreme_eigenpairs(a, k, options, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tridiag_test_matrix(n: usize) -> SparseMatrix {
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

    #[test]
    fn largest_eigenpair_matches_dense() {
        let n = 12;
        let a = tridiag_test_matrix(n);
        let dense_eig = a.to_dense().eigen_symmetric().unwrap();

        let options = IterativeOptions { max_iter: n as u32, ..Default::default() };
        let result = largest_eigenpairs(&a, 2, &options).unwrap();

        assert!((result.values[0] - dense_eig.values[0]).abs() < 1e-6);
        assert!((result.values[1] - dense_eig.values[1]).abs() < 1e-6);
    }

    #[test]
    fn smallest_eigenpair_matches_dense() {
        let n = 12;
        let a = tridiag_test_matrix(n);
        let dense_eig = a.to_dense().eigen_symmetric().unwrap();
        let n_dense = dense_eig.values.len();

        let options = IterativeOptions { max_iter: n as u32, ..Default::default() };
        let result = smallest_eigenpairs(&a, 2, &options).unwrap();

        assert!((result.values[0] - dense_eig.values[n_dense - 1]).abs() < 1e-6);
        assert!((result.values[1] - dense_eig.values[n_dense - 2]).abs() < 1e-6);
    }

    #[test]
    fn ritz_vector_is_approximate_eigenvector() {
        let n = 10;
        let a = tridiag_test_matrix(n);
        let options = IterativeOptions { max_iter: n as u32, ..Default::default() };
        let result = largest_eigenpairs(&a, 1, &options).unwrap();

        let v = result.vectors.get_col(0);
        let av = a.mul_vec(&v);
        let lambda_v = &v * result.values[0];
        let residual = (&av - &lambda_v).norm();
        assert!(residual < 1e-5, "residual = {residual}");
    }
}
