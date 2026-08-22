//! Nonlinear System Solving
//!
//! Solves systems of nonlinear equations using Newton's method
//! with finite-difference Jacobians, applied to practical problems.

use cnvx::math::calculus::*;
use cnvx::prelude::*;

#[rustfmt::skip]
fn main() -> Result<(), cnvx_core::CnvxError> {
    println!("=== Nonlinear System Solver ===\n");

    // --- Example 1: Intersection of Circle and Parabola ---
    // x^2 + y^2 = 1 (unit circle)
    // y = x^2 - 0.5 (parabola)
    // Solutions: substitute y: x^2 + (x^2 - 0.5)^2 = 1
    // x^2 + x^4 - x^2 + 0.25 = 1 => x^4 + 0.25 = 1 => x^4 = 0.75 => x =
    // +/-0.75^(1/4) x ~= +/-0.9306, y = x^2 - 0.5 ~= 0.366

    println!("--- Circle & Parabola Intersection ---");
    let f1 = |v: &Vector| -> Vector {
        let x = v[0];
        let y = v[1];
        Vector::from_slice(&[x * x + y * y - 1.0, y - x * x + 0.5])
    };

    // Test multiple starting points to find all solutions
    let starts = [
        Vector::from_slice(&[0.5, 0.0]),
        Vector::from_slice(&[-0.5, 0.0]),
        Vector::from_slice(&[0.0, 1.0]),
        Vector::from_slice(&[0.0, -1.0]),
        Vector::from_slice(&[1.0, 1.0]),
        Vector::from_slice(&[-1.0, -1.0]),
    ];

    let mut solutions = Vec::new();
    for (i, start) in starts.iter().enumerate() {
        if let Ok(sol) = newton_system(f1, start, 1e-12, 50) {
            // Check if this solution is new (within tolerance)
            let is_new = solutions.iter().all(|s: &Vector| {
                (s[0] - sol[0]).abs() > 1e-6 || (s[1] - sol[1]).abs() > 1e-6
            });
            if is_new {
                solutions.push(sol.clone());
                println!("  Start {i}: {start:?} -> {sol:?}");
            }
        }
    }

    println!("  Found {} unique solutions:", solutions.len());
    for (i, sol) in solutions.iter().enumerate() {
        let residual = f1(sol).norm();
        println!(
            "    Solution {i}: x={:.6}, y={:.6}, |f|={:.2e}",
            sol[0], sol[1], residual
        );
    }

    // --- Example 2: Chemical Equilibrium ---
    // A + B <-> C,  K1 = 10
    // C + B <-> D,  K2 = 5
    // Initial: A=1, B=1, C=0, D=0
    // Let x = amount of first reaction, y = amount of second
    // A = 1-x, B = 1-x-y, C = x-y, D = y
    // K1 = C/(A*B) = (x-y)/((1-x)*(1-x-y)) = 10
    // K2 = D/(C*B) = y/((x-y)*(1-x-y)) = 5

    println!("\n--- Chemical Equilibrium ---");
    let f2 = |v: &Vector| -> Vector {
        let x = v[0].max(1e-12);
        let y = v[1].max(1e-12);
        let a = 1.0 - x;
        let b = 1.0 - x - y;
        let c = x - y;
        let d = y;

        // Avoid negative concentrations
        if a <= 0.0 || b <= 0.0 || c <= 0.0 {
            return Vector::from_slice(&[1e6, 1e6]);
        }

        Vector::from_slice(&[c / (a * b) - 10.0, d / (c * b) - 5.0])
    };

    let start = Vector::from_slice(&[0.6, 0.25]);
    match newton_system(f2, &start, 1e-10, 200) {
        Ok(sol) => {
            let x = sol[0];
            let y = sol[1];
            let a = 1.0 - x;
            let b = 1.0 - x - y;
            let c = x - y;
            let d = y;
            println!("  Extent of reaction 1: x = {x:.6}");
            println!("  Extent of reaction 2: y = {y:.6}");
            println!("  Equilibrium concentrations:");
            println!("    [A] = {a:.6}, [B] = {b:.6}, [C] = {c:.6}, [D] = {d:.6}");
            println!("  Check: K1 = C/(A*B) = {:.3}", c / (a * b));
            println!("         K2 = D/(C*B) = {:.3}", d / (c * b));
        }
        Err(e) => {
            println!("  Newton failed: {e:?}");
        }
    };

    // --- Example 3: Combustion / Reactor Design ---
    // Stoichiometric combustion of methane: CH4 + 2O2 -> CO2 + 2H2O
    // With dissociation at high temperature:
    // CO2 <-> CO + 0.5 O2
    // H2O <-> H2 + 0.5 O2
    // H2O <-> OH + 0.5 H2
    // Solve for equilibrium composition

    println!("\n--- Combustion Equilibrium (simplified) ---");
    // Species: CH4, O2, CO2, H2O, CO, H2, OH (7 species)
    // Elements: C, H, O (3 elements)
    // Use element conservation + equilibrium constants

    // Simplified: just CO2 dissociation CO2 <-> CO + 0.5 O2
    // Kp = 0.1 at high temperature
    // Initial: 1 CO2, 0 CO, 0 O2
    // At eq: 1-x CO2, x CO, 0.5x O2
    // Kp = (x * (0.5x)^0.5) / (1-x) / P^0.5 = 0.1
    // Let total pressure P = 1 atm
    // x * sqrt(0.5x) / (1-x) = 0.1

    let f3 = |v: &Vector| -> Vector {
        let x = v[0].clamp(1e-12, 1.0 - 1e-12);
        let kp = 0.1;
        Vector::from_slice(&[x * (0.5 * x).sqrt() / (1.0 - x) - kp])
    };

    let start = Vector::from_slice(&[0.1]);
    if let Ok(sol) = newton(f3, &start, 1e-14, 50) {
        let x = sol[0];
        println!("  CO2 dissociation fraction: x = {x:.6}");
        println!("  Composition: CO2={:.4}, CO={:.4}, O2={:.4}", 1.0 - x, x, 0.5 * x);
    }

    // --- Example 4: Finite Difference Nonlinear BVP ---
    // u'' + u^2 = 0, u(0)=0, u(1)=1
    // Discretize with central differences

    println!("\n--- Nonlinear BVP: u'' + u^2 = 0 ---");
    let n = 50;
    let h = 1.0 / n as f64;

    let f_bvp = |u: &Vector| -> Vector {
        // u has interior points only (n-1 elements), boundaries are u(0)=0, u(1)=1
        let mut f = Vector::zeros(n - 1);
        for i in 1..n {
            let ui = u[i - 1]; // Interior index
            let uim1 = if i == 1 { 0.0 } else { u[i - 2] };
            let uip1 = if i == n - 1 { 1.0 } else { u[i] };
            // u'' ~ (u_{i-1} - 2u_i + u_{i+1}) / h^2
            let u_xx = (uim1 - 2.0 * ui + uip1) / (h * h);
            f[i - 1] = u_xx + ui * ui;
        }
        f
    };

    // Initial guess: linear
    let mut u0_interior = Vector::zeros(n - 1);
    for i in 1..n {
        u0_interior[i - 1] = i as f64 / n as f64;
    }

    if let Ok(sol_interior) = newton_system(f_bvp, &u0_interior, 1e-10, 100) {
        let mut sol = Vector::zeros(n + 1);
        sol[0] = 0.0;
        for i in 1..n {
            sol[i] = sol_interior[i - 1];
        }
        sol[n] = 1.0;

        println!("  Solved with Newton (interior points)");
        println!(
            "  u(0.25) = {:.6}, u(0.5) = {:.6}, u(0.75) = {:.6}",
            sol[n / 4],
            sol[n / 2],
            sol[3 * n / 4]
        );

        // Check residual
        let mut max_residual = 0.0_f64;
        for i in 1..n {
            let ui = sol[i];
            let uim1 = sol[i - 1];
            let uip1 = sol[i + 1];
            // u'' ~ (u_{i-1} - 2u_i + u_{i+1}) / h^2
            let u_xx = (uim1 - 2.0 * ui + uip1) / (h * h);
            let res = (u_xx + ui * ui).abs();
            max_residual = max_residual.max(res);
        }
        println!("  Max residual: {max_residual:.2e}");
    }

    // Expected output:
    //
    // === Nonlinear System Solver ===
    //
    // --- Circle & Parabola Intersection ---
    //   Start 0: Vector { data: [0.5, 0.0] } -> Vector { data: [0.9306048591020997, 0.36602540378443865] }
    //   Start 1: Vector { data: [-0.5, 0.0] } -> Vector { data: [-0.9306048591019687, 0.3660254037844393] }
    //   Found 2 unique solutions:
    //     Solution 0: x=0.930605, y=0.366025, |f|=3.14e-16
    //     Solution 1: x=-0.930605, y=0.366025, |f|=3.45e-13
    //
    // --- Chemical Equilibrium ---
    //   Extent of reaction 1: x = 0.646769
    //   Extent of reaction 2: y = 0.237314
    //   Equilibrium concentrations:
    //     [A] = 0.353231, [B] = 0.115917, [C] = 0.409455, [D] = 0.237314
    //   Check: K1 = C/(A*B) = 10.000
    //          K2 = D/(C*B) = 5.000
    //
    // --- Combustion Equilibrium (simplified) ---
    //   CO2 dissociation fraction: x = 0.228360
    //   Composition: CO2=0.7716, CO=0.2284, O2=0.1142
    //
    // --- Nonlinear BVP: u'' + u^2 = 0 ---
    //   Solved with Newton (interior points)
    //   u(0.25) = 0.262444, u(0.5) = 0.541251, u(0.75) = 0.780902
    //   Max residual: 4.07e-13

    Ok(())
}

// Newton's method for systems using finite-difference Jacobian
fn newton_system(
    f: impl Fn(&Vector) -> Vector,
    x0: &Vector,
    tol: f64,
    max_iter: usize,
) -> Result<Vector, cnvx_core::CnvxError> {
    let n = x0.len();
    let mut x = x0.clone();
    let mut f_val = f(&x);

    for _iter in 0..max_iter {
        if f_val.norm() < tol {
            return Ok(x);
        }

        // Finite-difference Jacobian
        let mut j = Matrix::zeros(n, n);
        let h = 1e-6;
        for j_col in 0..n {
            let mut x_plus = x.clone();
            x_plus[j_col] = x[j_col] + h;
            let f_plus = f(&x_plus);
            for i in 0..n {
                j.set(i, j_col, (f_plus[i] - f_val[i]) / h);
            }
        }

        // Solve J * dx = -f
        let neg_f = -f_val;
        let dx = j.solve(&neg_f)?;
        x = &x + &dx;
        f_val = f(&x);
    }

    Err(cnvx_core::CnvxError::Math(cnvx::math::MathError::Unsupported(
        "Newton did not converge".into(),
    )))
}

// Scalar Newton
fn newton(
    f: impl Fn(&Vector) -> Vector,
    x0: &Vector,
    tol: f64,
    max_iter: usize,
) -> Result<Vector, cnvx_core::CnvxError> {
    let mut x = x0.clone();

    for _ in 0..max_iter {
        let f_val = f(&x);
        if f_val.norm() < tol {
            return Ok(x);
        }
        let f_prime = gradient(|v: &Vector| f(v)[0], &x, 1e-6);
        let step = -f_val[0] / f_prime[0];
        x[0] += step;
    }

    Err(cnvx_core::CnvxError::Math(cnvx::math::MathError::Unsupported(
        "Newton did not converge".into(),
    )))
}
