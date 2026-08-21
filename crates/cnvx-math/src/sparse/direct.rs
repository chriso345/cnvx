use crate::error::MathError;
use crate::sparse::SparseMatrix;
use crate::vector::Vector;

const UMFPACK_CONTROL: usize = 20;
const UMFPACK_INFO: usize = 90;
const UMFPACK_A: i32 = 0; // Solve Ax = b (as opposed to A^T x = b, etc).

#[allow(non_snake_case)]
unsafe extern "C" {
    fn umfpack_di_symbolic(
        n_row: i32,
        n_col: i32,
        Ap: *const i32,
        Ai: *const i32,
        Ax: *const f64,
        Symbolic: *mut *mut std::ffi::c_void,
        Control: *const f64,
        Info: *mut f64,
    ) -> i32;

    fn umfpack_di_numeric(
        Ap: *const i32,
        Ai: *const i32,
        Ax: *const f64,
        Symbolic: *mut std::ffi::c_void,
        Numeric: *mut *mut std::ffi::c_void,
        Control: *const f64,
        Info: *mut f64,
    ) -> i32;

    fn umfpack_di_solve(
        sys: i32,
        Ap: *const i32,
        Ai: *const i32,
        Ax: *const f64,
        X: *mut f64,
        B: *const f64,
        Numeric: *mut std::ffi::c_void,
        Control: *const f64,
        Info: *mut f64,
    ) -> i32;

    fn umfpack_di_free_symbolic(Symbolic: *mut *mut std::ffi::c_void);
    fn umfpack_di_free_numeric(Numeric: *mut *mut std::ffi::c_void);
}

/// Converts this crate's CSR storage into the compressed-sparse-*column*
/// (CSC) form UMFPACK's `di` routines expect: `(Ap, Ai, Ax)`, where
/// column `c`'s entries are `Ai[Ap[c]..Ap[c+1]]` / `Ax[Ap[c]..Ap[c+1]]`.
///
/// CSR and CSC are transposes of each other in representation, so this
/// is exactly a transpose-and-repack.
fn csr_to_csc(a: &SparseMatrix) -> (Vec<i32>, Vec<i32>, Vec<f64>) {
    let n = a.rows();
    let ncols = a.cols();
    let nnz = a.nnz();

    let mut col_counts = vec![0usize; ncols + 1];
    for row in 0..n {
        for (col, _) in a.row_entries(row) {
            col_counts[col + 1] += 1;
        }
    }
    for c in 0..ncols {
        col_counts[c + 1] += col_counts[c];
    }
    let ap: Vec<i32> = col_counts.iter().map(|&x| x as i32).collect();

    let mut ai = vec![0i32; nnz];
    let mut ax = vec![0.0f64; nnz];
    let mut cursor = col_counts.clone();
    for row in 0..n {
        for (col, val) in a.row_entries(row) {
            let pos = cursor[col];
            ai[pos] = row as i32;
            ax[pos] = val;
            cursor[col] += 1;
        }
    }
    (ap, ai, ax)
}

/// Solves `A x = rhs` for a general (possibly non-symmetric) sparse `A`
/// via UMFPACK.
///
/// # Errors
/// Returns [`MathError::DimensionMismatch`] if `A` isn't square or
/// `rhs.len() != A.rows()`, and [`MathError::Singular`] if UMFPACK
/// reports the matrix is numerically or structurally singular.
pub fn solve_direct(a: &SparseMatrix, rhs: &Vector) -> Result<Vector, MathError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(MathError::DimensionMismatch(
            "solve_direct requires a square matrix".into(),
        ));
    }
    if rhs.len() != n {
        return Err(MathError::DimensionMismatch(format!(
            "matrix has {n} rows, right-hand side has {} entries",
            rhs.len()
        )));
    }

    let (ap, ai, ax) = csr_to_csc(a);
    let control = [0.0; UMFPACK_CONTROL];
    let mut info = vec![0.0; UMFPACK_INFO];

    unsafe {
        let mut symbolic: *mut std::ffi::c_void = std::ptr::null_mut();
        let status = umfpack_di_symbolic(
            n as i32,
            n as i32,
            ap.as_ptr(),
            ai.as_ptr(),
            ax.as_ptr(),
            &mut symbolic,
            control.as_ptr(),
            info.as_mut_ptr(),
        );
        if status != 0 {
            if !symbolic.is_null() {
                umfpack_di_free_symbolic(&mut symbolic);
            }
            return Err(MathError::Singular);
        }

        let mut numeric: *mut std::ffi::c_void = std::ptr::null_mut();
        let status = umfpack_di_numeric(
            ap.as_ptr(),
            ai.as_ptr(),
            ax.as_ptr(),
            symbolic,
            &mut numeric,
            control.as_ptr(),
            info.as_mut_ptr(),
        );
        umfpack_di_free_symbolic(&mut symbolic);
        if status != 0 {
            if !numeric.is_null() {
                umfpack_di_free_numeric(&mut numeric);
            }
            return Err(MathError::Singular);
        }

        let mut x = vec![0.0; n];
        let b: Vec<f64> = rhs.iter().copied().collect();
        let status = umfpack_di_solve(
            UMFPACK_A,
            ap.as_ptr(),
            ai.as_ptr(),
            ax.as_ptr(),
            x.as_mut_ptr(),
            b.as_ptr(),
            numeric,
            control.as_ptr(),
            info.as_mut_ptr(),
        );
        umfpack_di_free_numeric(&mut numeric);
        if status != 0 {
            return Err(MathError::Singular);
        }

        Ok(Vector::from(x))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csr_to_csc_round_trips_via_dense() {
        let a = SparseMatrix::from_triplets(
            3,
            3,
            &[(0, 0, 4.0), (0, 2, 1.0), (1, 1, 3.0), (2, 0, 2.0), (2, 2, 5.0)],
        );
        let (ap, ai, ax) = csr_to_csc(&a);
        // Reconstruct a dense matrix from the CSC arrays and compare
        // against SparseMatrix::to_dense.
        let mut dense = [0.0; 9];
        for col in 0..3 {
            for k in ap[col] as usize..ap[col + 1] as usize {
                let row = ai[k] as usize;
                dense[row * 3 + col] = ax[k];
            }
        }
        let expected = a.to_dense();
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(dense[r * 3 + c], expected.get(r, c));
            }
        }
    }

    #[test]
    fn solve_direct_matches_known_solution() {
        let a = SparseMatrix::from_triplets(
            3,
            3,
            &[
                (0, 0, 2.0),
                (0, 1, 1.0),
                (1, 0, 1.0),
                (1, 1, 3.0),
                (1, 2, 1.0),
                (2, 1, 1.0),
                (2, 2, 2.0),
            ],
        );
        let x_true = Vector::from_slice(&[1.0, -2.0, 3.0]);
        let b = a.mul_vec(&x_true);
        let x = solve_direct(&a, &b).unwrap();
        for i in 0..3 {
            assert!(
                (x[i] - x_true[i]).abs() < 1e-9,
                "index {i}: {} vs {}",
                x[i],
                x_true[i]
            );
        }
    }

    #[test]
    fn solve_direct_rejects_dimension_mismatch() {
        let a = SparseMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 1.0)]);
        let b = Vector::zeros(3);
        assert!(solve_direct(&a, &b).is_err());
    }
}
