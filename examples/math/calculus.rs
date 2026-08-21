//! Numerical Calculus
//!
//! Demonstrates finite-difference differentiation, root-finding, and
//! quadrature on a handful of classic test functions:
//! - Gradient and Hessian of the Rosenbrock function
//! - Root of cos(t) - t = 0, via bisection and Newton's method
//! - Definite integral of sin(t) e^t via Simpson's rule

use cnvx_math::Vector;
use cnvx_math::calculus::{DEFAULT_STEP, bisection, gradient, hessian, newton, simpson};

fn main() {
    // Finite-difference gradient and Hessian of a scalar function. This
    // is useful when an optimizer only has a black-box objective and no
    // hand-derived derivatives to work with.
    let rosenbrock = |point: &Vector| {
        (1.0 - point[0]).powi(2) + 100.0 * (point[1] - point[0].powi(2)).powi(2)
    };
    let eval_point = Vector::from_slice(&[0.5, 0.5]);

    let grad = gradient(rosenbrock, &eval_point, DEFAULT_STEP);
    println!("gradient of Rosenbrock at {eval_point:?}: {grad:?}");

    let hess = hessian(rosenbrock, &eval_point, 1e-4);
    println!("Hessian at {eval_point:?}:\n{hess:?}");

    // Root-finding: bisection always converges given a bracket, while
    // Newton's method converges faster near a simple root but needs a
    // derivative.
    let dottie_eq = |t: f64| t.cos() - t;
    let dottie_eq_prime = |t: f64| -t.sin() - 1.0;

    let root_via_bisection = bisection(dottie_eq, 0.0, 1.0, 1e-12, 100)
        .expect("dottie_eq changes sign on [0, 1]");
    let root_via_newton = newton(dottie_eq, dottie_eq_prime, 0.5, 1e-14, 50)
        .expect("Newton's method converges from t0 = 0.5");

    println!("\nroot of cos(t) - t = 0:");
    println!("  bisection: {root_via_bisection:.12}");
    println!("  Newton's method: {root_via_newton:.12}");

    // Quadrature: integrate a smooth function via Simpson's rule.
    let integral = simpson(|t: f64| t.sin() * t.exp(), 0.0, std::f64::consts::PI, 1000)
        .expect("n = 1000 is even and positive");
    println!("\nintegral of sin(t) e^t over [0, pi] = {integral:.6}");

    // Expected output:
    // (some values below are rounded to 3-4 decimal places for readability)
    //
    // gradient of Rosenbrock at Vector { data: [0.5, 0.5] }: Vector { data:
    // [-51.000, 50.000] }
    //
    // Hessian at [0.5, 0.5]:
    // [[102.000, -200.000], [-200.000, 200.000]]
    //
    // root of cos(t) - t = 0:
    //   bisection: 0.739085133215
    //   Newton's method: 0.739085133215
    //
    // integral of sin(t) e^t over [0, pi] = 12.070346
}
