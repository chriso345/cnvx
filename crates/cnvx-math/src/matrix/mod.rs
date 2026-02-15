use std::ops::{Index, IndexMut};

mod dense;
mod sparse;

pub use dense::DenseMatrix;
pub use sparse::SparseMatrix;

/// A generic matrix trait for linear algebra operations.
///
/// This trait defines the interface for matrices used in the LP solver,
/// including creation, element access, and solving linear systems.
///
/// Implementors must provide row/column indexing, element access, and
///  a method for solving linear systems.
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

    /// Compute the matrix-vector product `Ax` where `A` is this matrix and `x` is the input vector.
    ///
    /// # Panics
    /// Panics if the length of `x` does not match the number of columns in the matrix.
    fn matvec(&self, x: &[f64]) -> Vec<f64>;

    /// Solve a square linear system `Ax = rhs`
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
    /// a.mldivide(&mut rhs).unwrap();
    /// assert!((rhs[0] - 0.4).abs() < 1e-6);
    /// assert!((rhs[1] - 2.2).abs() < 1e-6);
    /// ```
    fn mldivide(&self, rhs: &mut [f64]) -> Result<(), String>
    where
        Self: Sized;

    /// Create a matrix of the given size, initialized with zeros.
    ///
    /// # Example
    /// ```
    /// # use cnvx_math::{DenseMatrix, Matrix};
    /// let m = DenseMatrix::zeros(2, 3);
    /// assert_eq!(m.rows(), 2);
    /// assert_eq!(m.cols(), 3);
    /// assert_eq!(m.get(0, 0), 0.0);
    /// ```
    fn zeros(rows: usize, cols: usize) -> Self
    where
        Self: Sized;

    /// Create an identity matrix of the given size.
    ///
    /// # Example
    /// ```
    /// # use cnvx_math::{DenseMatrix, Matrix};
    /// let eye = DenseMatrix::eye(3);
    /// assert_eq!(eye.get(0, 0), 1.0);
    /// assert_eq!(eye.get(0, 1), 0.0);
    /// ```
    fn eye(size: usize) -> Self
    where
        Self: Sized,
    {
        let mut m = Self::new(size, size);
        for i in 0..size {
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
}

/// MatrixSolveMethod defines a trait for defining different methods to solve linear
/// systems for a given Matrix type M.
///
/// This systems are used to solve the $A * x = b$ systems that arise in the Simplex
/// method.
///
/// This allows us to implement multiple solving strategies, and select the appropriate
/// strategy at runtime based on the properties of the matrix.
///
/// Implementors of this trait must provide a constructor to take a reference the the
/// matrix and a solve method, that takes a mutable reference to the right-hand side
/// vector.
pub(crate) trait MatrixSolveMethod<'matrix, M: Matrix> {
    /// Create a new solver for the given matrix.
    fn new(matrix: &'matrix M) -> Self
    where
        Self: Sized;

    /// Solve the linear system `Ax = rhs` where `A` is the matrix associated with this
    /// solver.
    ///
    /// On success, `rhs` is overwritten with the solution vector `x`.
    fn solve(&self, rhs: &mut [f64]) -> Result<(), String>;
}
