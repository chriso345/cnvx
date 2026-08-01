use cnvx_math::{DenseMatrix, Matrix};

#[test]
fn creates_matrix_with_correct_shape() {
    let m = DenseMatrix::new(3, 4);
    assert_eq!(m.rows(), 3);
    assert_eq!(m.cols(), 4);
}

#[test]
fn set_and_get_work() {
    let mut m = DenseMatrix::new(2, 2);

    m.set(0, 0, 1.0);
    m.set(1, 1, 5.0);

    assert_eq!(m.get(0, 0), 1.0);
    assert_eq!(m.get(1, 1), 5.0);
}

#[test]
#[should_panic]
fn get_out_of_bounds_panics() {
    let m = DenseMatrix::new(2, 2);
    m.get(10, 0);
}

#[test]
#[should_panic]
fn set_out_of_bounds_panics() {
    let mut m = DenseMatrix::new(2, 2);
    m.set(0, 10, 1.0);
}
