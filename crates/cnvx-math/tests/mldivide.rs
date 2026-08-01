use cnvx_math::{DenseMatrix, Matrix};

#[test]
fn solves_simple_system() {
    let mut a = DenseMatrix::new(2, 2);

    a.set(0, 0, 2.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 3.0);

    let rhs = vec![3.0, 7.0];
    let x = a.mldivide(&rhs).unwrap();

    assert!((x[0] - 0.4).abs() < 1e-6);
    assert!((x[1] - 2.2).abs() < 1e-6);
}

#[test]
fn identity_matrix_returns_rhs() {
    let mut a = DenseMatrix::new(3, 3);

    for i in 0..3 {
        a.set(i, i, 1.0);
    }

    let rhs = vec![5.0, -2.0, 10.0];
    a.mldivide(&rhs).unwrap();

    assert_eq!(rhs, vec![5.0, -2.0, 10.0]);
}

#[test]
fn singular_matrix_errors() {
    let mut a = DenseMatrix::new(2, 2);

    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(1, 0, 2.0);
    a.set(1, 1, 4.0); // dependent row

    let rhs = vec![3.0, 6.0];
    let result = a.mldivide(&rhs);

    assert!(result.is_err());
}

#[test]
fn non_square_matrix_errors() {
    let a = DenseMatrix::new(2, 3);
    let rhs = vec![1.0, 2.0];

    assert!(a.mldivide(&rhs).is_err());
}
