use super::Rng;
use crate::error::MathError;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// A probability distribution that can be sampled from.
pub trait Distribution {
    /// Draws one sample.
    fn sample(&self, rng: &mut Rng) -> f64;
}

/// The continuous uniform distribution on `[low, high)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uniform {
    pub low: f64,
    pub high: f64,
}

impl Uniform {
    /// Creates a `Uniform(low, high)` distribution.
    ///
    /// # Errors
    /// Returns [`MathError::InvalidDistributionParameter`] if `low >= high`.
    pub fn new(low: f64, high: f64) -> Result<Self, MathError> {
        if low >= high {
            return Err(MathError::InvalidDistributionParameter(format!(
                "Uniform: low ({low}) must be < high ({high})"
            )));
        }
        Ok(Self { low, high })
    }
}

impl Distribution for Uniform {
    fn sample(&self, rng: &mut Rng) -> f64 {
        self.low + (self.high - self.low) * rng.next_f64()
    }
}

/// The Gaussian (normal) distribution, parameterized by mean and standard
/// deviation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Normal {
    pub mean: f64,
    pub std_dev: f64,
}

impl Normal {
    /// Creates a `Normal(mean, std_dev)` distribution.
    ///
    /// # Errors
    /// Returns [`MathError::InvalidDistributionParameter`] if
    /// `std_dev <= 0`.
    pub fn new(mean: f64, std_dev: f64) -> Result<Self, MathError> {
        if std_dev <= 0.0 {
            return Err(MathError::InvalidDistributionParameter(format!(
                "Normal: std_dev ({std_dev}) must be > 0"
            )));
        }
        Ok(Self { mean, std_dev })
    }
}

impl Distribution for Normal {
    fn sample(&self, rng: &mut Rng) -> f64 {
        // Box-Muller transform. u1 is drawn from (0, 1] rather than
        // Rng::next_f64's [0, 1) to avoid ln(0).
        let u1 = 1.0 - rng.next_f64();
        let u2 = rng.next_f64();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        self.mean + self.std_dev * z0
    }
}

/// The exponential distribution with rate `lambda` (mean `1 / lambda`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Exponential {
    pub lambda: f64,
}

impl Exponential {
    /// Creates an `Exponential(lambda)` distribution.
    ///
    /// # Errors
    /// Returns [`MathError::InvalidDistributionParameter`] if
    /// `lambda <= 0`.
    pub fn new(lambda: f64) -> Result<Self, MathError> {
        if lambda <= 0.0 {
            return Err(MathError::InvalidDistributionParameter(format!(
                "Exponential: lambda ({lambda}) must be > 0"
            )));
        }
        Ok(Self { lambda })
    }
}

impl Distribution for Exponential {
    fn sample(&self, rng: &mut Rng) -> f64 {
        // Inverse-CDF sampling: -ln(1 - U) / lambda. 1 - next_f64() is
        // used (rather than next_f64() directly) to keep the argument to
        // ln in (0, 1] rather than [0, 1).
        -(1.0 - rng.next_f64()).ln() / self.lambda
    }
}

/// A Bernoulli (single coin-flip) distribution with success probability
/// `p`. Samples are exactly `0.0` or `1.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bernoulli {
    pub p: f64,
}

impl Bernoulli {
    /// Creates a `Bernoulli(p)` distribution.
    ///
    /// # Errors
    /// Returns [`MathError::InvalidDistributionParameter`] if `p` isn't in
    /// `[0, 1]`.
    pub fn new(p: f64) -> Result<Self, MathError> {
        if !(0.0..=1.0).contains(&p) {
            return Err(MathError::InvalidDistributionParameter(format!(
                "Bernoulli: p ({p}) must be in [0, 1]"
            )));
        }
        Ok(Self { p })
    }
}

impl Distribution for Bernoulli {
    fn sample(&self, rng: &mut Rng) -> f64 {
        if rng.next_f64() < self.p { 1.0 } else { 0.0 }
    }
}

/// The Poisson distribution with rate `lambda`, sampled via Knuth's
/// algorithm (a product of uniforms compared against `e^(-lambda)`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Poisson {
    pub lambda: f64,
}

impl Poisson {
    /// Creates a `Poisson(lambda)` distribution.
    ///
    /// # Errors
    /// Returns [`MathError::InvalidDistributionParameter`] if
    /// `lambda <= 0`.
    pub fn new(lambda: f64) -> Result<Self, MathError> {
        if lambda <= 0.0 {
            return Err(MathError::InvalidDistributionParameter(format!(
                "Poisson: lambda ({lambda}) must be > 0"
            )));
        }
        Ok(Self { lambda })
    }
}

impl Distribution for Poisson {
    fn sample(&self, rng: &mut Rng) -> f64 {
        let l = (-self.lambda).exp();
        let mut k = 0.0;
        let mut p = 1.0;
        loop {
            k += 1.0;
            p *= rng.next_f64();
            if p <= l {
                break;
            }
        }
        k - 1.0
    }
}

/// A categorical distribution over `{0, 1, ..., weights.len() - 1}`, with
/// `Pr[i] proportional to weights[i]`.
///
/// Samples are category indices, returned as `f64` (so `Categorical`
/// still implements [`Distribution`] alongside the continuous
/// distributions above; round the result to use it as an index).
#[derive(Debug, Clone, PartialEq)]
pub struct Categorical {
    cumulative: Vec<f64>,
}

impl Categorical {
    /// Creates a categorical distribution from unnormalized `weights`.
    ///
    /// # Errors
    /// Returns [`MathError::InvalidDistributionParameter`] if `weights` is
    /// empty, contains a negative entry, or sums to zero.
    pub fn new(weights: &[f64]) -> Result<Self, MathError> {
        if weights.is_empty() {
            return Err(MathError::InvalidDistributionParameter(
                "Categorical: weights must be non-empty".into(),
            ));
        }
        if weights.iter().any(|&w| w < 0.0) {
            return Err(MathError::InvalidDistributionParameter(
                "Categorical: weights must be non-negative".into(),
            ));
        }
        let total: f64 = weights.iter().sum();
        if total <= 0.0 {
            return Err(MathError::InvalidDistributionParameter(
                "Categorical: weights must sum to a positive value".into(),
            ));
        }
        let mut cumulative = Vec::with_capacity(weights.len());
        let mut running = 0.0;
        for &w in weights {
            running += w / total;
            cumulative.push(running);
        }
        Ok(Self { cumulative })
    }

    /// The number of categories.
    pub fn n_categories(&self) -> usize {
        self.cumulative.len()
    }
}

impl Distribution for Categorical {
    fn sample(&self, rng: &mut Rng) -> f64 {
        let u = rng.next_f64();
        for (i, &c) in self.cumulative.iter().enumerate() {
            if u < c {
                return i as f64;
            }
        }
        (self.cumulative.len() - 1) as f64
    }
}

/// A multivariate Gaussian distribution over `R^d`, parameterized by a
/// mean vector and a (symmetric positive-definite) covariance matrix.
#[derive(Debug, Clone, PartialEq)]
pub struct MultivariateNormal {
    mean: Vector,
    cholesky_l: Matrix,
}

impl MultivariateNormal {
    /// Creates a `MultivariateNormal(mean, cov)` distribution.
    ///
    /// # Errors
    /// Returns [`MathError::DimensionMismatch`] if `cov` isn't square or
    /// its size doesn't match `mean`'s length, and
    /// [`MathError::NotPositiveDefinite`] if `cov` isn't symmetric
    /// positive-definite.
    pub fn new(mean: Vector, cov: &Matrix) -> Result<Self, MathError> {
        if cov.rows() != cov.cols() || cov.rows() != mean.len() {
            return Err(MathError::DimensionMismatch(format!(
                "MultivariateNormal: mean has {} entries, cov is {}x{}",
                mean.len(),
                cov.rows(),
                cov.cols()
            )));
        }
        let cholesky_l = cov.cholesky()?.l;
        Ok(Self { mean, cholesky_l })
    }

    /// The dimensionality `d`.
    pub fn dim(&self) -> usize {
        self.mean.len()
    }

    /// Draws one sample: a [`Vector`] of length [`MultivariateNormal::dim`].
    pub fn sample(&self, rng: &mut Rng) -> Vector {
        let standard_normal = Normal { mean: 0.0, std_dev: 1.0 };
        let z: Vector = (0..self.dim()).map(|_| standard_normal.sample(rng)).collect();
        &self.mean + &self.cholesky_l.mul_vec(&z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mean_and_var(dist: &impl Distribution, n: usize, seed: u64) -> (f64, f64) {
        let mut rng = Rng::new(seed);
        let samples: Vec<f64> = (0..n).map(|_| dist.sample(&mut rng)).collect();
        let mean = samples.iter().sum::<f64>() / n as f64;
        let var =
            samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
        (mean, var)
    }

    #[test]
    fn uniform_mean_and_bounds() {
        let dist = Uniform::new(2.0, 6.0).unwrap();
        let mut rng = Rng::new(1);
        for _ in 0..10_000 {
            let x = dist.sample(&mut rng);
            assert!((2.0..6.0).contains(&x));
        }
        let (mean, _) = sample_mean_and_var(&dist, 100_000, 2);
        assert!((mean - 4.0).abs() < 0.05, "mean = {mean}");
    }

    #[test]
    fn uniform_rejects_bad_bounds() {
        assert!(Uniform::new(5.0, 1.0).is_err());
        assert!(Uniform::new(1.0, 1.0).is_err());
    }

    #[test]
    fn normal_mean_and_variance() {
        let dist = Normal::new(3.0, 2.0).unwrap();
        let (mean, var) = sample_mean_and_var(&dist, 200_000, 3);
        assert!((mean - 3.0).abs() < 0.02, "mean = {mean}");
        assert!((var - 4.0).abs() < 0.1, "var = {var}");
    }

    #[test]
    fn normal_rejects_nonpositive_std_dev() {
        assert!(Normal::new(0.0, 0.0).is_err());
        assert!(Normal::new(0.0, -1.0).is_err());
    }

    #[test]
    fn exponential_mean() {
        let dist = Exponential::new(0.5).unwrap();
        let (mean, _) = sample_mean_and_var(&dist, 200_000, 4);
        assert!((mean - 2.0).abs() < 0.05, "mean = {mean}");
    }

    #[test]
    fn bernoulli_proportion() {
        let dist = Bernoulli::new(0.3).unwrap();
        let mut rng = Rng::new(5);
        let n = 100_000;
        let ones = (0..n).filter(|_| dist.sample(&mut rng) == 1.0).count();
        let p_hat = ones as f64 / n as f64;
        assert!((p_hat - 0.3).abs() < 0.01, "p_hat = {p_hat}");
    }

    #[test]
    fn poisson_mean_equals_lambda() {
        let dist = Poisson::new(4.0).unwrap();
        let (mean, _) = sample_mean_and_var(&dist, 100_000, 6);
        assert!((mean - 4.0).abs() < 0.05, "mean = {mean}");
    }

    #[test]
    fn categorical_matches_weights() {
        let dist = Categorical::new(&[1.0, 3.0]).unwrap();
        let mut rng = Rng::new(7);
        let n = 100_000;
        let ones = (0..n).filter(|_| dist.sample(&mut rng) == 1.0).count();
        let p_hat = ones as f64 / n as f64;
        assert!((p_hat - 0.75).abs() < 0.01, "p_hat = {p_hat}");
    }

    #[test]
    fn categorical_rejects_bad_weights() {
        assert!(Categorical::new(&[]).is_err());
        assert!(Categorical::new(&[-1.0, 2.0]).is_err());
        assert!(Categorical::new(&[0.0, 0.0]).is_err());
    }

    #[test]
    fn multivariate_normal_sample_dimension() {
        let mean = Vector::from_slice(&[1.0, 2.0]);
        let mut cov = Matrix::zeros(2, 2);
        cov.set(0, 0, 2.0);
        cov.set(1, 1, 3.0);
        cov.set(0, 1, 0.5);
        cov.set(1, 0, 0.5);
        let dist = MultivariateNormal::new(mean, &cov).unwrap();
        let mut rng = Rng::new(8);
        let sample = dist.sample(&mut rng);
        assert_eq!(sample.len(), 2);
    }

    #[test]
    fn multivariate_normal_sample_mean_matches() {
        let mean = Vector::from_slice(&[5.0, -2.0]);
        let mut cov = Matrix::zeros(2, 2);
        cov.set(0, 0, 1.0);
        cov.set(1, 1, 1.0);
        let dist = MultivariateNormal::new(mean, &cov).unwrap();
        let mut rng = Rng::new(9);
        let n = 50_000;
        let mut sum = Vector::zeros(2);
        for _ in 0..n {
            sum = &sum + &dist.sample(&mut rng);
        }
        let empirical_mean = &sum * (1.0 / n as f64);
        assert!((empirical_mean[0] - 5.0).abs() < 0.05);
        assert!((empirical_mean[1] + 2.0).abs() < 0.05);
    }

    #[test]
    fn multivariate_normal_rejects_dimension_mismatch() {
        let mean = Vector::from_slice(&[1.0, 2.0, 3.0]);
        let cov = Matrix::identity(2);
        assert!(MultivariateNormal::new(mean, &cov).is_err());
    }
}
