use nalgebra::DMatrix;

use crate::matrix::MatrixWrapper;

#[derive(Debug, Clone)]
pub struct ExposedDenseMatrix {
    pub inner: DMatrix<f64>,
}

impl MatrixWrapper for ExposedDenseMatrix {
    fn new(rows: usize, cols: usize) -> Self {
        Self { inner: DMatrix::zeros(rows, cols) }
    }

    fn rows(&self) -> usize {
        self.inner.nrows()
    }

    fn cols(&self) -> usize {
        self.inner.ncols()
    }

    fn get(&self, row: usize, col: usize) -> f64 {
        self.inner[(row, col)]
    }

    fn set(&mut self, row: usize, col: usize, value: f64) {
        self.inner[(row, col)] = value;
    }

    // TODO: This is a very naive implementation, and will need to be improved for performance depending on the matrix shape and sparsity.
    fn mldivide(&self, rhs: &mut [f64]) -> Result<(), String> {
        let a = &self.inner;
        let b = DMatrix::from_column_slice(rhs.len(), 1, rhs);
        match a.clone().lu().solve(&b) {
            Some(solution) => {
                for i in 0..rhs.len() {
                    rhs[i] = solution[(i, 0)];
                }
                Ok(())
            }
            None => Err("Matrix is singular or not square".to_string()),
        }
    }
}
