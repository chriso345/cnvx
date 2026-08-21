//! The shared elementwise ("ufunc") function-generating macro.
//!
//! Every type this macro is used on must already provide an inherent
//! `map(&self, f: impl Fn(f64) -> f64) -> Self` method.
macro_rules! elementwise_fn {
    ($name:ident, $f64_method:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(&self) -> Self {
            self.map(|x| x.$f64_method())
        }
    };
}

/// Generates the standard elementary-function set (`sin`, `cos`, `tan`,
/// `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `exp`, `ln`, `log10`,
/// `log2`, `sqrt`, `cbrt`, `abs`, `floor`, `ceil`, `round`, `recip`) as
/// inherent methods on `Self`, plus `powi`/`powf`/`clamp`. Requires `Self`
/// to already provide `map`.
macro_rules! elementwise_fn_set {
    () => {
        crate::elementwise::elementwise_fn!(sin, sin, "Elementwise sine.");
        crate::elementwise::elementwise_fn!(cos, cos, "Elementwise cosine.");
        crate::elementwise::elementwise_fn!(tan, tan, "Elementwise tangent.");
        crate::elementwise::elementwise_fn!(asin, asin, "Elementwise arcsine.");
        crate::elementwise::elementwise_fn!(acos, acos, "Elementwise arccosine.");
        crate::elementwise::elementwise_fn!(atan, atan, "Elementwise arctangent.");
        crate::elementwise::elementwise_fn!(sinh, sinh, "Elementwise hyperbolic sine.");
        crate::elementwise::elementwise_fn!(cosh, cosh, "Elementwise hyperbolic cosine.");
        crate::elementwise::elementwise_fn!(
            tanh,
            tanh,
            "Elementwise hyperbolic tangent."
        );
        crate::elementwise::elementwise_fn!(exp, exp, "Elementwise natural exponential.");
        crate::elementwise::elementwise_fn!(ln, ln, "Elementwise natural logarithm.");
        crate::elementwise::elementwise_fn!(
            log10,
            log10,
            "Elementwise base-10 logarithm."
        );
        crate::elementwise::elementwise_fn!(log2, log2, "Elementwise base-2 logarithm.");
        crate::elementwise::elementwise_fn!(sqrt, sqrt, "Elementwise square root.");
        crate::elementwise::elementwise_fn!(cbrt, cbrt, "Elementwise cube root.");
        crate::elementwise::elementwise_fn!(abs, abs, "Elementwise absolute value.");
        crate::elementwise::elementwise_fn!(floor, floor, "Elementwise floor.");
        crate::elementwise::elementwise_fn!(ceil, ceil, "Elementwise ceiling.");
        crate::elementwise::elementwise_fn!(
            round,
            round,
            "Elementwise rounding to the nearest integer."
        );
        crate::elementwise::elementwise_fn!(
            recip,
            recip,
            "Elementwise reciprocal (`1 / x`)."
        );

        /// Raises every element to the integer power `n`.
        pub fn powi(&self, n: i32) -> Self {
            self.map(|x| x.powi(n))
        }

        /// Raises every element to the floating-point power `p`.
        pub fn powf(&self, p: f64) -> Self {
            self.map(|x| x.powf(p))
        }

        /// Clamps every element to the inclusive range `[lo, hi]`.
        pub fn clamp(&self, lo: f64, hi: f64) -> Self {
            self.map(|x| x.clamp(lo, hi))
        }
    };
}

pub(crate) use elementwise_fn;
pub(crate) use elementwise_fn_set;
