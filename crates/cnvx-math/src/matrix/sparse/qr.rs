use std::collections::HashMap;

use crate::matrix::{MatrixSolveMethod, SparseMatrix};

pub(crate) struct SparseQR<'matrix> {
    matrix: &'matrix SparseMatrix, // borrow reference to the matrix for QR factorization
}

impl<'matrix> MatrixSolveMethod<'matrix, SparseMatrix> for SparseQR<'matrix> {
    // Do we want the new() method??
    fn new(matrix: &'matrix SparseMatrix) -> Self {
        Self { matrix }
    }

    // FIXME: This is a very naive implementation of QR factorization.
    fn solve(&self, rhs: &mut [f64]) -> Result<(), String> {
        let m = self.matrix.rows;
        let n = self.matrix.cols;

        if rhs.len() != m {
            return Err("rhs length mismatch".into());
        }
        if m < n {
            return Err("QR requires rows >= cols".into());
        }

        // R stored sparsely (upper triangular)
        let mut r: HashMap<(usize, usize), f64> = HashMap::new();

        // Working copy of rhs for Q'b
        let mut b = rhs.to_vec();

        // Copy A into R initially
        for (&(i, j), &v) in &self.matrix.data {
            r.insert((i, j), v);
        }

        // Householder reflections
        for k in 0..n {
            // Compute norm of column k below diagonal
            let mut norm = 0.0;
            for i in k..m {
                if let Some(&v) = r.get(&(i, k)) {
                    norm += v * v;
                }
            }
            norm = norm.sqrt();
            if norm < 1e-12 {
                return Err("Matrix is rank deficient".into());
            }

            let rkk = *r.get(&(k, k)).unwrap_or(&0.0);
            let alpha = if rkk >= 0.0 { -norm } else { norm };

            // Build Householder vector v
            let mut v: HashMap<usize, f64> = HashMap::new();
            v.insert(k, rkk - alpha);

            for i in (k + 1)..m {
                if let Some(&val) = r.get(&(i, k)) {
                    v.insert(i, val);
                }
            }

            let mut vtv = 0.0;
            for &val in v.values() {
                vtv += val * val;
            }
            let beta = 2.0 / vtv;

            // Apply reflector to R (left multiply)
            for j in k..n {
                let mut dot = 0.0;
                for (&i, &vi) in &v {
                    if let Some(&rij) = r.get(&(i, j)) {
                        dot += vi * rij;
                    }
                }
                for (&i, &vi) in &v {
                    let entry = r.entry((i, j)).or_insert(0.0);
                    *entry -= beta * vi * dot;
                    if entry.abs() < 1e-14 {
                        r.remove(&(i, j));
                    }
                }
            }

            // Apply reflector to b
            let mut dot = 0.0;
            for (&i, &vi) in &v {
                dot += vi * b[i];
            }
            for (&i, &vi) in &v {
                b[i] -= beta * vi * dot;
            }

            // Force exact zero below diagonal
            for i in (k + 1)..m {
                r.remove(&(i, k));
            }
            r.insert((k, k), alpha);
        }

        // Back substitution: solve Rx = Qᵀb
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sum = b[i];
            for j in (i + 1)..n {
                if let Some(&rij) = r.get(&(i, j)) {
                    sum -= rij * x[j];
                }
            }
            let rii = *r.get(&(i, i)).unwrap();
            if rii.abs() < 1e-14 {
                return Err("Singular R matrix".into());
            }
            x[i] = sum / rii;
        }

        rhs[..n].copy_from_slice(&x);
        Ok(())
    }
}
