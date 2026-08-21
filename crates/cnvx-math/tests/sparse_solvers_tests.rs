use cnvx_math::{IterativeOptions, Preconditioner, SparseMatrix, Vector};

fn spd_tridiagonal(n: usize) -> SparseMatrix {
    let mut triplets = Vec::new();
    for i in 0..n {
        triplets.push((i, i, 2.0));
        if i > 0 {
            triplets.push((i, i - 1, -1.0));
        }
        if i + 1 < n {
            triplets.push((i, i + 1, -1.0));
        }
    }
    SparseMatrix::from_triplets(n, n, &triplets)
}

#[test]
fn solve_cg_matches_known_solution() {
    let a = spd_tridiagonal(10);
    let x_true = Vector::from((0..10).map(|i| i as f64 + 1.0).collect::<Vec<_>>());
    let b = a.mul_vec(&x_true);
    let result = a.solve_cg(&b, &IterativeOptions::default()).unwrap();
    for i in 0..10 {
        assert!((result.x[i] - x_true[i]).abs() < 1e-6);
    }
}

#[test]
fn solve_direct_matches_iterative_solution() {
    let a = spd_tridiagonal(6);
    let x_true = Vector::from_slice(&[1.0, -1.0, 2.0, -2.0, 3.0, -3.0]);
    let b = a.mul_vec(&x_true);
    let direct = a.solve_direct(&b).unwrap();
    for i in 0..6 {
        assert!((direct[i] - x_true[i]).abs() < 1e-9);
    }
}

#[test]
fn smallest_and_largest_eigenpairs_are_consistent_with_dense() {
    let n = 15;
    let a = spd_tridiagonal(n);
    let dense_eig = a.to_dense().eigen_symmetric().unwrap();

    let options = IterativeOptions { max_iter: n as u32, ..Default::default() };
    let largest = a.largest_eigenpairs(3, &options).unwrap();
    let smallest = a.smallest_eigenpairs(3, &options).unwrap();

    for i in 0..3 {
        assert!((largest.values[i] - dense_eig.values[i]).abs() < 1e-5);
    }
    for i in 0..3 {
        assert!((smallest.values[i] - dense_eig.values[n - 1 - i]).abs() < 1e-5);
    }
}

#[test]
fn preconditioners_all_converge_to_the_same_answer() {
    let a = spd_tridiagonal(12);
    let x_true = Vector::ones(12);
    let b = a.mul_vec(&x_true);

    for preconditioner in
        [Preconditioner::None, Preconditioner::Jacobi, Preconditioner::IncompleteCholesky]
    {
        let options = IterativeOptions { preconditioner, ..Default::default() };
        let result = a.solve_cg(&b, &options).unwrap();
        for i in 0..12 {
            assert!(
                (result.x[i] - 1.0).abs() < 1e-5,
                "preconditioner {preconditioner:?}, index {i}"
            );
        }
    }
}

#[test]
fn sparse_matrix_ergonomics() {
    let a = spd_tridiagonal(4);
    assert!(a.is_symmetric(1e-12));
    let dense = a.to_dense();
    assert_eq!(dense.rows(), 4);
    let transposed = a.transpose();
    assert!(transposed.is_symmetric(1e-12));
    let diag = a.diagonal();
    for &d in diag.iter() {
        assert!((d - 2.0).abs() < 1e-12);
    }
    let squared = a.mul_sparse(&a).unwrap();
    assert_eq!(squared.rows(), 4);
    assert_eq!(squared.cols(), 4);
}
