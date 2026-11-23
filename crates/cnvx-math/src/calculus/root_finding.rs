use crate::error::MathError;

// TODO: Add more root-finding methods (secant, regula falsi, Brent's method,
// etc.) and a generic solver that can take a `Method` enum to choose between
// them.

/// Finds a root of `f` in `[a, b]` via bisection.
///
/// Requires `f(a)` and `f(b)` to have opposite signs (the standard
/// bracketing precondition); halves the bracket every iteration until its
/// width is below `tol` or `max_iter` is reached.
///
/// Bisection is slower per-iteration progress than [`newton`] (linear
/// rather than quadratic convergence) but only needs `f`, not `f'`, and
/// is guaranteed to converge given a valid bracket.
///
/// # Errors
/// Returns [`MathError::InvalidArgument`] if `f(a)` and `f(b)` don't have
/// opposite signs, and [`MathError::NotConverged`] if `max_iter` is
/// reached before the bracket shrinks below `tol`.
pub fn bisection(
    f: impl Fn(f64) -> f64,
    mut a: f64,
    mut b: f64,
    tol: f64,
    max_iter: u32,
) -> Result<f64, MathError> {
    let mut fa = f(a);
    let fb = f(b);
    if fa == 0.0 {
        return Ok(a);
    }
    if fb == 0.0 {
        return Ok(b);
    }
    if fa.signum() == fb.signum() {
        return Err(MathError::InvalidArgument(
            "bisection: f(a) and f(b) must have opposite signs".into(),
        ));
    }

    for _ in 0..max_iter {
        let mid = 0.5 * (a + b);
        let fmid = f(mid);
        if fmid == 0.0 || (b - a) / 2.0 < tol {
            return Ok(mid);
        }
        if fmid.signum() == fa.signum() {
            a = mid;
            fa = fmid;
        } else {
            b = mid;
        }
    }
    Err(MathError::NotConverged(max_iter))
}

/// Finds a root of `f` near `x0` via Newton's method:
/// `x_(k+1) = x_k - f(x_k) / f'(x_k)`.
///
/// Converges quadratically near a simple root when it converges at all,
/// but (unlike [`bisection`]) has no global convergence guarantee: a poor
/// `x0`, a vanishing derivative, or a non-simple root can all cause it to
/// diverge or cycle, which is why the well-tested workhorse remains
/// bisection when the extra robustness is worth the slower convergence.
///
/// # Errors
/// Returns [`MathError::NotConverged`] if `max_iter` is reached before
/// `|f(x)| < tol`, and [`MathError::Singular`] if `f'(x)` vanishes during
/// the iteration.
pub fn newton(
    f: impl Fn(f64) -> f64,
    f_prime: impl Fn(f64) -> f64,
    x0: f64,
    tol: f64,
    max_iter: u32,
) -> Result<f64, MathError> {
    let mut x = x0;
    for _ in 0..max_iter {
        let fx = f(x);
        if fx.abs() < tol {
            return Ok(x);
        }
        let fpx = f_prime(x);
        if fpx.abs() < 1e-300 {
            return Err(MathError::Singular);
        }
        x -= fx / fpx;
    }
    Err(MathError::NotConverged(max_iter))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bisection_finds_sqrt_two() {
        let root = bisection(|x| x * x - 2.0, 0.0, 2.0, 1e-10, 100).unwrap();
        assert!((root - std::f64::consts::SQRT_2).abs() < 1e-8);
    }

    #[test]
    fn bisection_rejects_bad_bracket() {
        assert!(bisection(|x| x * x + 1.0, -1.0, 1.0, 1e-10, 100).is_err());
    }

    #[test]
    fn newton_finds_sqrt_two() {
        let root = newton(|x| x * x - 2.0, |x| 2.0 * x, 1.0, 1e-12, 50).unwrap();
        assert!((root - std::f64::consts::SQRT_2).abs() < 1e-9);
    }

    #[test]
    fn newton_reports_non_convergence() {
        // f(x) = x^3 - 2x + 2 famously cycles between 0 and 1 for Newton's
        // method starting at x0 = 0.
        let result = newton(
            |x| x.powi(3) - 2.0 * x + 2.0,
            |x| 3.0 * x.powi(2) - 2.0,
            0.0,
            1e-12,
            20,
        );
        assert!(result.is_err());
    }
}
