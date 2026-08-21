// examples/math/sparse_solvers.rs
//! Sparse Linear Systems
//!
//! Demonstrates sparse solvers on a 1-D discrete Laplacian (the kind of
//! tridiagonal structure that shows up constantly in finite-difference
//! and finite-element discretizations):
//! - Iterative solving via conjugate gradient, with and without preconditioning
//! - A direct solve for comparison
//! - Partial eigendecomposition via Lanczos

use cnvx_math::{IterativeOptions, Preconditioner, SparseMatrix, Vector};

fn main() {
    // A tridiagonal SPD matrix (2 on the diagonal, -1 on the off-diagonals).
    let n = 200;
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
    let laplacian = SparseMatrix::from_triplets(n, n, &triplets);

    let true_solution =
        Vector::from((0..n).map(|i| (i as f64).sin()).collect::<Vec<_>>());
    let rhs = laplacian.mul_vec(&true_solution);

    // Conjugate gradient without preconditioning.
    let cg_plain = laplacian
        .solve_cg(&rhs, &IterativeOptions::default())
        .expect("CG should converge on an SPD system");
    println!("CG (no preconditioner): {} iterations", cg_plain.iterations);

    // The same system with a Jacobi preconditioner.
    let jacobi_options = IterativeOptions {
        preconditioner: Preconditioner::Jacobi,
        ..Default::default()
    };
    let cg_jacobi =
        laplacian.solve_cg(&rhs, &jacobi_options).expect("CG should converge");
    println!("CG (Jacobi preconditioner): {} iterations", cg_jacobi.iterations);

    // And with incomplete Cholesky, usually stronger still.
    let incomplete_cholesky_options = IterativeOptions {
        preconditioner: Preconditioner::IncompleteCholesky,
        ..Default::default()
    };
    let cg_incomplete_cholesky = laplacian
        .solve_cg(&rhs, &incomplete_cholesky_options)
        .expect("CG should converge");
    println!(
        "CG (incomplete Cholesky): {} iterations",
        cg_incomplete_cholesky.iterations
    );

    // A direct solve (via UMFPACK)
    let direct_solution = laplacian.solve_direct(&rhs).expect("A is non-singular");
    let max_error = direct_solution
        .iter()
        .zip(true_solution.iter())
        .map(|(&a, &b)| (a - b).abs())
        .fold(0.0, f64::max);
    println!("solve_direct max error vs. true solution: {max_error:.2e}");

    // Partial eigendecomposition via Lanczos: the 3 largest eigenpairs,
    // without ever forming a dense n x n matrix.
    let lanczos_options = IterativeOptions { max_iter: 40, ..Default::default() };
    let top_eigenpairs = laplacian
        .largest_eigenpairs(3, &lanczos_options)
        .expect("A is symmetric");
    println!("3 largest eigenvalues: {:?}", top_eigenpairs.values);

    // Expected output:
    // (values below are rounded to 3-4 decimal places for readability)
    //
    // CG (no preconditioner): 199 iterations
    // CG (Jacobi preconditioner): 199 iterations
    // CG (incomplete Cholesky): 1 iterations
    // solve_direct max error vs. true solution: 8.73e-14
    // 3 largest eigenvalues: Vector { data: [3.994, 3.975, 3.945] }
}
