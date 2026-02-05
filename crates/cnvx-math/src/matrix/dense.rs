use crate::Matrix;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct DenseMatrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Matrix for DenseMatrix {
    fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols, data: vec![0.0; rows * cols] }
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn get(&self, r: usize, c: usize) -> f64 {
        self.data[r * self.cols + c]
    }

    fn set(&mut self, r: usize, c: usize, v: f64) {
        self.data[r * self.cols + c] = v;
    }

    fn as_vec2(&self) -> Vec<Vec<f64>> {
        (0..self.rows)
            .map(|r| (0..self.cols).map(|c| self.get(r, c)).collect())
            .collect()
    }

    // FIXME: Replace with a more efficient solver
    fn gaussian_elimination(&self, rhs: &mut [f64]) -> Result<(), String> {
        let n = self.rows();
        if rhs.len() != n {
            return Err("rhs length mismatch".into());
        }
        // build augmented matrix
        let mut aug = vec![vec![0.0; n + 1]; n];
        for i in 0..n {
            for j in 0..n {
                aug[i][j] = self.get(i, j);
            }
            aug[i][n] = rhs[i];
        }
        // gaussian elimination with partial pivot
        for col in 0..n {
            // pivot
            let mut pivot = col;
            let mut maxv = aug[pivot][col].abs();
            for r in (col + 1)..n {
                if aug[r][col].abs() > maxv {
                    pivot = r;
                    maxv = aug[r][col].abs();
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
            for k in col..=n {
                aug[col][k] /= diag;
            }
            // eliminate
            for r in 0..n {
                if r == col {
                    continue;
                }
                let fac = aug[r][col];
                if fac.abs() < 1e-15 {
                    continue;
                }
                for k in col..=n {
                    aug[r][k] -= fac * aug[col][k];
                }
            }
        }
        // write back solution
        for i in 0..n {
            rhs[i] = aug[i][n];
        }
        Ok(())
    }
}

impl Index<usize> for DenseMatrix {
    type Output = [f64];

    fn index(&self, row: usize) -> &Self::Output {
        let start = row * self.cols;
        let end = start + self.cols;
        &self.data[start..end] // &slice is fine
    }
}

impl IndexMut<usize> for DenseMatrix {
    fn index_mut(&mut self, row: usize) -> &mut Self::Output {
        let start = row * self.cols;
        let end = start + self.cols;
        &mut self.data[start..end]
    }
}
