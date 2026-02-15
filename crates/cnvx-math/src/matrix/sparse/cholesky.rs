use std::collections::HashMap;

use crate::{
    Matrix,
    matrix::{MatrixSolveMethod, SparseMatrix},
};

pub(crate) struct SparseCholesky<'matrix> {
    matrix: &'matrix SparseMatrix, // borrow reference to the matrix for QR factorization
}

impl<'matrix> MatrixSolveMethod<'matrix, SparseMatrix> for SparseCholesky<'matrix> {
    // Do we want the new() method??
    fn new(matrix: &'matrix SparseMatrix) -> Self {
        Self { matrix }
    }

    // FIXME: This is a very naive implementation of Cholesky factorization. It does not attempt to preserve sparsity and will be very slow for large matrices. A real implementation would use a more sophisticated data structure and algorithm to maintain sparsity.
    fn solve(&self, rhs: &mut [f64]) -> Result<(), String> {
        let n = self.matrix.rows;
        if self.matrix.rows != self.matrix.cols {
            return Err("Cholesky requires a square matrix".into());
        }
        if rhs.len() != n {
            return Err("rhs length mismatch".into());
        }

        // L factor: only store lower triangle
        let mut l: HashMap<(usize, usize), f64> = HashMap::new();

        // Compute L
        for i in 0..n {
            for j in 0..=i {
                let mut sum = self.matrix.get(i, j);

                for k in 0..j {
                    if let (Some(&lik), Some(&ljk)) = (l.get(&(i, k)), l.get(&(j, k))) {
                        sum -= lik * ljk;
                    }
                }

                if i == j {
                    if sum <= 0.0 {
                        return Err("Matrix is not positive definite".into());
                    }
                    l.insert((i, j), sum.sqrt());
                } else {
                    let ljj = *l.get(&(j, j)).unwrap();
                    l.insert((i, j), sum / ljj);
                }
            }
        }

        // Forward substitution
        let mut y = vec![0.0; n];
        for i in 0..n {
            let mut sum = rhs[i];
            for (j, yj) in y.iter().enumerate().take(i) {
                if let Some(&lij) = l.get(&(i, j)) {
                    sum -= lij * yj;
                }
            }
            let lii = *l.get(&(i, i)).unwrap();
            y[i] = sum / lii;
        }

        // Back substitution
        for i in (0..n).rev() {
            let mut sum = y[i];
            for (j, rj) in rhs.iter().enumerate().take(n).skip(i + 1) {
                if let Some(&lji) = l.get(&(j, i)) {
                    sum -= lji * rj;
                }
            }
            let lii = *l.get(&(i, i)).unwrap();
            rhs[i] = sum / lii;
        }

        Ok(())
    }
}
