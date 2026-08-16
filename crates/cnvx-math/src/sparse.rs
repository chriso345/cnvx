//! [`SparseMatrix`]: a compressed-sparse-row (CSR) matrix.

use crate::error::MathError;
use crate::vector::Vector;

/// A compressed-sparse-row (CSR) matrix.
///
/// This is what the LP constraint matrix and large graph adjacency
/// structures use internally; [`crate::Matrix`] is for small, dense
/// structures (e.g. NLP Hessians) where density doesn't matter.
#[derive(Debug, Clone, PartialEq)]
pub struct SparseMatrix {
    rows: usize,
    cols: usize,
    // CSR layout: row `r`'s entries live in `col_idx[row_ptr[r]..row_ptr[r + 1]]`
    // / `values[row_ptr[r]..row_ptr[r + 1]]`, with `col_idx` sorted per row.
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
}
