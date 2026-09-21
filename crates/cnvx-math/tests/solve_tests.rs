use cnvx_math::{MathError, Matrix, Vector};

#[test]
fn solves_simple_system() {
    let mut a = Matrix::zeros(2, 2);
    a.set(0, 0, 2.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 3.0);

    let rhs = Vector::from_slice(&[3.0, 7.0]);
    let x = a.solve(&rhs).unwrap();

    assert!((x[0] - 0.4).abs() < 1e-6);
    assert!((x[1] - 2.2).abs() < 1e-6);
}

#[test]
fn identity_matrix_returns_rhs() {
    let a = Matrix::identity(3);
    let rhs = Vector::from_slice(&[5.0, -2.0, 10.0]);
    let x = a.solve(&rhs).unwrap();
    assert_eq!(x.to_vec(), vec![5.0, -2.0, 10.0]);
}

#[test]
fn solve_triangular() {
    // Upper triangular
    let mut a = Matrix::zeros(2, 2);
    a.set(0, 0, 2.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 0.0);
    a.set(1, 1, 3.0);
    let b = Vector::from_slice(&[5.0, 6.0]);
    let x = a.solve(&b).unwrap();
    assert_eq!(x.to_vec(), vec![1.5, 2.0]);
}

#[test]
fn solve_cholesky_spd() {
    // Symmetric positive definite
    let mut a = Matrix::zeros(2, 2);
    a.set(0, 0, 4.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 3.0);
    let b = Vector::from_slice(&[1.0, 2.0]);
    let x = a.solve(&b).unwrap();
    // Solved via Cholesky automatically, since A is symmetric PD.
    assert!((x[0] - 0.09090909).abs() < 1e-6);
    assert!((x[1] - 0.63636363).abs() < 1e-6);
}

#[test]
fn solve_lu_general_square() {
    // General square (requires LU with pivoting)
    let mut a = Matrix::zeros(2, 2);
    a.set(0, 0, 0.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 0.0);
    let b = Vector::from_slice(&[2.0, 3.0]);
    let x = a.solve(&b).unwrap();
    assert_eq!(x.to_vec(), vec![3.0, 2.0]);
}

#[test]
fn solve_qr_least_squares() {
    // 3x2 overdetermined system (line fitting y = mx + c)
    let mut a = Matrix::zeros(3, 2);
    a.set(0, 0, 1.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 2.0);
    a.set(1, 1, 1.0);
    a.set(2, 0, 3.0);
    a.set(2, 1, 1.0);
    let b = Vector::from_slice(&[1.2, 1.9, 3.2]);
    let x = a.solve(&b).unwrap();

    // x[0] == slope, x[1] == y-intercept
    assert!((x[0] - 1.0).abs() < 1e-5);
    assert!((x[1] - 0.1).abs() < 1e-5);
}

#[test]
fn singular_matrix_errors() {
    let mut a = Matrix::zeros(2, 2);
    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(1, 0, 2.0);
    a.set(1, 1, 4.0); // dependent row

    let rhs = Vector::from_slice(&[3.0, 6.0]);
    let result = a.solve(&rhs);
    assert!(result.is_err());
}

#[test]
fn non_square_matrix_uses_least_squares_not_an_error() {
    // rows != cols dispatches to QR least squares, not an immediate error,
    // as long as it's overdetermined (rows >= cols).
    let mut a = Matrix::zeros(3, 2);
    a.set(0, 0, 1.0);
    a.set(0, 1, 0.0);
    a.set(1, 0, 0.0);
    a.set(1, 1, 1.0);
    a.set(2, 0, 1.0);
    a.set(2, 1, 1.0);
    let rhs = Vector::from_slice(&[1.0, 2.0, 3.0]);
    assert!(a.solve(&rhs).is_ok());
}

#[test]
fn underdetermined_system_is_unsupported() {
    let a = Matrix::zeros(2, 3);
    let rhs = Vector::from_slice(&[1.0, 2.0]);
    let err = a.solve(&rhs).unwrap_err();
    assert!(matches!(err, MathError::Unsupported(_)));
}

#[test]
fn rhs_length_mismatch_is_dimension_error() {
    let a = Matrix::identity(3);
    let rhs = Vector::from_slice(&[1.0, 2.0]);
    let err = a.solve(&rhs).unwrap_err();
    assert!(matches!(err, MathError::DimensionMismatch(_)));
}
