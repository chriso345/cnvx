//! Gradient-Based Optimization
//!
//! Demonstrates using gradients and Hessians for unconstrained optimization:
//! - Gradient descent with line search
//! - Newton's method
//! - BFGS quasi-Newton (using finite-difference Hessian)

use cnvx::math::calculus::*;
use cnvx::prelude::*;

#[rustfmt::skip]
fn main() -> Result<(), cnvx_core::CnvxError> {
    // Rosenbrock function: f(x,y) = (1-x)^2 + 100(y-x^2)^2
    // Global minimum at (1, 1) with f = 0
    let rosenbrock = |p: &Vector| {
        let x = p[0];
        let y = p[1];
        (1.0 - x).powi(2) + 100.0 * (y - x * x).powi(2)
    };

    // Gradient: [ -2(1-x) - 400x(y-x^2), 200(y-x^2) ]
    let rosenbrock_grad = |p: &Vector| {
        let x = p[0];
        let y = p[1];
        Vector::from_slice(&[
            -2.0 * (1.0 - x) - 400.0 * x * (y - x * x),
            200.0 * (y - x * x),
        ])
    };

    // Hessian: [ 2 - 400(y-x^2) + 800x^2, -400x; -400x, 200 ]
    let rosenbrock_hess = |p: &Vector| {
        let x = p[0];
        let y = p[1];
        let mut h = Matrix::zeros(2, 2);
        h.set(0, 0, 2.0 - 400.0 * (y - x * x) + 800.0 * x * x);
        h.set(0, 1, -400.0 * x);
        h.set(1, 0, -400.0 * x);
        h.set(1, 1, 200.0);
        h
    };

    let start = Vector::from_slice(&[-1.2, 1.0]);
    println!("=== Optimization of Rosenbrock Function ===");
    println!("Start: {start:?}, f(start) = {:.6}", rosenbrock(&start));
    println!("True minimum: [1, 1], f = 0\n");

    // 1. Gradient Descent with backtracking line search
    println!("--- Gradient Descent (backtracking line search) ---");
    let mut x = start.clone();
    let mut f = rosenbrock(&x);
    let mut grad = rosenbrock_grad(&x);
    let max_iter = 10000;
    let tol = 1e-8;

    for iter in 0..max_iter {
        let grad_norm = grad.norm();
        if grad_norm < tol {
            println!("Converged in {iter} iterations: f = {f:.6}, x = {x:?}");
            break;
        }

        // Backtracking line search
        let mut alpha = 1.0;
        let c = 1e-4;
        let rho = 0.5;
        let mut x_new;
        let mut f_new;

        loop {
            x_new = &x - &(&grad * alpha);
            f_new = rosenbrock(&x_new);
            if f_new <= f + c * alpha * (-grad_norm * grad_norm) {
                break;
            }
            alpha *= rho;
            if alpha < 1e-16 {
                println!("Line search failed");
                break;
            }
        }

        x = x_new;
        f = f_new;
        grad = rosenbrock_grad(&x);
    }

    // 2. Newton's Method
    println!("\n--- Newton's Method ---");
    let mut x = start.clone();
    let mut f = rosenbrock(&x);
    let mut grad = rosenbrock_grad(&x);

    for iter in 0..100 {
        let grad_norm = grad.norm();
        if grad_norm < tol {
            println!("Converged in {iter} iterations: f = {f:.6}, x = {x:?}");
            break;
        }

        let hess = rosenbrock_hess(&x);
        let neg_grad = Vector::from_slice(&[-grad[0], -grad[1]]);
        let p = hess.solve(&neg_grad)?;

        // Line search
        let mut alpha = 1.0;
        let c = 1e-4;
        let rho = 0.5;
        loop {
            let x_new = &x + &(&p * alpha);
            let f_new = rosenbrock(&x_new);
            if f_new <= f + c * alpha * grad.dot(&p) {
                x = x_new;
                f = f_new;
                break;
            }
            alpha *= rho;
        }
        grad = rosenbrock_grad(&x);
    }

    // 3. Finite-difference Newton (no analytic Hessian)
    println!("\n--- Finite-Difference Newton (no analytic Hessian) ---");
    let mut x = start.clone();
    let mut f = rosenbrock(&x);

    for iter in 0..100 {
        let grad = gradient(rosenbrock, &x, 1e-6);
        let grad_norm = grad.norm();
        if grad_norm < tol {
            println!("Converged in {iter} iterations: f = {f:.6}, x = {x:?}");
            break;
        }

        let hess = hessian(rosenbrock, &x, 1e-4);
        let neg_grad = Vector::from_slice(&[-grad[0], -grad[1]]);
        let p = hess.solve(&neg_grad)?;

        let mut alpha = 1.0;
        let c = 1e-4;
        let rho = 0.5;
        loop {
            let x_new = &x + &(&p * alpha);
            let f_new = rosenbrock(&x_new);
            if f_new <= f + c * alpha * grad.dot(&p) {
                x = x_new;
                f = f_new;
                break;
            }
            alpha *= rho;
        }
    }

    // 4. Using finite-difference for a black-box function
    println!("\n--- Black-box optimization (finite-diff gradient only) ---");
    // Himmelblau's function: f(x,y) = (x^2+y-11)^2 + (x+y^2-7)^2
    // Four minima at (3,2), (-2.805, 3.131), (-3.779, -3.283), (3.584, -1.848)
    let himmelblau = |p: &Vector| {
        let x = p[0];
        let y = p[1];
        (x * x + y - 11.0).powi(2) + (x + y * y - 7.0).powi(2)
    };

    let starts = [
        Vector::from_slice(&[0.0, 0.0]),
        Vector::from_slice(&[5.0, 5.0]),
        Vector::from_slice(&[-5.0, 5.0]),
        Vector::from_slice(&[5.0, -5.0]),
    ];

    for (i, start) in starts.iter().enumerate() {
        let mut x = start.clone();
        let mut f = himmelblau(&x);

        for _iter in 0..500 {
            let grad = gradient(himmelblau, &x, 1e-6);
            if grad.norm() < 1e-8 {
                println!("  Start {i}: {start:?} -> x = {x:?}, f = {f:.6}");
                break;
            }
            let hess = hessian(himmelblau, &x, 1e-4);
            let neg_grad = Vector::from_slice(&[-grad[0], -grad[1]]);
            if let Ok(p) = hess.solve(&neg_grad) {
                let mut alpha = 1.0;
                let c = 1e-4;
                let rho = 0.5;
                loop {
                    let x_new = &x + &(&p * alpha);
                    let f_new = himmelblau(&x_new);
                    if f_new <= f + c * alpha * grad.dot(&p) {
                        x = x_new;
                        f = f_new;
                        break;
                    }
                    alpha *= rho;
                    if alpha < 1e-16 {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    // Expected output:
    //
    // === Optimization of Rosenbrock Function ===
    // Start: Vector { data: [-1.2, 1.0] }, f(start) = 24.200000
    // True minimum: [1, 1], f = 0
    //
    // --- Gradient Descent (backtracking line search) ---
    //
    // --- Newton's Method ---
    // Converged in 21 iterations: f = 0.000000, x = Vector { data: [0.9999999999400667, 0.9999999998789006] }
    //
    // --- Finite-Difference Newton (no analytic Hessian) ---
    // Converged in 21 iterations: f = 0.000000, x = Vector { data: [0.9999999997345376, 0.9999999994678155] }
    //
    // --- Black-box optimization (finite-diff gradient only) ---
    //   Start 1: Vector { data: [5.0, 5.0] } -> x = Vector { data: [2.9999999999998823, 1.9999999999998368] }, f = 0.000000
    //   Start 2: Vector { data: [-5.0, 5.0] } -> x = Vector { data: [-2.805118086952577, 3.131312518250415] }, f = 0.000000
    //   Start 3: Vector { data: [5.0, -5.0] } -> x = Vector { data: [3.584428340330345, -1.8481265269642182] }, f = 0.000000

    Ok(())
}
