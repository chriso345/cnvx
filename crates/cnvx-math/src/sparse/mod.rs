use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

mod direct;
mod iterative;
mod lanczos;
mod preconditioner;

pub use iterative::{IterativeOptions, IterativeResult};
pub use preconditioner::Preconditioner;

/// A compressed-sparse-row (CSR) matrix.
///
/// This is what the LP constraint matrix and large graph adjacency
/// structures use internally; [`crate::Matrix`] is for small, dense
/// structures (e.g. NLP Hessians) where density doesn't matter.
#[derive(Debug, Clone, PartialEq)]
pub struct SparseMatrix {
    rows: usize,
    cols: usize,

    row_ptr: Vec<usize>,
    col_idx: Vec<usize>,
    values: Vec<f64>,
}

impl SparseMatrix {
    /// Builds a CSR matrix from `(row, col, value)` triplets.
    ///
    /// Duplicate `(row, col)` entries are summed.
    ///
    /// # Panics
    /// Panics if any triplet's `row` or `col` is out of bounds.
    pub fn from_triplets(
        rows: usize,
        cols: usize,
        triplets: &[(usize, usize, f64)],
    ) -> Self {
        let mut by_row: Vec<Vec<(usize, f64)>> = vec![Vec::new(); rows];
        for &(r, c, v) in triplets {
            assert!(
                r < rows && c < cols,
                "SparseMatrix::from_triplets: index ({r}, {c}) out of bounds for a {rows}x{cols} matrix"
            );
            by_row[r].push((c, v));
        }

        let mut row_ptr = Vec::with_capacity(rows + 1);
        let mut col_idx = Vec::new();
        let mut values = Vec::new();
        row_ptr.push(0);

        for row_entries in by_row.iter_mut() {
            row_entries.sort_by_key(|&(c, _)| c);
            let mut i = 0;
            while i < row_entries.len() {
                let col = row_entries[i].0;
                let mut sum = 0.0;
                while i < row_entries.len() && row_entries[i].0 == col {
                    sum += row_entries[i].1;
                    i += 1;
                }
                col_idx.push(col);
                values.push(sum);
            }
            row_ptr.push(col_idx.len());
        }

        Self { rows, cols, row_ptr, col_idx, values }
    }

    /// An alias for [`SparseMatrix::from_triplets`].
    pub fn from_coo(rows: usize, cols: usize, triplets: &[(usize, usize, f64)]) -> Self {
        Self::from_triplets(rows, cols, triplets)
    }

    /// Builds a `SparseMatrix` from a dense [`Matrix`], keeping only
    /// entries with an absolute value greater than `tol`.
    pub fn from_dense(m: &Matrix, tol: f64) -> Self {
        let mut triplets = Vec::new();
        for i in 0..m.rows() {
            for j in 0..m.cols() {
                let v = m.get(i, j);
                if v.abs() > tol {
                    triplets.push((i, j, v));
                }
            }
        }
        Self::from_triplets(m.rows(), m.cols(), &triplets)
    }

    /// Number of rows.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Number of explicitly-stored (non-zero) entries.
    pub fn nnz(&self) -> usize {
        self.values.len()
    }

    /// Reads the element at `(row, col)`. Returns `0.0` for entries that
    /// aren't explicitly stored.
    ///
    /// # Panics
    /// Panics if `row` or `col` is out of bounds.
    pub fn get(&self, row: usize, col: usize) -> f64 {
        assert!(
            row < self.rows && col < self.cols,
            "SparseMatrix::get: index out of bounds"
        );
        let start = self.row_ptr[row];
        let end = self.row_ptr[row + 1];
        self.col_idx[start..end]
            .iter()
            .position(|&c| c == col)
            .map(|offset| self.values[start + offset])
            .unwrap_or(0.0)
    }

    /// Sparse matrix-vector multiplication.
    ///
    /// # Panics
    /// Panics if `v.len() != self.cols()`.
    pub fn mul_vec(&self, v: &Vector) -> Vector {
        assert_eq!(self.cols, v.len(), "SparseMatrix::mul_vec: dimension mismatch");
        let mut result = vec![0.0; self.rows];
        for (row, result_row) in result.iter_mut().enumerate() {
            let start = self.row_ptr[row];
            let end = self.row_ptr[row + 1];
            let mut sum = 0.0;
            for k in start..end {
                sum += self.values[k] * v[self.col_idx[k]];
            }
            *result_row = sum;
        }
        Vector::from(result)
    }

    /// Checked variant of [`SparseMatrix::mul_vec`] for call sites where the
    /// vector's length isn't known to match statically.
    pub fn try_mul_vec(&self, v: &Vector) -> Result<Vector, MathError> {
        if v.len() != self.cols {
            return Err(MathError::DimensionMismatch(format!(
                "matrix has {} columns, vector has {} elements",
                self.cols,
                v.len()
            )));
        }
        Ok(self.mul_vec(v))
    }

    /// Iterates over the `(column, value)` pairs explicitly stored in
    /// `row`, in increasing column order.
    ///
    /// # Panics
    /// Panics if `row >= self.rows()`.
    pub fn row_entries(&self, row: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        assert!(row < self.rows, "SparseMatrix::row_entries: row out of bounds");
        let start = self.row_ptr[row];
        let end = self.row_ptr[row + 1];
        self.col_idx[start..end]
            .iter()
            .copied()
            .zip(self.values[start..end].iter().copied())
    }

    /// Converts to a dense [`Matrix`], filling unstored entries with zero.
    pub fn to_dense(&self) -> Matrix {
        let mut m = Matrix::zeros(self.rows, self.cols);
        for row in 0..self.rows {
            for (col, val) in self.row_entries(row) {
                m.set(row, col, val);
            }
        }
        m
    }

    /// The matrix transpose, as a new `SparseMatrix`.
    pub fn transpose(&self) -> SparseMatrix {
        let mut triplets = Vec::with_capacity(self.nnz());
        for row in 0..self.rows {
            for (col, val) in self.row_entries(row) {
                triplets.push((col, row, val));
            }
        }
        SparseMatrix::from_triplets(self.cols, self.rows, &triplets)
    }

    /// Sparse-sparse matrix multiplication: `self * other`.
    ///
    /// # Errors
    /// Returns [`MathError::DimensionMismatch`] if `self.cols() !=
    /// other.rows()`.
    pub fn mul_sparse(&self, other: &SparseMatrix) -> Result<SparseMatrix, MathError> {
        if self.cols != other.rows {
            return Err(MathError::DimensionMismatch(format!(
                "cannot multiply a {}x{} matrix by a {}x{} matrix",
                self.rows, self.cols, other.rows, other.cols
            )));
        }
        let mut triplets = Vec::new();
        for i in 0..self.rows {
            // Accumulate row i of the product by scaling and summing the
            // relevant rows of `other`.
            let mut acc: std::collections::HashMap<usize, f64> =
                std::collections::HashMap::new();
            for (k, a_ik) in self.row_entries(i) {
                for (j, b_kj) in other.row_entries(k) {
                    *acc.entry(j).or_insert(0.0) += a_ik * b_kj;
                }
            }
            for (j, v) in acc {
                if v != 0.0 {
                    triplets.push((i, j, v));
                }
            }
        }
        Ok(SparseMatrix::from_triplets(self.rows, other.cols, &triplets))
    }

    /// Extracts the diagonal as a dense [`Vector`] (entries with no
    /// explicitly stored diagonal value are `0.0`).
    pub fn diagonal(&self) -> Vector {
        let n = self.rows.min(self.cols);
        Vector::from((0..n).map(|i| self.get(i, i)).collect::<Vec<_>>())
    }

    /// Whether `self` is symmetric to within `tol`.
    ///
    /// Non-square matrices are never symmetric.
    pub fn is_symmetric(&self, tol: f64) -> bool {
        if self.rows != self.cols {
            return false;
        }
        for i in 0..self.rows {
            for (j, v) in self.row_entries(i) {
                if (v - self.get(j, i)).abs() > tol {
                    return false;
                }
            }
        }
        true
    }

    /// Solves `A x = rhs` via the conjugate gradient method. Requires `A`
    /// to be symmetric positive-definite.
    pub fn solve_cg(
        &self,
        rhs: &Vector,
        options: &IterativeOptions,
    ) -> Result<IterativeResult, MathError> {
        iterative::conjugate_gradient(self, rhs, options)
    }

    /// Solves `A x = rhs` via restarted GMRES, for general (possibly
    /// non-symmetric) `A`.
    pub fn solve_gmres(
        &self,
        rhs: &Vector,
        options: &IterativeOptions,
    ) -> Result<IterativeResult, MathError> {
        iterative::gmres(self, rhs, options)
    }

    /// Solves `A x = rhs` directly via UMFPACK, for general (possibly
    /// non-symmetric) sparse `A`. Not available on `wasm32`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn solve_direct(&self, rhs: &Vector) -> Result<Vector, MathError> {
        direct::solve_direct(self, rhs)
    }

    /// The `k` smallest eigenpairs of `self` (must be symmetric), via the
    /// Lanczos algorithm.
    pub fn smallest_eigenpairs(
        &self,
        k: usize,
        options: &IterativeOptions,
    ) -> Result<crate::linalg::EigenDecomposition, MathError> {
        lanczos::smallest_eigenpairs(self, k, options)
    }

    /// The `k` largest eigenpairs of `self` (must be symmetric), via the
    /// Lanczos algorithm.
    pub fn largest_eigenpairs(
        &self,
        k: usize,
        options: &IterativeOptions,
    ) -> Result<crate::linalg::EigenDecomposition, MathError> {
        lanczos::largest_eigenpairs(self, k, options)
    }
}
