use cnvx_math::Vector;
use cnvx_math::calculus::{
    DEFAULT_STEP, bisection, gradient, hessian, jacobian, newton, simpson, trapezoidal,
};

#[test]
fn gradient_matches_analytic_gradient_of_rosenbrock_at_minimum() {
    // The Rosenbrock function has a global minimum (zero gradient) at (1, 1).
    let f = |x: &Vector| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0].powi(2)).powi(2);
    let x = Vector::from_slice(&[1.0, 1.0]);
    let g = gradient(f, &x, DEFAULT_STEP);
    assert!(g[0].abs() < 1e-3);
    assert!(g[1].abs() < 1e-3);
}

#[test]
fn hessian_is_symmetric() {
    let f = |x: &Vector| x[0].powi(3) * x[1] + x[1].powi(2);
    let x = Vector::from_slice(&[1.5, 2.0]);
    let h = hessian(f, &x, 1e-4);
    assert!((h.get(0, 1) - h.get(1, 0)).abs() < 1e-6);
}

#[test]
fn jacobian_of_polar_to_cartesian() {
    // f(r, theta) = (r cos(theta), r sin(theta))
    let f = |x: &Vector| Vector::from_slice(&[x[0] * x[1].cos(), x[0] * x[1].sin()]);
    let x = Vector::from_slice(&[2.0, 0.0]);
    let j = jacobian(f, &x, DEFAULT_STEP);
    // At theta = 0: d(x)/dr = cos(0) = 1, d(x)/dtheta = -r sin(0) = 0
    //               d(y)/dr = sin(0) = 0, d(y)/dtheta = r cos(0) = 2
    assert!((j.get(0, 0) - 1.0).abs() < 1e-4);
    assert!((j.get(0, 1) - 0.0).abs() < 1e-4);
    assert!((j.get(1, 0) - 0.0).abs() < 1e-4);
    assert!((j.get(1, 1) - 2.0).abs() < 1e-4);
}

#[test]
fn root_finders_agree_on_a_shared_root() {
    let f = |x: f64| x.powi(3) - x - 2.0;
    let f_prime = |x: f64| 3.0 * x.powi(2) - 1.0;
    let bisected = bisection(f, 1.0, 2.0, 1e-10, 100).unwrap();
    let newtons = newton(f, f_prime, 1.5, 1e-12, 50).unwrap();
    assert!((bisected - newtons).abs() < 1e-6);
}

#[test]
fn quadrature_methods_agree_closely_for_smooth_functions() {
    let f = |x: f64| (-x * x).exp();
    let trap = trapezoidal(f, -3.0, 3.0, 1000).unwrap();
    let simp = simpson(f, -3.0, 3.0, 1000).unwrap();
    // The true value on this finite interval is sqrt(pi) * erf(3), not
    // quite sqrt(pi) (the improper integral over all reals) since a
    // vanishingly small tail beyond +-3 is excluded.
    let expected = std::f64::consts::PI.sqrt() * cnvx_math::special::erf(3.0);
    assert!((trap - expected).abs() < 1e-3);
    assert!((simp - expected).abs() < 1e-6);
}
