//! Interpolation and Approximation
//!
//! Demonstrates polynomial interpolation, spline interpolation,
//! and radial basis function (RBF) interpolation for scattered data.

use cnvx::math::random::{Distribution, Rng, Uniform};
use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    println!("=== Interpolation Examples ===\n");

    // --- Example 1: Polynomial Interpolation (Vandermonde) ---
    // Runge's function: f(x) = 1/(1+25*x^2) on [-1,1]
    println!("--- Polynomial Interpolation (Runge's Function) ---");
    let runge = |x: f64| 1.0 / (1.0 + 25.0 * x * x);

    for &n in &[5, 10, 15, 20] {
        // Equispaced nodes
        let x_nodes: Vector = (0..=n).map(|i| -1.0 + 2.0 * i as f64 / n as f64).collect();
        let y_nodes: Vector = x_nodes.iter().map(|&x| runge(x)).collect();

        // Build Vandermonde matrix
        let mut v = Matrix::zeros(n + 1, n + 1);
        for i in 0..=n {
            let mut x_pow = 1.0;
            for j in 0..=n {
                v.set(i, j, x_pow);
                x_pow *= x_nodes[i];
            }
        }

        // Solve for coefficients
        let coeffs = v.solve(&y_nodes)?;

        // Evaluate error on fine grid
        let n_test = 200;
        let mut max_err = 0.0_f64;
        for i in 0..=n_test {
            let x = -1.0 + 2.0 * i as f64 / n_test as f64;
            let mut y_interp = 0.0;
            let mut x_pow = 1.0;
            for j in 0..=n {
                y_interp += coeffs[j] * x_pow;
                x_pow *= x;
            }
            let err = (y_interp - runge(x)).abs();
            max_err = max_err.max(err);
        }

        println!("  n={n:2}: max error = {max_err:.2e}");
    }

    // --- Example 2: Chebyshev Nodes (avoids Runge phenomenon) ---
    println!("\n--- Chebyshev Nodes ---");
    for &n in &[5, 10, 15, 20] {
        // Chebyshev nodes of the second kind
        let x_nodes: Vector = (0..=n)
            .map(|i| (std::f64::consts::PI * i as f64 / n as f64).cos())
            .collect();
        let y_nodes: Vector = x_nodes.iter().map(|&x| runge(x)).collect();

        let mut v = Matrix::zeros(n + 1, n + 1);
        for i in 0..=n {
            let mut x_pow = 1.0;
            for j in 0..=n {
                v.set(i, j, x_pow);
                x_pow *= x_nodes[i];
            }
        }

        let coeffs = v.solve(&y_nodes)?;

        let n_test = 200;
        let mut max_err = 0.0_f64;
        for i in 0..=n_test {
            let x = -1.0 + 2.0 * i as f64 / n_test as f64;
            let mut y_interp = 0.0;
            let mut x_pow = 1.0;
            for j in 0..=n {
                y_interp += coeffs[j] * x_pow;
                x_pow *= x;
            }
            let err = (y_interp - runge(x)).abs();
            max_err = max_err.max(err);
        }

        println!("  n={n:2}: max error = {max_err:.2e}");
    }

    // --- Example 3: Cubic Spline Interpolation ---
    println!("\n--- Cubic Spline Interpolation ---");
    // Natural cubic spline: S''(x₀) = S''(xₙ) = 0
    let x_data = Vector::from_slice(&[0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0]);
    let y_data = Vector::from_slice(&[0.0, 0.5, 1.0, 0.5, 0.0, -0.5, 0.0]);
    let n = x_data.len() - 1; // 6 intervals

    // Build tridiagonal system for second derivatives
    // h[i] = x[i+1] - x[i]
    let h: Vec<f64> = (0..n).map(|i| x_data[i + 1] - x_data[i]).collect();

    // System: h[i-1]M[i-1] + 2(h[i-1]+h[i])M[i] + h[i]M[i+1] = 6((y[i+1]-y[i])/h[i]
    // - (y[i]-y[i-1])/h[i-1])
    let mut a = Matrix::zeros(n + 1, n + 1);
    let mut b = Vector::zeros(n + 1);

    // Natural spline: M[0] = M[n] = 0
    a.set(0, 0, 1.0);
    a.set(n, n, 1.0);
    b[0] = 0.0;
    b[n] = 0.0;

    for i in 1..n {
        a.set(i, i - 1, h[i - 1]);
        a.set(i, i, 2.0 * (h[i - 1] + h[i]));
        a.set(i, i + 1, h[i]);
        let dy1 = (y_data[i + 1] - y_data[i]) / h[i];
        let dy0 = (y_data[i] - y_data[i - 1]) / h[i - 1];
        b[i] = 6.0 * (dy1 - dy0);
    }

    let m = a.solve(&b)?; // Second derivatives at nodes

    // Evaluate spline
    let n_test = 100;
    let mut max_err = 0.0_f64;
    for i in 0..=n_test {
        let x = 3.0 * i as f64 / n_test as f64;
        // Find interval
        let mut idx = 0;
        while idx < n && x > x_data[idx + 1] {
            idx += 1;
        }
        idx = idx.min(n - 1);

        let xi = x_data[idx];
        let xip1 = x_data[idx + 1];
        let hi = xip1 - xi;
        let t = (x - xi) / hi;

        // Cubic spline formula
        let yi = y_data[idx];
        let yip1 = y_data[idx + 1];
        let mi = m[idx];
        let mip1 = m[idx + 1];

        let y_spline = (1.0 - t) * yi
            + t * yip1
            + hi * hi / 6.0 * ((1.0 - t).powi(3) - (1.0 - t) * mi + t.powi(3) - t * mip1);

        // True function (sin(pi*x) for comparison)
        let y_true = (std::f64::consts::PI * x).sin();
        let err = (y_spline - y_true).abs();
        max_err = max_err.max(err);
    }

    println!("  Max error vs sin(pi*x): {max_err:.2e}");

    // --- Example 4: Radial Basis Function (RBF) Interpolation ---
    println!("\n--- RBF Interpolation (Multiquadric) ---");
    // Scattered 2D data: f(x,y) = sin(x)cos(y)
    let n_centers = 20;
    let mut rng = Rng::new(42);
    let uniform = Uniform::new(-2.0, 2.0).unwrap();

    let centers: Vec<Vector> = (0..n_centers)
        .map(|_| {
            Vector::from_slice(&[uniform.sample(&mut rng), uniform.sample(&mut rng)])
        })
        .collect();

    let values: Vector = centers.iter().map(|c| (c[0]).sin() * (c[1]).cos()).collect();

    // Multiquadric RBF: phi(r) = sqrt(r^2 + c^2)
    let c = 0.5; // Shape parameter

    // Build interpolation matrix
    let mut a = Matrix::zeros(n_centers, n_centers);
    for (i, center_i) in centers.iter().enumerate() {
        for (j, center_j) in centers.iter().enumerate() {
            let dx = center_i[0] - center_j[0];
            let dy = center_i[1] - center_j[1];
            let r2 = dx * dx + dy * dy;
            a.set(i, j, (r2 + c * c).sqrt());
        }
    }

    // Add polynomial reproduction (linear: 1, x, y)
    let mut a_aug = Matrix::zeros(n_centers + 3, n_centers + 3);
    for (i, center_i) in centers.iter().enumerate().take(n_centers) {
        for (j, _center_j) in centers.iter().enumerate().take(n_centers) {
            a_aug.set(i, j, a.get(i, j));
        }
        a_aug.set(i, n_centers, 1.0);
        a_aug.set(i, n_centers + 1, center_i[0]);
        a_aug.set(i, n_centers + 2, center_i[1]);

        a_aug.set(n_centers, i, 1.0);
        a_aug.set(n_centers + 1, i, center_i[0]);
        a_aug.set(n_centers + 2, i, center_i[1]);
    }

    let mut b_aug = Vector::zeros(n_centers + 3);
    for i in 0..n_centers {
        b_aug[i] = values[i];
    }
    // Last 3 entries are 0 (orthogonality conditions)

    let sol = a_aug.solve(&b_aug)?;

    // Test on grid
    let n_test = 100;
    let mut max_err = 0.0_f64;
    for i in 0..n_test {
        for j in 0..n_test {
            let x = -2.0 + 4.0 * i as f64 / (n_test - 1) as f64;
            let y = -2.0 + 4.0 * j as f64 / (n_test - 1) as f64;

            let mut y_interp =
                sol[n_centers] + sol[n_centers + 1] * x + sol[n_centers + 2] * y;
            for k in 0..n_centers {
                let dx = x - centers[k][0];
                let dy = y - centers[k][1];
                let r2 = dx * dx + dy * dy;
                y_interp += sol[k] * (r2 + c * c).sqrt();
            }

            let y_true = x.sin() * y.cos();
            let err = (y_interp - y_true).abs();
            max_err = max_err.max(err);
        }
    }

    println!("  Max error on 100x100 grid: {max_err:.4}");

    // --- Example 5: Least Squares Approximation (not interpolation) ---
    println!("\n--- Least Squares Polynomial Fit (degree 5) ---");
    // Fit degree 5 polynomial to Runge function with many points
    let n_fit = 50;
    let x_fit: Vector = (0..n_fit)
        .map(|i| -1.0 + 2.0 * i as f64 / (n_fit - 1) as f64)
        .collect();
    let y_fit: Vector = x_fit.iter().map(|&x| runge(x)).collect();

    let degree = 5;
    let mut v = Matrix::zeros(n_fit, degree + 1);
    for i in 0..n_fit {
        let mut x_pow = 1.0;
        for j in 0..=degree {
            v.set(i, j, x_pow);
            x_pow *= x_fit[i];
        }
    }

    let coeffs = v.least_squares(&y_fit)?;

    let n_test = 200;
    let mut max_err = 0.0_f64;
    for i in 0..=n_test {
        let x = -1.0 + 2.0 * i as f64 / n_test as f64;
        let mut y_approx = 0.0;
        let mut x_pow = 1.0;
        for j in 0..=degree {
            y_approx += coeffs[j] * x_pow;
            x_pow *= x;
        }
        let err = (y_approx - runge(x)).abs();
        max_err = max_err.max(err);
    }

    println!("  Degree {degree} LS fit max error: {max_err:.2e}");

    // Expected output:
    //
    // === Interpolation Examples ===
    //
    // --- Polynomial Interpolation (Runge's Function) ---
    //   n= 5: max error = 4.33e-1
    //   n=10: max error = 1.92e0
    //   n=15: max error = 2.10e0
    //   n=20: max error = 5.86e1
    //
    // --- Chebyshev Nodes ---
    //   n= 5: max error = 6.39e-1
    //   n=10: max error = 1.32e-1
    //   n=15: max error = 9.93e-2
    //   n=20: max error = 1.77e-2
    //
    // --- Cubic Spline Interpolation ---
    //   Max error vs sin(pi*x): 1.72e0
    //
    // --- RBF Interpolation (Multiquadric) ---
    //   Max error on 100x100 grid: 1.0285
    //
    // --- Least Squares Polynomial Fit (degree 5) ---
    //   Degree 5 LS fit max error: 3.37e-1

    Ok(())
}
