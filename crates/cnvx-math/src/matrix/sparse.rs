use crate::matrix::Matrix;

#[derive(Debug, Clone)]
pub struct SparseMatrix {}

#[allow(unused)]
impl Matrix for SparseMatrix {
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

    fn add(&self, other: &Self) -> Result<Self, String> {
        unimplemented!()
    }

    fn sub(&self, other: &Self) -> Result<Self, String> {
        unimplemented!()
    }

    fn mul(&self, other: &Self) -> Result<Self, String> {
        unimplemented!()
    }

    fn mul_vec(&self, rhs: &[f64]) -> Result<Vec<f64>, String> {
        unimplemented!()
    }

    fn add_scalar(&self, scalar: f64) -> Self {
        unimplemented!()
    }

    fn sub_scalar(&self, scalar: f64) -> Self {
        unimplemented!()
    }

    fn mul_scalar(&self, scalar: f64) -> Self {
        unimplemented!()
    }

    fn div_scalar(&self, scalar: f64) -> Self {
        unimplemented!()
    }

    fn mul_elementwise(&self, other: &Self) -> Result<Self, String> {
        unimplemented!()
    }

    fn get_row(&self, row: usize) -> Vec<f64> {
        unimplemented!()
    }

    fn get_col(&self, col: usize) -> Vec<f64> {
        unimplemented!()
    }

    fn set_row(&mut self, row: usize, vec: &[f64]) -> Result<(), String> {
        unimplemented!()
    }

    fn set_col(&mut self, col: usize, vec: &[f64]) -> Result<(), String> {
        unimplemented!()
    }

    fn swap_rows(&mut self, row1: usize, row2: usize) {
        unimplemented!()
    }

    fn norm_inf(&self) -> f64 {
        unimplemented!()
    }

    fn transpose(&self) -> Self {
        unimplemented!()
    }

    fn identity(size: usize) -> Self {
        unimplemented!()
    }

    fn from_diagonal(vec: &[f64]) -> Self {
        unimplemented!()
    }

    fn diagonal(&self) -> Vec<f64> {
        unimplemented!()
    }

    fn mldivide(&self, rhs: &[f64]) -> Result<Vec<f64>, String> {
        unimplemented!()
    }
}
