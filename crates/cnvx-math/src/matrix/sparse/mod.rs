use crate::Matrix;

mod cholesky;
mod qr;
// use crate::qr::SparseQR;

use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

/// Sparse matrix storing only non-zero entries.
/// Keys are (row, col) pairs.
#[derive(Debug, Clone)]
pub struct SparseMatrix {
    rows: usize,
    cols: usize,
    data: HashMap<(usize, usize), f64>,
}

impl Matrix for SparseMatrix {
    fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols, data: HashMap::new() }
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn get(&self, r: usize, c: usize) -> f64 {
        *self.data.get(&(r, c)).unwrap_or(&0.0)
    }

    fn set(&mut self, r: usize, c: usize, v: f64) {
        if v.abs() < 1e-15 {
            // keep matrix sparse
            self.data.remove(&(r, c));
        } else {
            self.data.insert((r, c), v);
        }
    }

    fn matvec(&self, x: &[f64]) -> Vec<f64> {
        if x.len() != self.cols {
            panic!("input vector length mismatch");
        }
        let mut result = vec![0.0; self.rows];
        for (&(r, c), &v) in &self.data {
            result[r] += v * x[c];
        }
        result
    }

    fn mldivide(&self, rhs: &mut [f64]) -> Result<(), String> {
        // https://au.mathworks.com/help/matlab/ref/mldivide_sparse.png

        if self.rows != self.cols { self.qr(rhs) } else { self.cholesky(rhs) }
    }

    fn as_vec2(&self) -> Vec<Vec<f64>> {
        let mut result = vec![vec![0.0; self.cols]; self.rows];
        for (&(r, c), &v) in &self.data {
            result[r][c] = v;
        }
        result
    }

    fn zeros(rows: usize, cols: usize) -> Self {
        Self::new(rows, cols)
    }
}

impl Index<usize> for SparseMatrix {
    type Output = [f64];

    fn index(&self, _: usize) -> &Self::Output {
        panic!(
            "Indexing by row is not supported for SparseMatrix. Use get(r, c) instead."
        )
    }
}

impl IndexMut<usize> for SparseMatrix {
    fn index_mut(&mut self, _: usize) -> &mut Self::Output {
        panic!(
            "Indexing by row is not supported for SparseMatrix. Use set(r, c, v) instead."
        )
    }
}
