/// The natural logarithm of the gamma function, via the Lanczos
/// approximation.
pub fn ln_gamma(x: f64) -> f64 {
    const G: f64 = 7.0;
    const COEFFS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];

    if x < 0.5 {
        // Reflection formula: Gamma(x) Gamma(1-x) = pi / sin(pi x).
        let pi = std::f64::consts::PI;
        (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x)
    } else {
        let x = x - 1.0;
        let mut a = COEFFS[0];
        let t = x + G + 0.5;
        for (i, &c) in COEFFS.iter().enumerate().skip(1) {
            a += c / (x + i as f64);
        }
        0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
    }
}

/// The gamma function `Gamma(x) = integral(t^(x-1) e^(-t), t = 0..inf)`.
///
/// Overflows to `f64::INFINITY` for `x` roughly greater than 171; use
/// [`ln_gamma`] instead if only the (much wider-ranging) logarithm is
/// needed, e.g. when normalizing a Gamma or Beta distribution's density.
pub fn gamma(x: f64) -> f64 {
    if x < 0.5 {
        let pi = std::f64::consts::PI;
        pi / ((pi * x).sin() * gamma(1.0 - x))
    } else {
        ln_gamma(x).exp()
    }
}

/// The natural logarithm of the beta function:
/// `ln(Beta(a, b)) = ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)`.
pub fn ln_beta(a: f64, b: f64) -> f64 {
    ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)
}

/// The beta function `Beta(a, b) = Gamma(a) Gamma(b) / Gamma(a + b)`.
pub fn beta(a: f64, b: f64) -> f64 {
    ln_beta(a, b).exp()
}

/// The error function: `erf(x) = (2/sqrt(pi)) * integral(e^(-t^2), t = 0..x)`.
///
/// Uses the Abramowitz & Stegun 7.1.26 rational approximation (max error
/// ~1.5e-7), which is accurate enough for this crate's use (Normal
/// distribution CDFs) without pulling in a dedicated numerics dependency.
pub fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    const A1: f64 = 0.254_829_592;
    const A2: f64 = -0.284_496_736;
    const A3: f64 = 1.421_413_741;
    const A4: f64 = -1.453_152_027;
    const A5: f64 = 1.061_405_429;
    const P: f64 = 0.327_591_1;

    let t = 1.0 / (1.0 + P * x);
    let poly = ((((A5 * t + A4) * t + A3) * t + A2) * t + A1) * t;
    sign * (1.0 - poly * (-x * x).exp())
}

/// The complementary error function: `erfc(x) = 1 - erf(x)`.
pub fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_matches_factorials() {
        // Gamma(n) = (n - 1)! for positive integers n.
        assert!((gamma(1.0) - 1.0).abs() < 1e-9);
        assert!((gamma(2.0) - 1.0).abs() < 1e-9);
        assert!((gamma(5.0) - 24.0).abs() < 1e-7);
        assert!((gamma(10.0) - 362_880.0).abs() < 1e-3);
    }

    #[test]
    fn gamma_half() {
        // Gamma(1/2) = sqrt(pi).
        assert!((gamma(0.5) - std::f64::consts::PI.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn ln_gamma_matches_ln_of_gamma_for_moderate_x() {
        for x in [1.5, 3.0, 7.25, 20.0] {
            assert!((ln_gamma(x) - gamma(x).ln()).abs() < 1e-8);
        }
    }

    #[test]
    fn beta_matches_gamma_identity() {
        let a = 2.5;
        let b = 3.5;
        let expected = gamma(a) * gamma(b) / gamma(a + b);
        assert!((beta(a, b) - expected).abs() < 1e-9);
    }

    #[test]
    fn erf_known_values() {
        // The Abramowitz & Stegun 7.1.26 approximation this is built on
        // has a documented max error of about 1.5e-7, so exact-zero
        // isn't expected at x = 0, just "close".
        assert!((erf(0.0)).abs() < 2e-7);
        assert!((erf(1.0) - 0.842_700_79).abs() < 1e-6);
        assert!((erf(-1.0) + 0.842_700_79).abs() < 1e-6);
        assert!((erfc(0.0) - 1.0).abs() < 2e-7);
    }
}
