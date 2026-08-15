//! [`Matrix`]: a dense, row-major matrix.

#[cfg(not(target_arch = "wasm32"))]
use cblas::{Layout, Transpose, dgemm, dgemv};

use crate::error::MathError;
use crate::vector::Vector;

mod solvers;

/// A dense, row-major matrix.
///
/// Large, mostly-zero structures should use [`crate::SparseMatrix`]
/// instead.
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    #[inline(always)]
    fn index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    /// Exposes the underlying row-major data slice. Crate-internal: used
    /// by the native LAPACK/BLAS solvers.
    pub(crate) fn data(&self) -> &[f64] {
        &self.data
    }

    /// Creates a `rows` x `cols` matrix of zeros.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self { rows, cols, data: vec![0.0; rows * cols] }
    }

    /// Creates an `n` x `n` identity matrix.
    pub fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n, n);
        for i in 0..n {
            m.set(i, i, 1.0);
        }
        m
    }

    /// Creates a square diagonal matrix from `diag`.
    pub fn from_diagonal(diag: &[f64]) -> Self {
        let n = diag.len();
        let mut m = Self::zeros(n, n);
        for (i, &v) in diag.iter().enumerate() {
            m.data[i * n + i] = v; // faster than set() for a fresh diagonal
        }
        m
    }

    /// Number of rows.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Reads the element at `(row, col)`.
    ///
    /// # Panics
    /// Panics if `row` or `col` is out of bounds.
    #[inline(always)]
    pub fn get(&self, row: usize, col: usize) -> f64 {
        assert!(row < self.rows && col < self.cols, "Matrix::get: index out of bounds");
        self.data[self.index(row, col)]
    }

    /// Writes `value` to `(row, col)`.
    ///
    /// # Panics
    /// Panics if `row` or `col` is out of bounds.
    #[inline(always)]
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        assert!(row < self.rows && col < self.cols, "Matrix::set: index out of bounds");
        let idx = self.index(row, col);
        self.data[idx] = value;
    }

    /// Extracts the diagonal elements as a [`Vector`].
    pub fn diagonal(&self) -> Vector {
        let n = self.rows.min(self.cols);
        (0..n).map(|i| self.get(i, i)).collect()
    }

    /// Extracts row `row` as a [`Vector`].
    ///
    /// # Panics
    /// Panics if `row` is out of bounds.
    pub fn get_row(&self, row: usize) -> Vector {
        assert!(row < self.rows, "Matrix::get_row: index out of bounds");
        let start = row * self.cols;
        Vector::from_slice(&self.data[start..start + self.cols])
    }

    /// Extracts column `col` as a [`Vector`].
    ///
    /// # Panics
    /// Panics if `col` is out of bounds.
    pub fn get_col(&self, col: usize) -> Vector {
        assert!(col < self.cols, "Matrix::get_col: index out of bounds");
        (0..self.rows).map(|i| self.data[i * self.cols + col]).collect()
    }

    /// Overwrites row `row` with `values`.
    pub fn set_row(&mut self, row: usize, values: &[f64]) -> Result<(), MathError> {
        if row >= self.rows {
            return Err(MathError::DimensionMismatch(format!(
                "row {row} out of bounds (matrix has {} rows)",
                self.rows
            )));
        }
        if values.len() != self.cols {
            return Err(MathError::DimensionMismatch(format!(
                "expected {} values, got {}",
                self.cols,
                values.len()
            )));
        }
        let start = row * self.cols;
        self.data[start..start + self.cols].copy_from_slice(values);
        Ok(())
    }

    /// Overwrites column `col` with `values`.
    pub fn set_col(&mut self, col: usize, values: &[f64]) -> Result<(), MathError> {
        if col >= self.cols {
            return Err(MathError::DimensionMismatch(format!(
                "column {col} out of bounds (matrix has {} cols)",
                self.cols
            )));
        }
        if values.len() != self.rows {
            return Err(MathError::DimensionMismatch(format!(
                "expected {} values, got {}",
                self.rows,
                values.len()
            )));
        }
        for (i, &v) in values.iter().enumerate() {
            self.data[i * self.cols + col] = v;
        }
        Ok(())
    }

    /// Swaps two rows in place.
    ///
    /// # Panics
    /// Panics if `row1` or `row2` is out of bounds.
    pub fn swap_rows(&mut self, row1: usize, row2: usize) {
        assert!(
            row1 < self.rows && row2 < self.rows,
            "Matrix::swap_rows: index out of bounds"
        );
        if row1 == row2 {
            return;
        }
        let (r1, r2) = if row1 < row2 { (row1, row2) } else { (row2, row1) };
        let (first, second) = self.data.split_at_mut(r2 * self.cols);
        let row1_slice = &mut first[r1 * self.cols..(r1 + 1) * self.cols];
        let row2_slice = &mut second[0..self.cols];
        row1_slice.swap_with_slice(row2_slice);
    }

    /// The L-infinity norm: the maximum absolute row sum.
    pub fn norm_inf(&self) -> f64 {
        let mut max_sum = 0.0_f64;
        for i in 0..self.rows {
            let start = i * self.cols;
            let row_sum: f64 =
                self.data[start..start + self.cols].iter().map(|x| x.abs()).sum();
            max_sum = max_sum.max(row_sum);
        }
        max_sum
    }

    /// Returns the transpose.
    pub fn transpose(&self) -> Matrix {
        let mut result = Matrix::zeros(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.set(j, i, self.get(i, j));
            }
        }
        result
    }

    /// Elementwise addition.
    pub fn add(&self, other: &Matrix) -> Result<Matrix, MathError> {
        self.check_same_shape(other, "add")?;
        let data = self.data.iter().zip(&other.data).map(|(a, b)| a + b).collect();
        Ok(Matrix { rows: self.rows, cols: self.cols, data })
    }

    /// Elementwise subtraction.
    pub fn sub(&self, other: &Matrix) -> Result<Matrix, MathError> {
        self.check_same_shape(other, "sub")?;
        let data = self.data.iter().zip(&other.data).map(|(a, b)| a - b).collect();
        Ok(Matrix { rows: self.rows, cols: self.cols, data })
    }

    /// Elementwise (Hadamard) multiplication.
    pub fn mul_elementwise(&self, other: &Matrix) -> Result<Matrix, MathError> {
        self.check_same_shape(other, "mul_elementwise")?;
        let data = self.data.iter().zip(&other.data).map(|(a, b)| a * b).collect();
        Ok(Matrix { rows: self.rows, cols: self.cols, data })
    }

    /// Adds `scalar` to every element.
    pub fn add_scalar(&self, scalar: f64) -> Matrix {
        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|x| x + scalar).collect(),
        }
    }

    /// Subtracts `scalar` from every element.
    pub fn sub_scalar(&self, scalar: f64) -> Matrix {
        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|x| x - scalar).collect(),
        }
    }

    /// Multiplies every element by `scalar`.
    pub fn mul_scalar(&self, scalar: f64) -> Matrix {
        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|x| x * scalar).collect(),
        }
    }

    /// Divides every element by `scalar`.
    pub fn div_scalar(&self, scalar: f64) -> Matrix {
        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|x| x / scalar).collect(),
        }
    }

    /// Matrix-matrix multiplication.
    pub fn mul(&self, other: &Matrix) -> Result<Matrix, MathError> {
        if self.cols != other.rows {
            return Err(MathError::DimensionMismatch(format!(
                "cannot multiply a {}x{} matrix by a {}x{} matrix",
                self.rows, self.cols, other.rows, other.cols
            )));
        }

        let mut result = Matrix::zeros(self.rows, other.cols);

        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            let m = self.rows as i32;
            let k = self.cols as i32;
            let n = other.cols as i32;
            dgemm(
                Layout::RowMajor,
                Transpose::None,
                Transpose::None,
                m,
                n,
                k,
                1.0,
                &self.data,
                k, // lda
                &other.data,
                n, // ldb
                0.0,
                &mut result.data,
                n, // ldc
            );
        }

        #[cfg(target_arch = "wasm32")]
        {
            for i in 0..self.rows {
                for j in 0..other.cols {
                    let mut sum = 0.0;
                    for k in 0..self.cols {
                        sum += self.get(i, k) * other.get(k, j);
                    }
                    result.set(i, j, sum);
                }
            }
        }

        Ok(result)
    }

    /// Matrix-vector multiplication.
    ///
    /// # Panics
    /// Panics if `v.len() != self.cols()`.
    pub fn mul_vec(&self, v: &Vector) -> Vector {
        assert_eq!(self.cols, v.len(), "Matrix::mul_vec: dimension mismatch");

        #[cfg(not(target_arch = "wasm32"))]
        {
            let m = self.rows as i32;
            let n = self.cols as i32;
            let mut result = vec![0.0; self.rows];
            unsafe {
                dgemv(
                    Layout::RowMajor,
                    Transpose::None,
                    m,
                    n,
                    1.0,
                    &self.data,
                    n, // lda
                    v,
                    1, // incx
                    0.0,
                    &mut result,
                    1, // incy
                );
            }
            Vector::from(result)
        }

        #[cfg(target_arch = "wasm32")]
        {
            let mut result = vec![0.0; self.rows];
            for i in 0..self.rows {
                let mut sum = 0.0;
                for j in 0..self.cols {
                    sum += self.get(i, j) * v[j];
                }
                result[i] = sum;
            }
            Vector::from(result)
        }
    }

    /// Solves `A x = rhs`.
    ///
    /// Dispatches on the structure of `A` (MATLAB-style `mldivide`):
    /// 1. If `A` is rectangular, solves the least-squares problem via QR.
    /// 2. If `A` is triangular (or diagonal), uses forward/back substitution
    ///    directly.
    /// 3. If `A` is symmetric, tries a Cholesky factorization (i.e. `A` is
    ///    symmetric positive-definite).
    /// 4. Otherwise, falls back to LU decomposition with partial pivoting.
    pub fn solve(&self, rhs: &Vector) -> Result<Vector, MathError> {
        solvers::solve_dense(self, rhs)
    }

    fn check_same_shape(&self, other: &Matrix, op: &str) -> Result<(), MathError> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err(MathError::DimensionMismatch(format!(
                "{op}: {}x{} vs {}x{}",
                self.rows, self.cols, other.rows, other.cols
            )));
        }
        Ok(())
    }
}
