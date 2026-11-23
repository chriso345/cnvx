//! 1D Finite Element Method: Poisson Equation
//!
//! Solves -u''(x) = f(x) on [0,1] with u(0)=u(1)=0
//! using linear finite elements and sparse solvers.

use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    println!("=== 1D FEM: Poisson Equation ===\n");

    // Problem: -u'' = f, u(0)=u(1)=0
    // Exact solution: u(x) = x(1-x), f(x) = 2

    let n_elements = 100; // Number of elements
    let n_nodes = n_elements + 1;
    let h = 1.0 / n_elements as f64; // Element size

    println!("Mesh: {n_elements} elements, {n_nodes} nodes, h = {h:.4}");

    // Assemble global stiffness matrix (tridiagonal)
    // K[i,i] = 2/h, K[i,i+1] = K[i+1,i] = -1/h
    let mut triplets = Vec::new();
    for i in 0..n_nodes {
        if i > 0 {
            triplets.push((i, i - 1, -1.0 / h));
        }
        triplets.push((i, i, 2.0 / h));
        if i + 1 < n_nodes {
            triplets.push((i, i + 1, -1.0 / h));
        }
    }

    let _k = SparseMatrix::from_triplets(n_nodes, n_nodes, &triplets);

    // Assemble load vector: f_i = int f(x) * phi_i(x) dx
    // For f(x) = 2, using midpoint rule: f_i = 2 * h (interior), h (boundary)
    let mut rhs = Vector::zeros(n_nodes);
    for i in 1..n_nodes - 1 {
        rhs[i] = 2.0 * h;
    }
    rhs[0] = h;
    rhs[n_nodes - 1] = h;

    // Apply Dirichlet BCs: u[0] = u[n-1] = 0
    // We build a new sparse matrix with BCs applied
    let mut bc_triplets = Vec::new();
    for i in 0..n_nodes {
        if i == 0 || i == n_nodes - 1 {
            // Boundary nodes: identity row
            bc_triplets.push((i, i, 1.0));
        } else {
            // Interior nodes: copy from original
            if i > 0 {
                bc_triplets.push((i, i - 1, -1.0 / h));
            }
            bc_triplets.push((i, i, 2.0 / h));
            if i + 1 < n_nodes {
                bc_triplets.push((i, i + 1, -1.0 / h));
            }
        }
    }
    let k_bc = SparseMatrix::from_triplets(n_nodes, n_nodes, &bc_triplets);

    // Apply BCs to RHS
    let mut rhs_bc = rhs.clone();
    rhs_bc[0] = 0.0;
    rhs_bc[n_nodes - 1] = 0.0;

    // Solve using iterative solver with Jacobi preconditioning
    let options = IterativeOptions {
        preconditioner: Preconditioner::Jacobi,
        tol: 1e-10,
        max_iter: 2000,
    };

    let result = k_bc.solve_cg(&rhs_bc, &options)?;
    let u = result.x;

    println!("CG converged in {} iterations", result.iterations);
    println!("Final residual: {:.2e}", result.residual_norm);

    // Compute exact solution and error
    let mut exact = Vector::zeros(n_nodes);
    for i in 0..n_nodes {
        let x = i as f64 * h;
        exact[i] = x * (1.0 - x);
    }

    let mut l2_error = 0.0;
    for i in 0..n_nodes {
        let diff = u[i] - exact[i];
        l2_error += diff * diff;
    }
    l2_error = l2_error.sqrt() * h.sqrt();

    let mut h1_error = 0.0;
    for i in 0..n_elements {
        let u_deriv = (u[i + 1] - u[i]) / h;
        let exact_deriv = 1.0 - 2.0 * (i as f64 + 0.5) * h;
        let diff = u_deriv - exact_deriv;
        h1_error += diff * diff;
    }
    h1_error = h1_error.sqrt() * h.sqrt();

    println!("\nL2 error: {l2_error:.2e}");
    println!("H1 error: {h1_error:.2e}");

    // Also solve directly for comparison
    let u_direct = k_bc.solve_direct(&rhs_bc)?;
    let mut direct_diff = 0.0;
    for i in 0..n_nodes {
        direct_diff += (u[i] - u_direct[i]).abs();
    }
    println!("Direct vs CG max diff: {direct_diff:.2e}");

    // Convergence study
    println!("\n=== Convergence Study ===");
    for &n in &[10, 20, 40, 80, 160] {
        let h = 1.0 / n as f64;
        let n_nodes = n + 1;

        let mut triplets = Vec::new();
        for i in 0..n_nodes {
            if i > 0 {
                triplets.push((i, i - 1, -1.0 / h));
            }
            triplets.push((i, i, 2.0 / h));
            if i + 1 < n_nodes {
                triplets.push((i, i + 1, -1.0 / h));
            }
        }
        let _k = SparseMatrix::from_triplets(n_nodes, n_nodes, &triplets);

        let mut rhs = Vector::zeros(n_nodes);
        for i in 1..n_nodes - 1 {
            rhs[i] = 2.0 * h;
        }
        rhs[0] = h;
        rhs[n_nodes - 1] = h;

        // Apply BCs - build new sparse matrix
        let mut bc_triplets = Vec::new();
        for i in 0..n_nodes {
            if i == 0 || i == n_nodes - 1 {
                bc_triplets.push((i, i, 1.0));
            } else {
                if i > 0 {
                    bc_triplets.push((i, i - 1, -1.0 / h));
                }
                bc_triplets.push((i, i, 2.0 / h));
                if i + 1 < n_nodes {
                    bc_triplets.push((i, i + 1, -1.0 / h));
                }
            }
        }
        let k_bc = SparseMatrix::from_triplets(n_nodes, n_nodes, &bc_triplets);

        let mut rhs_bc = rhs.clone();
        rhs_bc[0] = 0.0;
        rhs_bc[n_nodes - 1] = 0.0;

        let u = k_bc.solve_direct(&rhs_bc)?;

        let mut err_l2 = 0.0;
        for i in 0..n_nodes {
            let x = i as f64 * h;
            let exact = x * (1.0 - x);
            let diff = u[i] - exact;
            err_l2 += diff * diff;
        }
        err_l2 = err_l2.sqrt() * h.sqrt();

        println!("  n={n:3}: h={h:.4}, L2 error={err_l2:.2e}");
    }

    // Expected output:
    //
    // === 1D FEM: Poisson Equation ===
    //
    // Mesh: 100 elements, 101 nodes, h = 0.0100
    // CG converged in 50 iterations
    // Final residual: 1.04e-14
    //
    // L2 error: 4.50e-16
    // H1 error: 4.88e-15
    // Direct vs CG max diff: 3.64e-14
    //
    // === Convergence Study ===
    //   n= 10: h=0.1000, L2 error=1.06e-16
    //   n= 20: h=0.0500, L2 error=1.54e-16
    //   n= 40: h=0.0250, L2 error=5.44e-16
    //   n= 80: h=0.0125, L2 error=2.03e-15
    //   n=160: h=0.0063, L2 error=2.02e-15

    Ok(())
}
