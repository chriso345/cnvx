use crate::{DenseMatrix, Matrix, matrix::MatrixSolveMethod};

pub(crate) struct DenseGaussElim<'matrix> {
    matrix: &'matrix DenseMatrix, // borrow reference to the matrix for QR factorization
}

impl<'matrix> MatrixSolveMethod<'matrix, DenseMatrix> for DenseGaussElim<'matrix> {
    // Do we want the new() method??
    fn new(matrix: &'matrix DenseMatrix) -> Self {
        Self { matrix }
    }

    // FIXME: This is a very naive implementation of QR factorization.
    fn solve(&self, rhs: &mut [f64]) -> Result<(), String> {
        let n = self.matrix.rows;
        if rhs.len() != n {
            return Err("rhs length mismatch".into());
        }
        // build augmented matrix
        let mut aug = vec![vec![0.0; n + 1]; n];
        for (i, row) in aug.iter_mut().enumerate().take(n) {
            for (j, cell) in row.iter_mut().take(n).enumerate() {
                *cell = self.matrix.get(i, j);
            }
            row[n] = rhs[i];
        }
        // gaussian elimination with partial pivot
        for col in 0..n {
            // pivot
            let mut pivot = col;
            let mut maxv = aug[pivot][col].abs();
            for (r, row) in aug.iter().enumerate().skip(col + 1) {
                if row[col].abs() > maxv {
                    pivot = r;
                    maxv = row[col].abs();
                }
            }
            if maxv < 1e-12 {
                return Err("singular matrix".into());
            }
            if pivot != col {
                aug.swap(pivot, col);
            }
            // normalize
            let diag = aug[col][col];
            aug[col].iter_mut().skip(col).for_each(|v| *v /= diag);
            // capture pivot row to avoid borrowing issues
            let pivot_row = aug[col].clone();
            // eliminate
            for (r, row) in aug.iter_mut().enumerate().take(n) {
                if r == col {
                    continue;
                }
                let fac = row[col];
                if fac.abs() < 1e-15 {
                    continue;
                }
                for (k, val) in row.iter_mut().enumerate().skip(col) {
                    *val -= fac * pivot_row[k];
                }
            }
        }
        // write back solution
        for (i, row) in aug.iter().enumerate().take(n) {
            rhs[i] = row[n];
        }
        Ok(())
    }
}
