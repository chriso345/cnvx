use std::ops::{Index, IndexMut};

mod dense;

pub use dense::DenseMatrix;

pub trait Matrix: Index<usize, Output = [f64]> + IndexMut<usize, Output = [f64]> {
    fn new(rows: usize, cols: usize) -> Self
    where
        Self: Sized;

    fn rows(&self) -> usize;
    fn cols(&self) -> usize;

    fn get(&self, row: usize, col: usize) -> f64;
    fn set(&mut self, row: usize, col: usize, value: f64);

    fn eye(rows: usize) -> Self
    where
        Self: Sized,
    {
        let mut m = Self::new(rows, rows);
        for i in 0..rows {
            m.set(i, i, 1.0);
        }
        m
    }

    fn as_vec2(&self) -> Vec<Vec<f64>>; // optional, for debugging or conversion

    /// Solve a square linear system Ax = rhs using Gaussian elimination.
    /// On success rhs is overwritten with the solution vector x.
    fn gaussian_elimination(&self, rhs: &mut [f64]) -> Result<(), String>
    where
        Self: Sized;
}
