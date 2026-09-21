//! Pure-Rust structure detection shared by the native and wasm solver
//! backends.

use crate::matrix::Matrix;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TriType {
    Upper,
    Lower,
    Diagonal,
}

/// Detects whether `a` is (numerically) upper-triangular, lower-triangular,
/// or diagonal, so `solve` can take a cheap substitution fast-path instead
/// of a full factorization.
pub(crate) fn check_triangular(a: &Matrix) -> Option<TriType> {
    let n = a.rows();
    let mut is_upper = true;
    let mut is_lower = true;

    for i in 0..n {
        for j in 0..n {
            let val = a.get(i, j).abs();
            if i > j && val > 1e-12 {
                is_upper = false;
            }
            if i < j && val > 1e-12 {
                is_lower = false;
            }
        }
    }

    if is_upper && is_lower {
        Some(TriType::Diagonal)
    } else if is_upper {
        Some(TriType::Upper)
    } else if is_lower {
        Some(TriType::Lower)
    } else {
        None
    }
}

/// Checks if `a` is symmetric: `a[i][j] == a[j][i]`.
pub(crate) fn is_symmetric(a: &Matrix) -> bool {
    let n = a.rows();
    for i in 0..n {
        for j in (i + 1)..n {
            if (a.get(i, j) - a.get(j, i)).abs() > 1e-12 {
                return false;
            }
        }
    }
    true
}
