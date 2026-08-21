//! [`Matrix`]: a dense, row-major matrix.

#[cfg(not(target_arch = "wasm32"))]
use cblas::{Layout, Transpose, dgemm, dgemv};

use crate::error::MathError;
use crate::linalg::{
    CholeskyDecomposition, EigenDecomposition, LuDecomposition, QrDecomposition,
    SvdDecomposition,
};
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
    crate::elementwise::elementwise_fn_set!();

    #[inline(always)]
    fn index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    /// Exposes the underlying row-major data slice. Crate-internal: used
    /// by the native LAPACK/BLAS solvers.
    pub(crate) fn data(&self) -> &[f64] {
        &self.data
    }

    /// Creates a matrix directly from row-major data. Crate-internal:
    /// used to build decomposition results from LAPACK output buffers
    /// (which are already row-major, matching this crate's own layout).
    ///
    /// # Panics
    /// Panics if `data.len() != rows * cols`.
    pub(crate) fn from_row_major(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        assert_eq!(
            data.len(),
            rows * cols,
            "Matrix::from_row_major: data length mismatch"
        );
        Self { rows, cols, data }
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

    /// The trace: the sum of the diagonal elements.
    ///
    /// # Panics
    /// Panics if the matrix isn't square.
    pub fn trace(&self) -> f64 {
        assert_eq!(self.rows, self.cols, "Matrix::trace requires a square matrix");
        (0..self.rows).map(|i| self.get(i, i)).sum()
    }

    /// The Frobenius norm: `sqrt(sum(a_ij^2))`.
    pub fn frobenius_norm(&self) -> f64 {
        self.data.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    /// Whether `self` is symmetric to within `tol`: `|a[i][j] - a[j][i]| <=
    /// tol` for every `i`, `j`.
    ///
    /// Non-square matrices are never symmetric.
    pub fn is_symmetric(&self, tol: f64) -> bool {
        if self.rows != self.cols {
            return false;
        }
        for i in 0..self.rows {
            for j in (i + 1)..self.cols {
                if (self.get(i, j) - self.get(j, i)).abs() > tol {
                    return false;
                }
            }
        }
        true
    }

    /// Scales every element by `s`. An alias for [`Matrix::mul_scalar`]
    /// matching the design document's naming.
    pub fn scale(&self, s: f64) -> Matrix {
        self.mul_scalar(s)
    }

    /// Extracts row `r` as a [`Vector`]. An alias for [`Matrix::get_row`].
    pub fn row(&self, r: usize) -> Vector {
        self.get_row(r)
    }

    /// Extracts column `c` as a [`Vector`]. An alias for [`Matrix::get_col`].
    pub fn col(&self, c: usize) -> Vector {
        self.get_col(c)
    }

    /// Extracts the diagonal as a [`Vector`]. An alias for
    /// [`Matrix::diagonal`].
    pub fn diag(&self) -> Vector {
        self.diagonal()
    }

    /// Applies `f` to every element, producing a new matrix of the same
    /// shape.
    pub fn map(&self, f: impl Fn(f64) -> f64) -> Matrix {
        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|&x| f(x)).collect(),
        }
    }

    /// LU decomposition with partial pivoting: `P A = L U`.
    ///
    /// This is the same factorization [`Matrix::solve`] already uses
    /// internally for general square systems, exposed directly so callers
    /// solving against the same matrix with multiple right-hand sides can
    /// factorize once and reuse `l`/`u`/`p`.
    ///
    /// # Errors
    /// Returns [`MathError::Singular`] if `A` is numerically singular, and
    /// [`MathError::DimensionMismatch`] if `A` isn't square.
    pub fn lu(&self) -> Result<LuDecomposition, MathError> {
        crate::linalg::lu(self)
    }

    /// QR decomposition via Householder reflections: `A = Q R`.
    ///
    /// Requires `rows >= cols`. This is the numerically-preferred building
    /// block for least-squares regression (see [`Matrix::least_squares`]).
    ///
    /// # Errors
    /// Returns [`MathError::Unsupported`] if `rows < cols`, and
    /// [`MathError::RankDeficient`] if `A` doesn't have full column rank.
    pub fn qr(&self) -> Result<QrDecomposition, MathError> {
        crate::linalg::qr(self)
    }

    /// Cholesky decomposition: `A = L L^T`, valid only for symmetric
    /// positive-definite `A`.
    ///
    /// Roughly half the cost of a general LU factorization for the
    /// symmetric positive-definite systems it applies to (covariance
    /// matrices, kernel/Gram matrices).
    ///
    /// # Errors
    /// Returns [`MathError::NotPositiveDefinite`] if `A` isn't symmetric
    /// positive-definite, and [`MathError::DimensionMismatch`] if `A`
    /// isn't square.
    pub fn cholesky(&self) -> Result<CholeskyDecomposition, MathError> {
        crate::linalg::cholesky(self)
    }

    /// Eigendecomposition for symmetric matrices: real eigenvalues and an
    /// orthonormal eigenvector basis, sorted by descending eigenvalue.
    ///
    /// # Errors
    /// Returns [`MathError::DimensionMismatch`] if `A` isn't square.
    pub fn eigen_symmetric(&self) -> Result<EigenDecomposition, MathError> {
        crate::linalg::eigen_symmetric(self)
    }

    /// Eigendecomposition for general (possibly non-symmetric) square
    /// matrices, which may have complex-conjugate eigenvalue pairs. See
    /// [`EigenDecomposition`]'s documentation for how complex eigenvalues
    /// are represented.
    ///
    /// # Errors
    /// Returns [`MathError::DimensionMismatch`] if `A` isn't square, and
    /// [`MathError::Unsupported`] if the underlying iteration fails to
    /// converge.
    pub fn eigen_general(&self) -> Result<EigenDecomposition, MathError> {
        crate::linalg::eigen_general(self)
    }

    /// Singular value decomposition: `A = U * diag(s) * V^T`, with
    /// singular values in `s` sorted in descending order.
    ///
    /// [`Matrix::condition_number`], [`Matrix::rank`], and
    /// [`Matrix::pseudo_inverse`] are all derived from this one
    /// decomposition rather than three separate implementations.
    pub fn svd(&self) -> Result<SvdDecomposition, MathError> {
        crate::linalg::svd(self)
    }

    /// The determinant, via LU decomposition:
    /// `det(A) = (-1)^(number of row swaps) * product(diag(U))`.
    ///
    /// Returns `0.0` (rather than an error) for numerically singular `A`.
    ///
    /// # Errors
    /// Returns [`MathError::DimensionMismatch`] if `A` isn't square.
    pub fn determinant(&self) -> Result<f64, MathError> {
        let lu = match self.lu() {
            Ok(lu) => lu,
            Err(MathError::Singular) => return Ok(0.0),
            Err(e) => return Err(e),
        };
        let n = self.rows;
        let mut swaps = 0usize;
        let mut visited = vec![false; n];
        for i in 0..n {
            if visited[i] || lu.p[i] == i {
                continue;
            }
            let mut j = i;
            let mut cycle_len = 0;
            while !visited[j] {
                visited[j] = true;
                j = lu.p[j];
                cycle_len += 1;
            }
            swaps += cycle_len - 1;
        }
        let sign = if swaps.is_multiple_of(2) { 1.0 } else { -1.0 };
        let product: f64 = (0..n).map(|i| lu.u.get(i, i)).product();
        Ok(sign * product)
    }

    /// The numerical rank: the number of singular values greater than
    /// `tol`, via SVD.
    pub fn rank(&self, tol: f64) -> Result<usize, MathError> {
        let svd = self.svd()?;
        Ok(svd.s.iter().filter(|&&s| s > tol).count())
    }

    /// The 2-norm condition number: the ratio of the largest to smallest
    /// singular value, via SVD. A large condition number means `A` is
    /// close to singular / `Matrix::solve` results will be numerically
    /// sensitive to input perturbations.
    ///
    /// Returns `f64::INFINITY` if the smallest singular value is exactly
    /// zero (i.e. `A` is exactly singular).
    pub fn condition_number(&self) -> Result<f64, MathError> {
        let svd = self.svd()?;
        let max_s = svd.s.iter().copied().fold(0.0_f64, f64::max);
        let min_s = svd.s.iter().copied().fold(f64::INFINITY, f64::min);
        if min_s == 0.0 {
            return Ok(f64::INFINITY);
        }
        Ok(max_s / min_s)
    }

    /// The Moore-Penrose pseudo-inverse, via SVD: `A^+ = V * diag(1/s) * U^T`,
    /// with singular values below `1e-12` treated as zero (i.e. their
    /// corresponding term is dropped rather than blowing up numerically).
    pub fn pseudo_inverse(&self) -> Result<Matrix, MathError> {
        let svd = self.svd()?;
        let tol = 1e-12;
        let k = svd.s.len();
        let mut s_inv = Matrix::zeros(k, k);
        for i in 0..k {
            if svd.s[i] > tol {
                s_inv.set(i, i, 1.0 / svd.s[i]);
            }
        }
        // A^+ = V S^+ U^T, truncated to the k = min(rows, cols) reduced
        // singular basis.
        let v = svd.vt.transpose();
        let v_ref = &v;
        let v_k = Matrix::from_row_major(
            v.rows(),
            k,
            (0..v.rows())
                .flat_map(|i| (0..k).map(move |j| v_ref.get(i, j)))
                .collect(),
        );
        let u = &svd.u;
        let u_k = Matrix::from_row_major(
            u.rows(),
            k,
            (0..u.rows()).flat_map(|i| (0..k).map(move |j| u.get(i, j))).collect(),
        );
        v_k.mul(&s_inv)?.mul(&u_k.transpose())
    }

    /// Solves the least-squares problem `min ||A x - rhs||` via QR
    /// decomposition.
    ///
    /// Prefer this over [`Matrix::solve`]'s LU-based normal-equations-free
    /// path for overdetermined regression-style systems: QR is
    /// numerically better-conditioned than solving the normal equations
    /// `(A^T A) x = A^T rhs` directly (squaring `A`'s condition number).
    ///
    /// # Errors
    /// Returns [`MathError::Unsupported`] if `A` has fewer rows than
    /// columns, and [`MathError::RankDeficient`] if `A` doesn't have full
    /// column rank.
    pub fn least_squares(&self, rhs: &Vector) -> Result<Vector, MathError> {
        if rhs.len() != self.rows {
            return Err(MathError::DimensionMismatch(format!(
                "right-hand side has {} entries, matrix has {} rows",
                rhs.len(),
                self.rows
            )));
        }
        let qr = self.qr()?;
        // Solve R x = Q^T rhs via back substitution (R is upper triangular).
        let qtb = qr.q.transpose().mul_vec(rhs);
        let n = qr.r.cols();
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let diag = qr.r.get(i, i);
            if diag.abs() < 1e-12 {
                return Err(MathError::RankDeficient);
            }
            let mut sum = qtb[i];
            for (k, &xk) in x.iter().enumerate().skip(i + 1).take(n - i - 1) {
                sum -= qr.r.get(i, k) * xk;
            }
            x[i] = sum / diag;
        }
        Ok(Vector::from(x))
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
