use std::ops::{Index, IndexMut};

use crate::{
    Matrix,
    matrix::{MatrixSolveMethod, dense::gauss_elim::DenseGaussElim},
};

mod gauss_elim;

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

    fn matvec(&self, x: &[f64]) -> Vec<f64> {
        if x.len() != self.cols {
            panic!("input vector length mismatch");
        }
        let mut result = vec![0.0; self.rows];
        for (r, row) in result.iter_mut().enumerate().take(self.rows) {
            let mut sum = 0.0;
            for (c, xc) in x.iter().enumerate().take(self.cols) {
                sum += self.get(r, c) * xc;
            }
            *row = sum;
        }
        result
    }

    fn mldivide(&self, rhs: &mut [f64]) -> Result<(), String> {
        // https://mathworks.com/help/matlab/ref/mldivide_full.png
        let solver: Box<dyn MatrixSolveMethod<DenseMatrix>> =
            Box::new(DenseGaussElim::new(self));
        solver.solve(rhs)
    }

    fn as_vec2(&self) -> Vec<Vec<f64>> {
        (0..self.rows)
            .map(|r| (0..self.cols).map(|c| self.get(r, c)).collect())
            .collect()
    }

    fn zeros(rows: usize, cols: usize) -> Self {
        Self::new(rows, cols)
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
