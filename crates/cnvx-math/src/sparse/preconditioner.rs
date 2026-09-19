use crate::error::MathError;
use crate::sparse::SparseMatrix;
use crate::vector::Vector;

/// The choice of preconditioner for [`super::IterativeOptions`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Preconditioner {
    /// No preconditioning (`M = I`).
    #[default]
    None,
    /// Jacobi (diagonal) preconditioning: `M = diag(A)`. Cheap to build
    /// and apply, and effective when `A` is diagonally dominant.
    Jacobi,
    /// Incomplete Cholesky, `IC(0)`: a sparse approximate Cholesky factor
    /// that only fills in entries where `A` itself is already non-zero.
    /// More expensive to build than Jacobi, but is a much stronger
    /// preconditioner for symmetric positive-definite systems.
    IncompleteCholesky,
}

/// The matrix-dependent data built from a [`Preconditioner`] choice, used
/// internally by the iterative solvers to apply `M^-1 r` each iteration.
pub(crate) enum Prepared {
    None,
    Jacobi(Vec<f64>),
    /// The sparse lower-triangular `IC(0)` factor `L` (`A ~= L L^T`),
    /// stored as CSR (so its rows are `L`'s rows).
    IncompleteCholesky(SparseMatrix),
}

impl Prepared {
    pub(crate) fn build(
        kind: Preconditioner,
        a: &SparseMatrix,
    ) -> Result<Self, MathError> {
        match kind {
            Preconditioner::None => Ok(Prepared::None),
            Preconditioner::Jacobi => {
                let n = a.rows();
                let mut inv_diag = vec![0.0; n];
                for (i, inv_diag_i) in inv_diag.iter_mut().enumerate().take(n) {
                    let d = a.get(i, i);
                    if d.abs() < 1e-300 {
                        return Err(MathError::Singular);
                    }
                    *inv_diag_i = 1.0 / d;
                }
                Ok(Prepared::Jacobi(inv_diag))
            }
            Preconditioner::IncompleteCholesky => {
                Ok(Prepared::IncompleteCholesky(incomplete_cholesky(a)?))
            }
        }
    }

    /// Applies `M^-1` to `r`, i.e. solves `M z = r` for `z`.
    pub(crate) fn apply(&self, r: &Vector) -> Vector {
        match self {
            Prepared::None => r.clone(),
            Prepared::Jacobi(inv_diag) => Vector::from(
                r.iter().zip(inv_diag).map(|(&ri, &di)| ri * di).collect::<Vec<_>>(),
            ),
            Prepared::IncompleteCholesky(l) => {
                // Solve L y = r (forward substitution), then L^T z = y
                // (backward substitution), since M = L L^T.
                let n = l.rows();
                let mut y = vec![0.0; n];
                for i in 0..n {
                    let mut sum = r[i];
                    for (col, val) in l.row_entries(i) {
                        if col < i {
                            sum -= val * y[col];
                        }
                    }
                    let diag = l.get(i, i);
                    y[i] = sum / diag;
                }
                let mut z = vec![0.0; n];
                for i in (0..n).rev() {
                    let mut sum = y[i];
                    // L^T's row i is L's column i: entries (k, i) with k > i.
                    for (k, &zk) in z.iter().enumerate().skip(i + 1).take(n - i - 1) {
                        let val = l.get(k, i);
                        if val != 0.0 {
                            sum -= val * zk;
                        }
                    }
                    let diag = l.get(i, i);
                    z[i] = sum / diag;
                }
                Vector::from(z)
            }
        }
    }
}

/// Computes the `IC(0)` factor of a symmetric positive-definite sparse
/// matrix: a sparse lower-triangular `L` with the same non-zero pattern
/// as `A`'s lower triangle, such that `L L^T` approximates `A`.
fn incomplete_cholesky(a: &SparseMatrix) -> Result<SparseMatrix, MathError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(MathError::DimensionMismatch(
            "incomplete Cholesky requires a square matrix".into(),
        ));
    }

    // Work in a dense-per-row sparse accumulator keyed by column, since
    // IC(0) only ever touches entries already present in A's sparsity
    // pattern (so the working set per row is small even though we index
    // into it by column number).
    let mut l_rows: Vec<Vec<(usize, f64)>> = (0..n)
        .map(|i| a.row_entries(i).filter(|&(c, _)| c <= i).collect())
        .collect();

    for i in 0..n {
        // Diagonal.
        let mut diag = l_rows[i]
            .iter()
            .find(|&&(c, _)| c == i)
            .map(|&(_, v)| v)
            .unwrap_or(0.0);
        for &(k, lik) in l_rows[i].iter() {
            if k < i {
                diag -= lik * lik;
            }
        }
        if diag <= 0.0 {
            return Err(MathError::NotPositiveDefinite);
        }
        let diag_sqrt = diag.sqrt();
        for entry in l_rows[i].iter_mut() {
            if entry.0 == i {
                entry.1 = diag_sqrt;
            }
        }

        // Off-diagonal entries L[j][i] for j > i where A[j][i] != 0.
        for j in (i + 1)..n {
            let a_ji = l_rows[j].iter().find(|&&(c, _)| c == i);
            let Some(&(_, a_ji_val)) = a_ji else { continue };
            let mut sum = a_ji_val;
            // sum -= dot(L[i][0..i], L[j][0..i])
            for &(k, lik) in l_rows[i].iter() {
                if k < i
                    && let Some(&(_, ljk)) = l_rows[j].iter().find(|&&(c, _)| c == k)
                {
                    sum -= lik * ljk;
                }
            }
            let value = sum / diag_sqrt;
            if let Some(entry) = l_rows[j].iter_mut().find(|(c, _)| *c == i) {
                entry.1 = value;
            }
        }
    }

    let triplets: Vec<(usize, usize, f64)> = l_rows
        .into_iter()
        .enumerate()
        .flat_map(|(i, row)| row.into_iter().map(move |(c, v)| (i, c, v)))
        .collect();
    Ok(SparseMatrix::from_triplets(n, n, &triplets))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spd_test_matrix() -> SparseMatrix {
        // A tridiagonal SPD matrix: 2 on the diagonal, -1 off-diagonal.
        let n = 5;
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
    fn jacobi_matches_diagonal() {
        let a = spd_test_matrix();
        let prepared = Prepared::build(Preconditioner::Jacobi, &a).unwrap();
        let r = Vector::from_slice(&[2.0, 2.0, 2.0, 2.0, 2.0]);
        let z = prepared.apply(&r);
        for i in 0..5 {
            assert!((z[i] - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn incomplete_cholesky_reconstructs_tridiagonal_exactly() {
        // For a tridiagonal matrix, IC(0) has no missing fill-in
        // compared to full Cholesky, so it should be an exact
        // factorization.
        let a = spd_test_matrix();
        let l = incomplete_cholesky(&a).unwrap();
        let dense_l = l.to_dense();
        let reconstructed = dense_l.mul(&dense_l.transpose()).unwrap();
        for i in 0..5 {
            for j in 0..5 {
                assert!(
                    (reconstructed.get(i, j) - a.get(i, j)).abs() < 1e-9,
                    "mismatch at ({i},{j})"
                );
            }
        }
    }
}
