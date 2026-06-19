use cnvx_math::{DenseMatrix, Matrix};

#[test]
fn test_mldivide_triangular() {
    // Upper triangular
    let mut a = DenseMatrix::new(2, 2);
    a.set(0, 0, 2.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 0.0);
    a.set(1, 1, 3.0);
    let b = vec![5.0, 6.0];
    let x = a.mldivide(&b).unwrap();
    assert_eq!(x, vec![1.5, 2.0]);
}

#[test]
fn test_mldivide_cholesky() {
    // Symmetric Positive Definite
    let mut a = DenseMatrix::new(2, 2);
    a.set(0, 0, 4.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 3.0);
    let b = vec![1.0, 2.0];
    let x = a.mldivide(&b).unwrap();
    // Validates via Cholesky decomposition automatically
    assert!((x[0] - 0.09090909).abs() < 1e-6);
    assert!((x[1] - 0.63636363).abs() < 1e-6);
}

#[test]
fn test_mldivide_lu() {
    // General Square (Requires LU Pivot)
    let mut a = DenseMatrix::new(2, 2);
    a.set(0, 0, 0.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 0.0);
    let b = vec![2.0, 3.0];
    let x = a.mldivide(&b).unwrap();
    assert_eq!(x, vec![3.0, 2.0]);
}

#[test]
fn test_mldivide_qr_least_squares() {
    // 3x2 Overdetermined system (Line fitting y = mx + c)
    let mut a = DenseMatrix::new(3, 2);
    a.set(0, 0, 1.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 2.0);
    a.set(1, 1, 1.0);
    a.set(2, 0, 3.0);
    a.set(2, 1, 1.0);
    let b = vec![1.2, 1.9, 3.2];
    let x = a.mldivide(&b).unwrap();

    // Result x[0] == slope, x[1] == y-intercept
    assert!((x[0] - 1.0).abs() < 1e-5);
    assert!((x[1] - 0.1).abs() < 1e-5);
}
