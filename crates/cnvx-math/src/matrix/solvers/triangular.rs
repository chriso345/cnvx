use crate::matrix::{DenseMatrix, Matrix};
use cblas::{Diagonal, Layout, Part, Transpose, dtrsv};

pub enum TriType {
    Upper,
    Lower,
    Diagonal,
}

pub fn check_structure(a: &DenseMatrix) -> Option<TriType> {
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

pub fn solve(a: &DenseMatrix, b: &[f64], tri_type: TriType) -> Result<Vec<f64>, String> {
    let n = a.rows() as i32;
    let mut x = b.to_vec();

    // Check for exact zeros on the diagonal to avoid NaN pollution before CBLAS
    for i in 0..a.rows() {
        if a.get(i, i).abs() < 1e-12 {
            return Err("Singular matrix".into());
        }
    }

    let uplo = match tri_type {
        TriType::Upper => Part::Upper,
        TriType::Lower => Part::Lower,
        TriType::Diagonal => {
            // Pure diagonal division is faster than invoking BLAS
            for (i, xi) in x.iter_mut().enumerate() {
                *xi /= a.get(i, i);
            }
            return Ok(x);
        }
    };

    unsafe {
        dtrsv(
            Layout::RowMajor,
            uplo,
            Transpose::None,
            Diagonal::Generic,
            n,
            a.data(),
            n, // lda
            &mut x,
            1, // incx
        );
    }
    Ok(x)
}
