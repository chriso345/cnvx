use crate::error::MathError;

// TODO: Add more quadrature methods (Gauss-Legendre, adaptive Simpson, etc.)
// and a generic integrator that can take a `Method` enum to choose between
// them.

/// Approximates `integral(f(x) dx, x = a..b)` via the composite
/// trapezoidal rule with `n` equal-width panels: `O(h^2)` accurate for
/// smooth `f`.
///
/// # Errors
/// Returns [`MathError::InvalidArgument`] if `n == 0`.
pub fn trapezoidal(
    f: impl Fn(f64) -> f64,
    a: f64,
    b: f64,
    n: u32,
) -> Result<f64, MathError> {
    if n == 0 {
        return Err(MathError::InvalidArgument("trapezoidal: n must be > 0".into()));
    }
    let h = (b - a) / n as f64;
    let mut sum = 0.5 * (f(a) + f(b));
    for i in 1..n {
        sum += f(a + i as f64 * h);
    }
    Ok(sum * h)
}

/// Approximates `integral(f(x) dx, x = a..b)` via the composite Simpson's
/// rule with `n` equal-width panels: `O(h^4)` accurate for smooth `f`,
/// substantially better than [`trapezoidal`] at the same panel count.
///
/// # Errors
/// Returns [`MathError::InvalidArgument`] if `n == 0` or `n` is odd
/// (Simpson's rule pairs panels, so it needs an even count).
pub fn simpson(f: impl Fn(f64) -> f64, a: f64, b: f64, n: u32) -> Result<f64, MathError> {
    if n == 0 || !n.is_multiple_of(2) {
        return Err(MathError::InvalidArgument(
            "simpson: n must be a positive even number".into(),
        ));
    }
    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);
    for i in 1..n {
        let x = a + i as f64 * h;
        sum += if i % 2 == 0 { 2.0 * f(x) } else { 4.0 * f(x) };
    }
    Ok(sum * h / 3.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trapezoidal_integrates_polynomial() {
        // integral(x^2, 0..3) = 9
        let result = trapezoidal(|x| x * x, 0.0, 3.0, 10_000).unwrap();
        assert!((result - 9.0).abs() < 1e-3);
    }

    #[test]
    fn simpson_is_exact_for_cubics() {
        // Simpson's rule is exact for polynomials up to degree 3.
        // integral(x^3 - 2x, 0..2) = 4 - 4 = 0
        let result = simpson(|x| x.powi(3) - 2.0 * x, 0.0, 2.0, 10).unwrap();
        assert!((result - 0.0).abs() < 1e-10);
    }

    #[test]
    fn simpson_integrates_sine() {
        // integral(sin(x), 0..pi) = 2
        let result = simpson(|x| x.sin(), 0.0, std::f64::consts::PI, 100).unwrap();
        assert!((result - 2.0).abs() < 1e-6);
    }

    #[test]
    fn simpson_rejects_odd_n() {
        assert!(simpson(|x| x, 0.0, 1.0, 3).is_err());
    }
}
