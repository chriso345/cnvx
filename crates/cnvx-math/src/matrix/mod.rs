mod dense;
mod solvers;
mod sparse;

pub use dense::DenseMatrix;
pub use sparse::SparseMatrix;

/// A generic matrix trait for linear algebra operations.
///
/// This trait defines the interface for matrices used in the LP solver,
/// including creation, element access, arithmetic, and solving linear systems.
pub trait Matrix: Clone {
    /// Create a new matrix with the given number of rows and columns,
    /// initialized with zeros.
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

    /// Adds another matrix to this matrix.
    fn add(&self, other: &Self) -> Result<Self, String>
    where
        Self: Sized;

    /// Subtracts another matrix from this matrix.
    fn sub(&self, other: &Self) -> Result<Self, String>
    where
        Self: Sized;

    /// Multiplies this matrix by another matrix.
    fn mul(&self, other: &Self) -> Result<Self, String>
    where
        Self: Sized;

    /// Multiplies this matrix by a column vector.
    fn mul_vec(&self, rhs: &[f64]) -> Result<Vec<f64>, String>;

    /// Adds a scalar to every element in the matrix.
    fn add_scalar(&self, scalar: f64) -> Self
    where
        Self: Sized;

    /// Subtracts a scalar from every element in the matrix.
    fn sub_scalar(&self, scalar: f64) -> Self
    where
        Self: Sized;

    /// Multiplies every element in the matrix by a scalar.
    fn mul_scalar(&self, scalar: f64) -> Self
    where
        Self: Sized;

    /// Divides every element in the matrix by a scalar.
    fn div_scalar(&self, scalar: f64) -> Self
    where
        Self: Sized;

    /// Performs element-wise (Hadamard) multiplication with another matrix.
    fn mul_elementwise(&self, other: &Self) -> Result<Self, String>
    where
        Self: Sized;

    /// Extracts a specific row as a standard vector.
    fn get_row(&self, row: usize) -> Vec<f64>;

    /// Extracts a specific column as a standard vector.
    fn get_col(&self, col: usize) -> Vec<f64>;

    /// Overwrites a specific row with a new vector.
    fn set_row(&mut self, row: usize, vec: &[f64]) -> Result<(), String>;

    /// Overwrites a specific column with a new vector.
    fn set_col(&mut self, col: usize, vec: &[f64]) -> Result<(), String>;

    /// Swaps two rows in place.
    fn swap_rows(&mut self, row1: usize, row2: usize);

    /// Calculates the L_infinity norm (maximum absolute row sum).
    fn norm_inf(&self) -> f64;

    /// Returns the transpose of the matrix.
    fn transpose(&self) -> Self
    where
        Self: Sized;

    /// Creates a square identity matrix of the given size.
    fn identity(size: usize) -> Self
    where
        Self: Sized;

    /// Creates a square diagonal matrix from a vector of diagonal elements.
    fn from_diagonal(vec: &[f64]) -> Self
    where
        Self: Sized;

    /// Extracts the diagonal elements of the matrix as a vector.
    fn diagonal(&self) -> Vec<f64>;

    /// Solves Ax = b.
    /// Returns a dynamically allocated Vec<f64> to support both square
    /// and rectangular (least-squares) solutions.
    fn mldivide(&self, rhs: &[f64]) -> Result<Vec<f64>, String>;
}

/// Helper function to calculate the L2 (Euclidean) norm of a standard vector.
pub fn vector_norm_l2(vec: &[f64]) -> f64 {
    vec.iter().map(|&x| x * x).sum::<f64>().sqrt()
}
