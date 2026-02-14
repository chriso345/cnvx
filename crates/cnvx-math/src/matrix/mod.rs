use std::ops::{Index, IndexMut};

mod dense;
pub use dense::DenseMatrix;

/// A generic matrix trait for linear algebra operations.
///
/// This trait defines the interface for matrices used in the LP solver,
/// including creation, element access, and solving linear systems.
///
/// Implementors must provide row/column indexing, element access, and
/// Gaussian elimination for square systems.
pub trait Matrix:
    Index<usize, Output = [f64]> + IndexMut<usize, Output = [f64]> + Clone
{
    /// Create a new matrix with the given number of rows and columns,
    /// initialized with zeros.
    ///
    /// # Example
    /// ```
    /// # use cnvx_math::{DenseMatrix, Matrix};
    /// let m = DenseMatrix::new(2, 3);
    /// assert_eq!(m.rows(), 2);
    /// assert_eq!(m.cols(), 3);
    /// ```
    fn new(rows: usize, cols: usize) -> Self
    where
        Self: Sized;

    /// Return the number of rows in the matrix.
    fn rows(&self) -> usize;

    /// Return the number of columns in the matrix.
    fn cols(&self) -> usize;

    /// Get the element at position `(row, col)`.
    ///
    /// # Panics
    /// Panics if `row` or `col` are out of bounds.
    fn get(&self, row: usize, col: usize) -> f64;

    /// Set the element at position `(row, col)` to `value`.
    ///
    /// # Panics
    /// Panics if `row` or `col` are out of bounds.
    fn set(&mut self, row: usize, col: usize, value: f64);

    /// Create an identity matrix of the given size.
    ///
    /// # Example
    /// ```
    /// # use cnvx_math::{DenseMatrix, Matrix};
    /// let eye = DenseMatrix::eye(3);
    /// assert_eq!(eye.get(0, 0), 1.0);
    /// assert_eq!(eye.get(0, 1), 0.0);
    /// ```
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

    /// Return the matrix as a 2D `Vec` for debugging or conversion purposes.
    ///
    /// # Example
    /// ```
    /// # use cnvx_math::{DenseMatrix, Matrix};
    /// let m = DenseMatrix::new(2, 2);
    /// let v = m.as_vec2();
    /// assert_eq!(v.len(), 2);
    /// assert_eq!(v[0].len(), 2);
    /// ```
    fn as_vec2(&self) -> Vec<Vec<f64>>;

    /// Solve a square linear system `Ax = rhs` using Gaussian elimination.
    ///
    /// On success, `rhs` is overwritten with the solution vector `x`.
    ///
    /// # Errors
    /// Returns an `Err(String)` if the system cannot be solved, e.g., if the matrix
    /// is singular.
    ///
    /// # Example
    /// ```
    /// # use cnvx_math::{DenseMatrix, Matrix};
    /// let mut a = DenseMatrix::new(2, 2);
    /// a.set(0, 0, 2.0);
    /// a.set(0, 1, 1.0);
    /// a.set(1, 0, 1.0);
    /// a.set(1, 1, 3.0);
    ///
    /// let mut rhs = vec![3.0, 7.0];
    /// a.gaussian_elimination(&mut rhs).unwrap();
    /// assert!((rhs[0] - 0.4).abs() < 1e-6);
    /// assert!((rhs[1] - 2.2).abs() < 1e-6);
    /// ```
    fn gaussian_elimination(&self, rhs: &mut [f64]) -> Result<(), String>
    where
        Self: Sized;
}
