//! Dense Linear Algebra
//!
//! Demonstrates decompositions, solving, and the quantities derived
//! from them on a small symmetric positive-definite matrix:
//! - Solving a linear system, directly and via an explicit Cholesky
//!   factorization
//! - Eigendecomposition, determinant, rank, and condition number
//! - Least-squares regression via QR

use cnvx_math::{Matrix, Vector};

fn main() {
    // A symmetric positive-definite matrix (a small covariance-like matrix).
    let mut spd_matrix = Matrix::zeros(3, 3);
    spd_matrix.set(0, 0, 4.0);
    spd_matrix.set(0, 1, 1.0);
    spd_matrix.set(0, 2, 0.0);
    spd_matrix.set(1, 0, 1.0);
    spd_matrix.set(1, 1, 3.0);
    spd_matrix.set(1, 2, 1.0);
    spd_matrix.set(2, 0, 0.0);
    spd_matrix.set(2, 1, 1.0);
    spd_matrix.set(2, 2, 2.0);

    println!("A =\n{spd_matrix:?}\n");

    // Matrix::solve dispatches to the right method automatically (here,
    // Cholesky, since the matrix is symmetric positive-definite).
    let rhs = Vector::from_slice(&[1.0, 2.0, 3.0]);
    let solution = spd_matrix
        .solve(&rhs)
        .expect("solve should succeed for a well-conditioned SPD matrix");
    println!("solve(A, b) = {solution:?}");

    // The same system, via an explicit Cholesky factorization.
    let cholesky = spd_matrix.cholesky().expect("A is SPD");
    println!("Cholesky factor L =\n{:?}", cholesky.l);

    // Eigendecomposition: for an SPD matrix, every eigenvalue is positive.
    let eigen = spd_matrix.eigen_symmetric().expect("A is square");
    println!("eigenvalues = {:?}", eigen.values);
    assert!(eigen.values.iter().all(|&v| v > 0.0));

    // Derived quantities, all built on SVD under the hood.
    println!("determinant = {}", spd_matrix.determinant().unwrap());
    println!("rank = {}", spd_matrix.rank(1e-10).unwrap());
    println!("condition number = {:.4}", spd_matrix.condition_number().unwrap());

    // Least-squares regression via QR: fit y = m*x + c to noisy points.
    let mut design = Matrix::zeros(5, 2);
    let x_values = [0.0, 1.0, 2.0, 3.0, 4.0];
    for (i, &x) in x_values.iter().enumerate() {
        design.set(i, 0, x);
        design.set(i, 1, 1.0);
    }
    let y_values = Vector::from_slice(&[1.05, 2.95, 5.02, 6.98, 9.03]); // ~= 2x + 1
    let fit = design
        .least_squares(&y_values)
        .expect("design matrix has full column rank");
    println!("least-squares fit: y ~= {:.3} x + {:.3}", fit[0], fit[1]);

    // Expected output:
    // (values below are rounded to 3-4 decimal places for readability)
    //
    // solve(A, b) = [0.222, 0.111, 1.444]
    //
    // Cholesky factor L =
    // [[2.000, 0.000, 0.000],
    //  [0.500, 1.658, 0.000],
    //  [0.000, 0.603, 1.279]]
    //
    // eigenvalues = Vector { data: [4.732, 3.000, 1.268] }
    // determinant = 18
    // rank = 3
    // condition number = 3.7321
    // least-squares fit: y ~= 1.999 x + 1.008
}
