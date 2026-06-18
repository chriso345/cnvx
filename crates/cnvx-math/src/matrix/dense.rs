use crate::matrix::Matrix;

/// A dense matrix stored in row-major order using a flat 1D vector.
#[derive(Debug, Clone, PartialEq)]
pub struct DenseMatrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl DenseMatrix {
    /// Helper to calculate the 1D index from 2D coordinates.
    #[inline(always)]
    fn index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }
}

impl Matrix for DenseMatrix {
    fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols, data: vec![0.0; rows * cols] }
    }

    #[inline(always)]
    fn rows(&self) -> usize {
        self.rows
    }

    #[inline(always)]
    fn cols(&self) -> usize {
        self.cols
    }

    #[inline(always)]
    fn get(&self, row: usize, col: usize) -> f64 {
        assert!(row < self.rows && col < self.cols, "Index out of bounds");
        self.data[self.index(row, col)]
    }

    #[inline(always)]
    fn set(&mut self, row: usize, col: usize, value: f64) {
        assert!(row < self.rows && col < self.cols, "Index out of bounds");
        let idx = self.index(row, col);
        self.data[idx] = value;
    }

    fn add(&self, other: &Self) -> Result<Self, String> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err("Matrix dimensions must match for addition".to_string());
        }
        let data = self.data.iter().zip(other.data.iter()).map(|(a, b)| a + b).collect();
        Ok(Self { rows: self.rows, cols: self.cols, data })
    }

    fn sub(&self, other: &Self) -> Result<Self, String> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err("Matrix dimensions must match for subtraction".to_string());
        }
        let data = self.data.iter().zip(other.data.iter()).map(|(a, b)| a - b).collect();
        Ok(Self { rows: self.rows, cols: self.cols, data })
    }

    fn mul(&self, other: &Self) -> Result<Self, String> {
        if self.cols != other.rows {
            return Err("Incompatible dimensions for matrix multiplication".to_string());
        }

        let mut result = Self::new(self.rows, other.cols);

        // IKJ loop ordering for cache-friendly memory access
        for i in 0..self.rows {
            for k in 0..self.cols {
                let a_ik = self.get(i, k);
                for j in 0..other.cols {
                    let b_kj = other.get(k, j);
                    let current = result.get(i, j);
                    result.set(i, j, current + a_ik * b_kj);
                }
            }
        }
        Ok(result)
    }

    fn mul_vec(&self, rhs: &[f64]) -> Result<Vec<f64>, String> {
        if self.cols != rhs.len() {
            return Err("Matrix columns must match vector length".to_string());
        }
        let mut result = vec![0.0; self.rows];
        for i in 0..self.rows {
            let mut sum = 0.0;
            for j in 0..self.cols {
                sum += self.get(i, j) * rhs[j];
            }
            result[i] = sum;
        }
        Ok(result)
    }

    fn add_scalar(&self, scalar: f64) -> Self {
        let data = self.data.iter().map(|&x| x + scalar).collect();
        Self { rows: self.rows, cols: self.cols, data }
    }

    fn sub_scalar(&self, scalar: f64) -> Self {
        let data = self.data.iter().map(|&x| x - scalar).collect();
        Self { rows: self.rows, cols: self.cols, data }
    }

    fn mul_scalar(&self, scalar: f64) -> Self {
        let data = self.data.iter().map(|&x| x * scalar).collect();
        Self { rows: self.rows, cols: self.cols, data }
    }

    fn div_scalar(&self, scalar: f64) -> Self {
        let data = self.data.iter().map(|&x| x / scalar).collect();
        Self { rows: self.rows, cols: self.cols, data }
    }

    fn mul_elementwise(&self, other: &Self) -> Result<Self, String> {
        if self.rows != other.rows || self.cols != other.cols {
            return Err("Matrix dimensions must match for element-wise multiplication"
                .to_string());
        }
        let data = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| a * b)
            .collect();
        Ok(Self { rows: self.rows, cols: self.cols, data })
    }

    fn get_row(&self, row: usize) -> Vec<f64> {
        assert!(row < self.rows, "Row index out of bounds");
        let start = row * self.cols;
        self.data[start..start + self.cols].to_vec()
    }

    fn get_col(&self, col: usize) -> Vec<f64> {
        assert!(col < self.cols, "Column index out of bounds");
        (0..self.rows).map(|i| self.data[i * self.cols + col]).collect()
    }

    fn set_row(&mut self, row: usize, vec: &[f64]) -> Result<(), String> {
        if row >= self.rows {
            return Err("Row index out of bounds".to_string());
        }
        if vec.len() != self.cols {
            return Err("Vector length must match matrix columns".to_string());
        }
        let start = row * self.cols;
        self.data[start..start + self.cols].copy_from_slice(vec);
        Ok(())
    }

    fn set_col(&mut self, col: usize, vec: &[f64]) -> Result<(), String> {
        if col >= self.cols {
            return Err("Column index out of bounds".to_string());
        }
        if vec.len() != self.rows {
            return Err("Vector length must match matrix rows".to_string());
        }
        for i in 0..self.rows {
            self.data[i * self.cols + col] = vec[i];
        }
        Ok(())
    }

    fn swap_rows(&mut self, row1: usize, row2: usize) {
        assert!(row1 < self.rows && row2 < self.rows, "Row index out of bounds");
        if row1 == row2 {
            return;
        }
        // Ensure row1 is the smaller index to safely split the slice
        let (r1, r2) = if row1 < row2 { (row1, row2) } else { (row2, row1) };
        let (first, second) = self.data.split_at_mut(r2 * self.cols);

        let row1_slice = &mut first[r1 * self.cols..(r1 + 1) * self.cols];
        let row2_slice = &mut second[0..self.cols];
        row1_slice.swap_with_slice(row2_slice);
    }

    fn norm_inf(&self) -> f64 {
        let mut max_sum = 0.0_f64;
        for i in 0..self.rows {
            let start = i * self.cols;
            let row_sum: f64 =
                self.data[start..start + self.cols].iter().map(|&x| x.abs()).sum();
            if row_sum > max_sum {
                max_sum = row_sum;
            }
        }
        max_sum
    }

    fn transpose(&self) -> Self {
        let mut result = Self::new(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.set(j, i, self.get(i, j));
            }
        }
        result
    }

    fn identity(size: usize) -> Self {
        let mut m = Self::new(size, size);
        for i in 0..size {
            m.set(i, i, 1.0);
        }
        m
    }

    fn from_diagonal(vec: &[f64]) -> Self {
        let size = vec.len();
        let mut m = Self::new(size, size);
        for i in 0..size {
            m.set(i, i, vec[i]);
        }
        m
    }

    fn diagonal(&self) -> Vec<f64> {
        let size = self.rows.min(self.cols);
        (0..size).map(|i| self.get(i, i)).collect()
    }

    fn mldivide(&self, rhs: &mut [f64]) -> Result<(), String> {
        if self.rows != self.cols {
            return Err("Matrix must be square to solve linear systems".to_string());
        }
        if self.rows != rhs.len() {
            return Err("RHS vector length must match matrix dimensions".to_string());
        }

        let n = self.rows;
        let mut lu = self.data.clone();
        let mut p: Vec<usize> = (0..n).collect();

        // LU Decomposition with Partial Pivoting
        for i in 0..n {
            let mut max_a = 0.0;
            let mut imax = i;
            for k in i..n {
                let abs_a = lu[k * n + i].abs();
                if abs_a > max_a {
                    max_a = abs_a;
                    imax = k;
                }
            }

            if max_a < 1e-12 {
                return Err("Matrix is singular or nearly singular".to_string());
            }

            if imax != i {
                for k in 0..n {
                    lu.swap(i * n + k, imax * n + k);
                }
                p.swap(i, imax);
            }

            for j in (i + 1)..n {
                lu[j * n + i] /= lu[i * n + i];
                for k in (i + 1)..n {
                    let factor = lu[j * n + i] * lu[i * n + k];
                    lu[j * n + k] -= factor;
                }
            }
        }

        // Forward substitution
        let mut x = vec![0.0; n];
        for i in 0..n {
            x[i] = rhs[p[i]];
            for k in 0..i {
                x[i] -= lu[i * n + k] * x[k];
            }
        }

        // Backward substitution
        for i in (0..n).rev() {
            for k in (i + 1)..n {
                x[i] -= lu[i * n + k] * x[k];
            }
            x[i] /= lu[i * n + i];
        }

        rhs.copy_from_slice(&x);
        Ok(())
    }
}
