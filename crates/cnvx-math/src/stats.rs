use crate::matrix::Matrix;
use crate::vector::Vector;

/// The arithmetic mean.
///
/// # Panics
/// Panics if `data` is empty.
pub fn mean(data: &Vector) -> f64 {
    assert!(!data.is_empty(), "stats::mean: empty input");
    data.sum() / data.len() as f64
}

/// The sample variance (Bessel's correction: divides by `n - 1`).
///
/// # Panics
/// Panics if `data` has fewer than 2 elements.
pub fn variance(data: &Vector) -> f64 {
    assert!(data.len() >= 2, "stats::variance: need at least 2 points");
    let m = mean(data);
    let sum_sq: f64 = data.iter().map(|x| (x - m).powi(2)).sum();
    sum_sq / (data.len() - 1) as f64
}

/// The sample standard deviation: `sqrt(variance(data))`.
///
/// # Panics
/// Panics if `data` has fewer than 2 elements.
pub fn std_dev(data: &Vector) -> f64 {
    variance(data).sqrt()
}

/// The median. For an even-length input, averages the two middle
/// elements.
///
/// # Panics
/// Panics if `data` is empty.
pub fn median(data: &Vector) -> f64 {
    quantile(data, 0.5)
}

/// The `q`-quantile (`q` in `[0, 1]`), via linear interpolation between
/// the two closest ranks.
///
/// # Panics
/// Panics if `data` is empty or `q` isn't in `[0, 1]`.
pub fn quantile(data: &Vector, q: f64) -> f64 {
    assert!(!data.is_empty(), "stats::quantile: empty input");
    assert!((0.0..=1.0).contains(&q), "stats::quantile: q must be in [0, 1]");
    let mut sorted: Vec<f64> = data.iter().copied().collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let pos = q * (n - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let frac = pos - lo as f64;
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }
}

/// The sample covariance between two equal-length vectors (Bessel's
/// correction: divides by `n - 1`).
///
/// # Panics
/// Panics if `x.len() != y.len()` or either has fewer than 2 elements.
pub fn covariance(x: &Vector, y: &Vector) -> f64 {
    assert_eq!(x.len(), y.len(), "stats::covariance: dimension mismatch");
    assert!(x.len() >= 2, "stats::covariance: need at least 2 points");
    let mx = mean(x);
    let my = mean(y);
    let sum: f64 = x.iter().zip(y.iter()).map(|(&a, &b)| (a - mx) * (b - my)).sum();
    sum / (x.len() - 1) as f64
}

/// The Pearson correlation coefficient between two equal-length vectors:
/// `covariance(x, y) / (std_dev(x) * std_dev(y))`.
///
/// # Panics
/// Panics if `x.len() != y.len()` or either has fewer than 2 elements.
pub fn correlation(x: &Vector, y: &Vector) -> f64 {
    covariance(x, y) / (std_dev(x) * std_dev(y))
}

/// The sample covariance matrix of a set of observations.
///
/// `observations[i]` is treated as one observation (a row); the returned
/// matrix is `d x d`, where `d = observations[0].len()` is the number of
/// variables.
///
/// # Panics
/// Panics if `observations` is empty, has fewer than 2 rows, or the rows
/// don't all have the same length.
pub fn covariance_matrix(observations: &[Vector]) -> Matrix {
    assert!(
        observations.len() >= 2,
        "stats::covariance_matrix: need at least 2 observations"
    );
    let d = observations[0].len();
    for obs in observations {
        assert_eq!(obs.len(), d, "stats::covariance_matrix: ragged observations");
    }
    let n = observations.len();
    let mut means = vec![0.0; d];
    for obs in observations {
        for (k, &v) in obs.iter().enumerate() {
            means[k] += v;
        }
    }
    for m in means.iter_mut() {
        *m /= n as f64;
    }

    let mut cov = Matrix::zeros(d, d);
    for i in 0..d {
        for j in i..d {
            let s: f64 = observations
                .iter()
                .map(|obs| (obs[i] - means[i]) * (obs[j] - means[j]))
                .sum::<f64>()
                / (n - 1) as f64;
            cov.set(i, j, s);
            cov.set(j, i, s);
        }
    }
    cov
}

/// The Pearson correlation matrix of a set of observations, derived from
/// [`covariance_matrix`] by normalizing each entry by the corresponding
/// pair of standard deviations.
///
/// # Panics
/// Panics if `observations` is empty, has fewer than 2 rows, or the rows
/// don't all have the same length.
pub fn correlation_matrix(observations: &[Vector]) -> Matrix {
    let cov = covariance_matrix(observations);
    let d = cov.rows();
    let std_devs: Vec<f64> = (0..d).map(|i| cov.get(i, i).sqrt()).collect();
    let mut corr = Matrix::zeros(d, d);
    for i in 0..d {
        for j in 0..d {
            corr.set(i, j, cov.get(i, j) / (std_devs[i] * std_devs[j]));
        }
    }
    corr
}

/// A fixed-width histogram: `bins.len()` equal-width buckets spanning
/// `[min, max]`, with counts of how many samples fell in each.
#[derive(Debug, Clone, PartialEq)]
pub struct Histogram {
    /// The lower edge of each bin, plus one final entry for the upper edge
    /// of the last bin (so `edges.len() == counts.len() + 1`).
    pub edges: Vec<f64>,
    /// The number of samples falling in each bin.
    pub counts: Vec<usize>,
}

impl Histogram {
    /// Builds a histogram of `data` with `n_bins` equal-width bins
    /// spanning `[min(data), max(data)]`.
    ///
    /// Samples exactly equal to the maximum are placed in the last bin
    ///
    /// # Panics
    /// Panics if `data` is empty or `n_bins == 0`.
    pub fn new(data: &Vector, n_bins: usize) -> Self {
        assert!(!data.is_empty(), "Histogram::new: empty input");
        assert!(n_bins > 0, "Histogram::new: n_bins must be > 0");
        let min = data.min();
        let max = data.max();
        let width = if max > min { (max - min) / n_bins as f64 } else { 1.0 };
        let edges: Vec<f64> = (0..=n_bins).map(|i| min + width * i as f64).collect();
        let mut counts = vec![0usize; n_bins];
        for &x in data.iter() {
            let mut idx = ((x - min) / width).floor() as isize;
            if idx < 0 {
                idx = 0;
            }
            if idx as usize >= n_bins {
                idx = n_bins as isize - 1;
            }
            counts[idx as usize] += 1;
        }
        Self { edges, counts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_variance_std_dev() {
        let v = Vector::from_slice(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        assert!((mean(&v) - 5.0).abs() < 1e-12);
        assert!((variance(&v) - 4.5714285714285).abs() < 1e-9);
        assert!((std_dev(&v) - 2.13809).abs() < 1e-4);
    }

    #[test]
    fn median_odd_even() {
        let odd = Vector::from_slice(&[1.0, 3.0, 2.0]);
        assert!((median(&odd) - 2.0).abs() < 1e-12);
        let even = Vector::from_slice(&[1.0, 2.0, 3.0, 4.0]);
        assert!((median(&even) - 2.5).abs() < 1e-12);
    }

    #[test]
    fn correlation_perfect() {
        let x = Vector::from_slice(&[1.0, 2.0, 3.0, 4.0]);
        let y = Vector::from_slice(&[2.0, 4.0, 6.0, 8.0]);
        assert!((correlation(&x, &y) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn covariance_matrix_diagonal_is_variance() {
        let obs = vec![
            Vector::from_slice(&[1.0, 5.0]),
            Vector::from_slice(&[2.0, 4.0]),
            Vector::from_slice(&[3.0, 3.0]),
            Vector::from_slice(&[4.0, 2.0]),
        ];
        let cov = covariance_matrix(&obs);
        let x = Vector::from_slice(&[1.0, 2.0, 3.0, 4.0]);
        assert!((cov.get(0, 0) - variance(&x)).abs() < 1e-9);
        // x and y are perfectly anti-correlated.
        assert!(cov.get(0, 1) < 0.0);
    }

    #[test]
    fn histogram_counts_all_samples() {
        let data = Vector::from_slice(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 9.9, 10.0]);
        let hist = Histogram::new(&data, 5);
        assert_eq!(hist.counts.iter().sum::<usize>(), data.len());
        assert_eq!(hist.edges.len(), 6);
    }
}
