use cnvx_math::{DenseMatrix, Matrix, matrix::vector_norm_l2};

const EPSILON: f64 = 1e-9;

#[test]
fn test_scalar_operations() {
    let mut m = DenseMatrix::new(2, 2);
    m.set(0, 0, 1.0);
    m.set(0, 1, 2.0);
    m.set(1, 0, 3.0);
    m.set(1, 1, 4.0);

    let add_m = m.add_scalar(2.0);
    assert_eq!(add_m.get(0, 0), 3.0);
    assert_eq!(add_m.get(1, 1), 6.0);

    let mul_m = m.mul_scalar(3.0);
    assert_eq!(mul_m.get(0, 1), 6.0);
    assert_eq!(mul_m.get(1, 0), 9.0);
}

#[test]
fn test_elementwise_multiplication() {
    let mut a = DenseMatrix::new(2, 2);
    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(1, 0, 3.0);
    a.set(1, 1, 4.0);

    let mut b = DenseMatrix::new(2, 2);
    b.set(0, 0, 2.0);
    b.set(0, 1, 3.0);
    b.set(1, 0, 4.0);
    b.set(1, 1, 5.0);

    let c = a.mul_elementwise(&b).unwrap();
    assert_eq!(c.get(0, 0), 2.0);
    assert_eq!(c.get(0, 1), 6.0);
    assert_eq!(c.get(1, 0), 12.0);
    assert_eq!(c.get(1, 1), 20.0);
}

#[test]
fn test_row_col_manipulations() {
    let mut m = DenseMatrix::new(3, 3);
    for i in 0..3 {
        for j in 0..3 {
            m.set(i, j, (i * 3 + j) as f64);
        }
    }
    // Matrix is:
    // [0, 1, 2]
    // [3, 4, 5]
    // [6, 7, 8]

    assert_eq!(m.get_row(1), vec![3.0, 4.0, 5.0]);
    assert_eq!(m.get_col(2), vec![2.0, 5.0, 8.0]);

    m.set_row(0, &[9.0, 9.0, 9.0]).unwrap();
    assert_eq!(m.get(0, 1), 9.0);

    m.set_col(1, &[0.0, 0.0, 0.0]).unwrap();
    assert_eq!(m.get(2, 1), 0.0);

    m.swap_rows(1, 2);
    assert_eq!(m.get_row(1), vec![6.0, 0.0, 8.0]);
    assert_eq!(m.get_row(2), vec![3.0, 0.0, 5.0]);
}

#[test]
fn test_norms() {
    let mut m = DenseMatrix::new(2, 2);
    m.set(0, 0, -3.0);
    m.set(0, 1, 4.0); // abs sum = 7
    m.set(1, 0, 1.0);
    m.set(1, 1, -8.0); // abs sum = 9

    assert_eq!(m.norm_inf(), 9.0);

    let v = vec![3.0, -4.0, 0.0];
    assert_eq!(vector_norm_l2(&v), 5.0);
}

#[test]
fn test_constructors() {
    let id = DenseMatrix::identity(3);
    assert_eq!(id.get(0, 0), 1.0);
    assert_eq!(id.get(1, 1), 1.0);
    assert_eq!(id.get(0, 1), 0.0);

    let diag = DenseMatrix::from_diagonal(&[1.5, 2.5]);
    assert_eq!(diag.rows(), 2);
    assert_eq!(diag.cols(), 2);
    assert_eq!(diag.get(0, 0), 1.5);
    assert_eq!(diag.get(1, 1), 2.5);

    let mut m = DenseMatrix::new(2, 3);
    m.set(0, 0, 7.0);
    m.set(1, 1, 8.0);
    assert_eq!(m.diagonal(), vec![7.0, 8.0]);
}

#[test]
fn test_mldivide_success() {
    let mut a = DenseMatrix::new(2, 2);
    a.set(0, 0, 2.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 3.0);

    let mut rhs = vec![3.0, 7.0];
    a.mldivide(&mut rhs).unwrap();

    assert!((rhs[0] - 0.4).abs() < EPSILON);
    assert!((rhs[1] - 2.2).abs() < EPSILON);
}

#[test]
fn test_mldivide_singular_matrix() {
    let mut a = DenseMatrix::new(2, 2);
    a.set(0, 0, 1.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 1.0); // Singular

    let mut rhs = vec![2.0, 2.0];
    let result = a.mldivide(&mut rhs);

    assert!(result.is_err());
}
