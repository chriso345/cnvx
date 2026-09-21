use cnvx_math::{SparseMatrix, Vector};

#[test]
fn from_triplets_and_get() {
    // [1, 0, 2]
    // [0, 0, 0]
    // [0, 3, 0]
    let m = SparseMatrix::from_triplets(3, 3, &[(0, 0, 1.0), (0, 2, 2.0), (2, 1, 3.0)]);

    assert_eq!(m.rows(), 3);
    assert_eq!(m.cols(), 3);
    assert_eq!(m.nnz(), 3);

    assert_eq!(m.get(0, 0), 1.0);
    assert_eq!(m.get(0, 2), 2.0);
    assert_eq!(m.get(2, 1), 3.0);
    assert_eq!(m.get(1, 1), 0.0); // not stored -> implicit zero
}

#[test]
fn duplicate_triplets_are_summed() {
    let m = SparseMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 0, 2.0)]);
    assert_eq!(m.nnz(), 1);
    assert_eq!(m.get(0, 0), 3.0);
}

#[test]
fn mul_vec() {
    // [1, 0, 2]   [1]   [1*1 + 2*3]   [7]
    // [0, 3, 0] * [1] = [3*1]       = [3]
    // [4, 0, 0]   [3]   [4*1]         [4]
    let m = SparseMatrix::from_triplets(
        3,
        3,
        &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0), (2, 0, 4.0)],
    );
    let v = Vector::from_slice(&[1.0, 1.0, 3.0]);
    assert_eq!(m.mul_vec(&v).to_vec(), vec![7.0, 3.0, 4.0]);
}

#[test]
fn try_mul_vec_reports_dimension_mismatch() {
    let m = SparseMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]);
    let v = Vector::from_slice(&[1.0, 2.0]); // wrong length: needs 3
    assert!(m.try_mul_vec(&v).is_err());
}

#[test]
#[should_panic]
fn from_triplets_out_of_bounds_panics() {
    SparseMatrix::from_triplets(2, 2, &[(5, 0, 1.0)]);
}
