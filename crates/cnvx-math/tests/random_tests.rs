use cnvx_math::random::{
    Bernoulli, Categorical, Distribution, Exponential, MultivariateNormal, Normal,
    Poisson, Rng, Uniform,
};
use cnvx_math::{Matrix, Vector};

#[test]
fn reproducibility_across_all_distributions() {
    let dists: Vec<Box<dyn Distribution>> = vec![
        Box::new(Uniform::new(0.0, 1.0).unwrap()),
        Box::new(Normal::new(0.0, 1.0).unwrap()),
        Box::new(Exponential::new(1.0).unwrap()),
        Box::new(Bernoulli::new(0.5).unwrap()),
        Box::new(Poisson::new(3.0).unwrap()),
        Box::new(Categorical::new(&[1.0, 1.0, 2.0]).unwrap()),
    ];
    for dist in dists {
        let mut a = Rng::new(99);
        let mut b = Rng::new(99);
        let samples_a: Vec<f64> = (0..20).map(|_| dist.sample(&mut a)).collect();
        let samples_b: Vec<f64> = (0..20).map(|_| dist.sample(&mut b)).collect();
        assert_eq!(samples_a, samples_b);
    }
}

#[test]
fn multivariate_normal_respects_covariance_structure() {
    // Strongly correlated 2-D Gaussian: covariance [[1, 0.9], [0.9, 1]].
    let mean = Vector::from_slice(&[0.0, 0.0]);
    let mut cov = Matrix::zeros(2, 2);
    cov.set(0, 0, 1.0);
    cov.set(1, 1, 1.0);
    cov.set(0, 1, 0.9);
    cov.set(1, 0, 0.9);
    let dist = MultivariateNormal::new(mean, &cov).unwrap();

    let mut rng = Rng::new(42);
    let n = 50_000;
    let samples: Vec<Vector> = (0..n).map(|_| dist.sample(&mut rng)).collect();
    let xs = Vector::from(samples.iter().map(|s| s[0]).collect::<Vec<_>>());
    let ys = Vector::from(samples.iter().map(|s| s[1]).collect::<Vec<_>>());
    let corr = cnvx_math::stats::correlation(&xs, &ys);
    assert!((corr - 0.9).abs() < 0.03, "correlation = {corr}");
}
