//! Random Sampling and Statistics
//!
//! Demonstrates random sampling and descriptive statistics:
//! - Drawing samples from a univariate Normal distribution and summarizing them
//!   (mean, std dev, median, quantile, histogram)
//! - Drawing correlated samples from a MultivariateNormal distribution and
//!   recovering the target correlation

use cnvx_math::random::{Distribution, MultivariateNormal, Normal, Rng};
use cnvx_math::{Matrix, Vector, stats};

fn main() {
    // A fixed seed makes this reproducible across runs.
    let mut rng = Rng::new(42);

    let normal = Normal::new(10.0, 2.0).expect("std_dev > 0");
    let samples: Vector = (0..10_000).map(|_| normal.sample(&mut rng)).collect();

    println!("sample mean = {:.3} (true mean = 10.0)", stats::mean(&samples));
    println!("sample std dev = {:.3} (true std dev = 2.0)", stats::std_dev(&samples));
    println!("sample median = {:.3}", stats::median(&samples));
    println!("90th percentile = {:.3}", stats::quantile(&samples, 0.9));

    let histogram = stats::Histogram::new(&samples, 10);
    println!("\nhistogram (10 bins):");
    for (i, &count) in histogram.counts.iter().enumerate() {
        let bar = "#".repeat(count / 100);
        println!("  [{:6.2}, {:6.2}): {bar}", histogram.edges[i], histogram.edges[i + 1]);
    }

    // Correlated samples via MultivariateNormal.
    let joint_mean = Vector::from_slice(&[0.0, 0.0]);
    let mut covariance = Matrix::zeros(2, 2);
    covariance.set(0, 0, 1.0);
    covariance.set(1, 1, 1.0);
    covariance.set(0, 1, 0.8);
    covariance.set(1, 0, 0.8);
    let joint_normal =
        MultivariateNormal::new(joint_mean, &covariance).expect("covariance is SPD");

    let correlated_samples: Vec<Vector> =
        (0..5_000).map(|_| joint_normal.sample(&mut rng)).collect();
    let x_samples =
        Vector::from(correlated_samples.iter().map(|p| p[0]).collect::<Vec<_>>());
    let y_samples =
        Vector::from(correlated_samples.iter().map(|p| p[1]).collect::<Vec<_>>());
    println!(
        "\nempirical correlation = {:.3} (target = 0.8)",
        stats::correlation(&x_samples, &y_samples)
    );

    // Expected output:
    //
    // sample mean = 10.007 (true mean = 10.0)
    // sample std dev = 1.987 (true std dev = 2.0)
    // sample median = 10.011
    // 90th percentile = 12.560
    //
    // histogram (10 bins):
    //   [  2.40,   3.91):
    //   [  3.91,   5.43): #
    //   [  5.43,   6.94): ####
    //   [  6.94,   8.46): ################
    //   [  8.46,   9.97): ###########################
    //   [  9.97,  11.49): ###########################
    //   [ 11.49,  13.00): ################
    //   [ 13.00,  14.52): #####
    //   [ 14.52,  16.03): #
    //   [ 16.03,  17.54):
    //
    // empirical correlation = 0.796 (target = 0.8)
}
