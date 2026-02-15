use std::collections::HashMap;

use crate::{Matrix, matrix::SparseMatrix};

impl SparseMatrix {
    pub fn cholesky(&self, rhs: &mut [f64]) -> Result<(), String> {
        let n = self.rows;
        if self.rows != self.cols {
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
                let mut sum = self.get(i, j);

                // sum -= Σ_k L[i,k] * L[j,k]
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

        // Forward substitution: L y = rhs
        let mut y = vec![0.0; n];
        for i in 0..n {
            let mut sum = rhs[i];
            for j in 0..i {
                if let Some(&lij) = l.get(&(i, j)) {
                    sum -= lij * y[j];
                }
            }
            let lii = *l.get(&(i, i)).unwrap();
            y[i] = sum / lii;
        }

        // Back substitution: Lᵀ x = y
        for i in (0..n).rev() {
            let mut sum = y[i];
            for j in (i + 1)..n {
                if let Some(&lji) = l.get(&(j, i)) {
                    sum -= lji * rhs[j];
                }
            }
            let lii = *l.get(&(i, i)).unwrap();
            rhs[i] = sum / lii;
        }

        Ok(())
    }
}
