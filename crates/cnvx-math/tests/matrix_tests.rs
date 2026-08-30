use cnvx_math::{Matrix, Vector};

#[test]
fn creates_matrix_with_correct_shape() {
    let m = Matrix::zeros(3, 4);
    assert_eq!(m.rows(), 3);
    assert_eq!(m.cols(), 4);
}

#[test]
fn set_and_get_work() {
    let mut m = Matrix::zeros(2, 2);
    m.set(0, 0, 1.0);
    m.set(1, 1, 5.0);

    assert_eq!(m.get(0, 0), 1.0);
    assert_eq!(m.get(1, 1), 5.0);
}

#[test]
#[should_panic]
fn get_out_of_bounds_panics() {
    let m = Matrix::zeros(2, 2);
    m.get(10, 0);
}

#[test]
#[should_panic]
fn set_out_of_bounds_panics() {
    let mut m = Matrix::zeros(2, 2);
    m.set(0, 10, 1.0);
}

#[test]
fn scalar_operations() {
    let mut m = Matrix::zeros(2, 2);
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
fn elementwise_multiplication() {
    let mut a = Matrix::zeros(2, 2);
    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(1, 0, 3.0);
    a.set(1, 1, 4.0);

    let mut b = Matrix::zeros(2, 2);
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
fn mismatched_shapes_return_dimension_mismatch() {
    let a = Matrix::zeros(2, 2);
    let b = Matrix::zeros(3, 3);
    assert!(a.add(&b).is_err());
    assert!(a.mul_elementwise(&b).is_err());
}

#[test]
fn row_col_manipulations() {
    let mut m = Matrix::zeros(3, 3);
    for i in 0..3 {
        for j in 0..3 {
            m.set(i, j, (i * 3 + j) as f64);
        }
    }
    // Matrix is:
    // [0, 1, 2]
    // [3, 4, 5]
    // [6, 7, 8]

    assert_eq!(m.get_row(1).to_vec(), vec![3.0, 4.0, 5.0]);
    assert_eq!(m.get_col(2).to_vec(), vec![2.0, 5.0, 8.0]);

    m.set_row(0, &[9.0, 9.0, 9.0]).unwrap();
    assert_eq!(m.get(0, 1), 9.0);

    m.set_col(1, &[0.0, 0.0, 0.0]).unwrap();
    assert_eq!(m.get(2, 1), 0.0);

    m.swap_rows(1, 2);
    assert_eq!(m.get_row(1).to_vec(), vec![6.0, 0.0, 8.0]);
    assert_eq!(m.get_row(2).to_vec(), vec![3.0, 0.0, 5.0]);
}

#[test]
fn norms() {
    let mut m = Matrix::zeros(2, 2);
    m.set(0, 0, -3.0);
    m.set(0, 1, 4.0); // abs sum = 7
    m.set(1, 0, 1.0);
    m.set(1, 1, -8.0); // abs sum = 9

    assert_eq!(m.norm_inf(), 9.0);

    let v = Vector::from_slice(&[3.0, -4.0, 0.0]);
    assert_eq!(v.norm(), 5.0);
}

#[test]
fn constructors() {
    let id = Matrix::identity(3);
    assert_eq!(id.get(0, 0), 1.0);
    assert_eq!(id.get(1, 1), 1.0);
    assert_eq!(id.get(0, 1), 0.0);

    let diag = Matrix::from_diagonal(&[1.5, 2.5]);
    assert_eq!(diag.rows(), 2);
    assert_eq!(diag.cols(), 2);
    assert_eq!(diag.get(0, 0), 1.5);
    assert_eq!(diag.get(1, 1), 2.5);

    let mut m = Matrix::zeros(2, 3);
    m.set(0, 0, 7.0);
    m.set(1, 1, 8.0);
    assert_eq!(m.diagonal().to_vec(), vec![7.0, 8.0]);
}

#[test]
fn transpose_and_mul() {
    let mut a = Matrix::zeros(2, 3);
    for i in 0..2 {
        for j in 0..3 {
            a.set(i, j, (i * 3 + j) as f64);
        }
    }
    let t = a.transpose();
    assert_eq!(t.rows(), 3);
    assert_eq!(t.cols(), 2);
    assert_eq!(t.get(2, 1), a.get(1, 2));

    let product = a.mul(&t).unwrap();
    assert_eq!(product.rows(), 2);
    assert_eq!(product.cols(), 2);
}

#[test]
fn mul_vec_matches_identity() {
    let id = Matrix::identity(3);
    let v = Vector::from_slice(&[5.0, -2.0, 10.0]);
    assert_eq!(id.mul_vec(&v).to_vec(), vec![5.0, -2.0, 10.0]);
}

#[test]
#[should_panic]
fn mul_vec_dimension_mismatch_panics() {
    let m = Matrix::zeros(2, 3);
    let v = Vector::from_slice(&[1.0, 2.0]);
    let _ = m.mul_vec(&v);
}
