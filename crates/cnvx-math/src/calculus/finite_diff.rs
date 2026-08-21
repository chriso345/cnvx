use crate::matrix::Matrix;
use crate::vector::Vector;

/// The default step size used when the caller doesn't specify one: a
/// standard choice balancing truncation error (which shrinks with `h`)
/// against floating-point cancellation error (which grows as `h` shrinks
/// below `f64` precision).
pub const DEFAULT_STEP: f64 = 1e-6;

/// Approximates the gradient of a scalar-valued function `f: R^n -> R` at
/// `x` via central differences: `df/dx_i ~= (f(x + h e_i) - f(x - h e_i)) /
/// (2h)`.
pub fn gradient(f: impl Fn(&Vector) -> f64, x: &Vector, h: f64) -> Vector {
    let n = x.len();
    let mut grad = vec![0.0; n];
    for i in 0..n {
        let mut x_plus = x.clone();
        x_plus[i] += h;
        let mut x_minus = x.clone();
        x_minus[i] -= h;
        grad[i] = (f(&x_plus) - f(&x_minus)) / (2.0 * h);
    }
    Vector::from(grad)
}

/// Approximates the Jacobian of a vector-valued function `f: R^n -> R^m`
/// at `x` via central differences. The returned matrix is `m x n`, with
/// row `j`, column `i` equal to `d f_j / d x_i`.
///
/// # Panics
/// Panics if `f(x)` returns an empty vector.
pub fn jacobian(f: impl Fn(&Vector) -> Vector, x: &Vector, h: f64) -> Matrix {
    let n = x.len();
    let m = f(x).len();
    assert!(m > 0, "calculus::jacobian: f(x) must be non-empty");
    let mut jac = Matrix::zeros(m, n);
    for i in 0..n {
        let mut x_plus = x.clone();
        x_plus[i] += h;
        let mut x_minus = x.clone();
        x_minus[i] -= h;
        let f_plus = f(&x_plus);
        let f_minus = f(&x_minus);
        for j in 0..m {
            jac.set(j, i, (f_plus[j] - f_minus[j]) / (2.0 * h));
        }
    }
    jac
}

/// Approximates the Hessian of a scalar-valued function `f: R^n -> R` at
/// `x` via central second differences:
/// `d^2f/dx_i dx_j ~= (f(x+h e_i+h e_j) - f(x+h e_i-h e_j) - f(x-h e_i+h e_j) +
/// f(x-h e_i-h e_j)) / (4h^2)`.
///
/// The result is symmetric by construction (mixed partials are computed
/// once and mirrored), even though `2n^2` function evaluations are used
/// to get there.
pub fn hessian(f: impl Fn(&Vector) -> f64, x: &Vector, h: f64) -> Matrix {
    let n = x.len();
    let mut hess = Matrix::zeros(n, n);
    for i in 0..n {
        for j in i..n {
            let mut pp = x.clone();
            pp[i] += h;
            pp[j] += h;
            let mut pm = x.clone();
            pm[i] += h;
            pm[j] -= h;
            let mut mp = x.clone();
            mp[i] -= h;
            mp[j] += h;
            let mut mm = x.clone();
            mm[i] -= h;
            mm[j] -= h;
            let value = (f(&pp) - f(&pm) - f(&mp) + f(&mm)) / (4.0 * h * h);
            hess.set(i, j, value);
            hess.set(j, i, value);
        }
    }
    hess
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_of_quadratic() {
        // f(x) = x0^2 + 2 x1^2 => grad = [2 x0, 4 x1]
        let f = |x: &Vector| x[0] * x[0] + 2.0 * x[1] * x[1];
        let x = Vector::from_slice(&[3.0, -1.0]);
        let g = gradient(f, &x, DEFAULT_STEP);
        assert!((g[0] - 6.0).abs() < 1e-4);
        assert!((g[1] + 4.0).abs() < 1e-4);
    }

    #[test]
    fn jacobian_of_linear_map() {
        // f(x) = [2 x0 + x1, x0 - 3 x1] => J = [[2, 1], [1, -3]]
        let f = |x: &Vector| Vector::from_slice(&[2.0 * x[0] + x[1], x[0] - 3.0 * x[1]]);
        let x = Vector::from_slice(&[1.0, 1.0]);
        let j = jacobian(f, &x, DEFAULT_STEP);
        assert!((j.get(0, 0) - 2.0).abs() < 1e-4);
        assert!((j.get(0, 1) - 1.0).abs() < 1e-4);
        assert!((j.get(1, 0) - 1.0).abs() < 1e-4);
        assert!((j.get(1, 1) + 3.0).abs() < 1e-4);
    }

    #[test]
    fn hessian_of_quadratic() {
        // f(x) = x0^2 + x0 x1 + 2 x1^2
        // grad = [2x0 + x1, x0 + 4x1], Hessian = [[2, 1], [1, 4]]
        let f = |x: &Vector| x[0] * x[0] + x[0] * x[1] + 2.0 * x[1] * x[1];
        let x = Vector::from_slice(&[1.0, 1.0]);
        let h = hessian(f, &x, 1e-4);
        assert!((h.get(0, 0) - 2.0).abs() < 1e-2);
        assert!((h.get(0, 1) - 1.0).abs() < 1e-2);
        assert!((h.get(1, 0) - 1.0).abs() < 1e-2);
        assert!((h.get(1, 1) - 4.0).abs() < 1e-2);
    }
}
