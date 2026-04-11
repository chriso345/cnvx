use crate::matrix::MatrixWrapper;

// TODO: This will either use sprs or nalgebra_sparse csr format
#[derive(Debug, Clone)]
pub struct ExposedSparseMatrix {}

#[allow(unused)]
impl MatrixWrapper for ExposedSparseMatrix {
    fn new(rows: usize, cols: usize) -> Self {
        unimplemented!()
    }

    fn rows(&self) -> usize {
        unimplemented!()
    }

    fn cols(&self) -> usize {
        unimplemented!()
    }

    fn get(&self, row: usize, col: usize) -> f64 {
        unimplemented!()
    }

    fn set(&mut self, row: usize, col: usize, value: f64) {
        unimplemented!()
    }

    fn mldivide(&self, rhs: &mut [f64]) -> Result<(), String> {
        unimplemented!()
    }
}
