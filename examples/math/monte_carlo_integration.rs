//! Monte Carlo Integration
//!
//! Demonstrates Monte Carlo and quasi-Monte Carlo integration
//! for high-dimensional integrals where quadrature fails.

use cnvx::math::random::{
    Distribution, Exponential, MultivariateNormal, Normal, Rng, Uniform,
};
use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    println!("=== Monte Carlo Integration ===\n");

    // --- Example 1: Simple 1D integral for validation ---
    // int_0^1 x^2 dx = 1/3
    println!("--- 1D: int_0^1 x^2 dx = 1/3 ---");
    let mut rng = Rng::new(42);
    let uniform = Uniform::new(0.0, 1.0).unwrap();

    for &n in &[100, 1000, 10000, 100000] {
        let sum: f64 = (0..n)
            .map(|_| {
                let x = uniform.sample(&mut rng);
                x * x
            })
            .sum();
        let estimate = sum / n as f64;
        let error = (estimate - 1.0 / 3.0).abs();
        println!("  n={n:6}: estimate={estimate:.6}, error={error:.2e}");
    }

    // --- Example 2: High-dimensional integral ---
    // int_[0,1]^d prod_i x_i dx = (1/2)^d
    println!("\n--- d-D: int_[0,1]^d prod x_i dx = (1/2)^d ---");
    for &d in &[2, 5, 10, 20] {
        let exact = 0.5_f64.powi(d);
        let n = 100000;
        let mut sum = 0.0;

        for _ in 0..n {
            let mut prod = 1.0;
            for _ in 0..d {
                prod *= uniform.sample(&mut rng);
            }
            sum += prod;
        }
        let estimate = sum / n as f64;
        let rel_error = (estimate - exact).abs() / exact;
        println!(
            "  d={d:2}: exact={exact:.2e}, MC={estimate:.2e}, rel_error={:.2}%",
            rel_error * 100.0
        );
    }

    // --- Example 3: Importance Sampling ---
    // int_0^inf x^2 e^{-x} dx = Gamma(3) = 2
    // Use exponential distribution as importance sampling
    println!("\n--- Importance Sampling: int_0^inf x^2 e^{{-x}} dx = 2 ---");
    let exp_dist = Exponential::new(1.0).unwrap();

    for &n in &[1000, 10000, 100000] {
        let mut sum = 0.0;
        for _ in 0..n {
            let x = exp_dist.sample(&mut rng);
            // f(x) / p(x) = x^2 e^{-x} / e^{-x} = x^2
            sum += x * x;
        }
        let estimate = sum / n as f64;
        let error = (estimate - 2.0).abs();
        println!("  n={n:6}: estimate={estimate:.6}, error={error:.2e}");
    }

    // --- Example 4: Stratified Sampling ---
    println!("\n--- Stratified Sampling: int_0^1 x^2 dx ---");
    for &n in &[100, 1000, 10000] {
        let strata = 10;
        let per_stratum = n / strata;
        let mut sum = 0.0;
        for s in 0..strata {
            let a = s as f64 / strata as f64;
            let b = (s + 1) as f64 / strata as f64;
            let mut stratum_sum = 0.0;
            for _ in 0..per_stratum {
                let x = a + (b - a) * uniform.sample(&mut rng);
                stratum_sum += x * x;
            }
            sum += stratum_sum / per_stratum as f64 * (b - a);
        }
        let error = (sum - 1.0 / 3.0).abs();
        println!("  n={n:6}: estimate={sum:.6}, error={error:.2e}");
    }

    // --- Example 5: Estimating pi ---
    // Area of quarter circle = pi/4
    println!("\n--- Estimating pi (quarter circle area) ---");
    for &n in &[1000, 10000, 100000, 1000000] {
        let mut inside = 0;
        for _ in 0..n {
            let x = uniform.sample(&mut rng);
            let y = uniform.sample(&mut rng);
            if x * x + y * y <= 1.0 {
                inside += 1;
            }
        }
        let pi_est = 4.0 * inside as f64 / n as f64;
        let error = (pi_est - std::f64::consts::PI).abs();
        println!("  n={n:7}: pi ~= {pi_est:.6}, error={error:.2e}");
    }

    // --- Example 6: Multivariate Normal Probability ---
    // P(X1 > 0, X2 > 0) for bivariate normal with correlation rho
    println!("\n--- Bivariate Normal Orthant Probability ---");
    let rho = 0.5;
    let mean = Vector::from_slice(&[0.0, 0.0]);
    let mut cov = Matrix::zeros(2, 2);
    cov.set(0, 0, 1.0);
    cov.set(1, 1, 1.0);
    cov.set(0, 1, rho);
    cov.set(1, 0, rho);

    let mvn = MultivariateNormal::new(mean, &cov).unwrap();

    for &n in &[10000, 100000, 1000000] {
        let mut count = 0;
        for _ in 0..n {
            let sample = mvn.sample(&mut rng);
            if sample[0] > 0.0 && sample[1] > 0.0 {
                count += 1;
            }
        }
        let prob = count as f64 / n as f64;
        // Exact: 1/4 + arcsin(rho) / (2*pi)
        let exact = 0.25 + rho.asin() / (2.0 * std::f64::consts::PI);
        println!(
            "  n={n:7}: P~={prob:.6}, exact={exact:.6}, error={:.2e}",
            (prob - exact).abs()
        );
    }

    // --- Example 7: Integration over a Sphere ---
    // int_{||x||<=1} f(x) dx for f(x) = ||x||^2 in 3D
    // Volume of unit sphere = 4*pi/3, average of r^2 over sphere = 3/5
    // Integral = (4*pi/3) * (3/5) = 4*pi/5
    println!("\n--- 3D Ball: int_{{||x||<=1}} ||x||^2 dx = 4*pi/5 ---");
    let exact = 4.0 * std::f64::consts::PI / 5.0;

    for &n in &[10000, 100000, 1000000] {
        let mut sum = 0.0;
        let mut count = 0;
        // Rejection sampling: sample from [-1,1]^3, accept if in ball
        for _ in 0..n * 2 {
            // Oversample to account for rejection
            let x = 2.0 * uniform.sample(&mut rng) - 1.0;
            let y = 2.0 * uniform.sample(&mut rng) - 1.0;
            let z = 2.0 * uniform.sample(&mut rng) - 1.0;
            let r2 = x * x + y * y + z * z;
            if r2 <= 1.0 {
                sum += r2;
                count += 1;
                if count >= n {
                    break;
                }
            }
        }
        // Volume of [-1,1]^3 = 8, acceptance ratio ~ (4*pi/3)/8 = pi/6
        let volume_est = 8.0 * count as f64 / (n * 2) as f64;
        let integral_est = volume_est * sum / count as f64;
        let error = (integral_est - exact).abs();
        println!(
            "  n={n:7}: integral={integral_est:.6}, exact={exact:.6}, error={error:.2e}"
        );
    }

    // --- Example 8: Financial Option Pricing (Asian Option) ---
    // Asian call option: payoff = max(avg(S) - K, 0)
    // Geometric Brownian Motion: S(t) = S0 exp((r-sigma^2/2)t + sigma*W(t))
    println!("\n--- Asian Option Pricing (Monte Carlo) ---");
    let s0 = 100.0;
    let k = 100.0;
    let r = 0.05;
    let sigma = 0.2;
    let t = 1.0;
    let n_steps = 12; // Monthly averaging
    let dt = t / n_steps as f64;
    let drift = (r - 0.5 * sigma * sigma) * dt;
    let vol = sigma * dt.sqrt();

    for &n in &[10000, 100000] {
        let mut sum_payoff = 0.0;
        for _ in 0..n {
            let mut s = s0;
            let mut avg = 0.0;
            for _ in 0..n_steps {
                let z = Normal::new(0.0, 1.0).unwrap().sample(&mut rng);
                s *= (drift + vol * z).exp();
                avg += s;
            }
            avg /= n_steps as f64;
            let payoff = (avg - k).max(0.0);
            sum_payoff += payoff;
        }
        let price = (-r * t).exp() * sum_payoff / n as f64;
        println!("  n={n:6}: Asian call price = ${price:.2}");
    }

    // Expected output:
    //
    // === Monte Carlo Integration ===
    //
    // --- 1D: int_0^1 x^2 dx = 1/3 ---
    //   n=   100: estimate=0.377699, error=4.44e-2
    //   n=  1000: estimate=0.325992, error=7.34e-3
    //   n= 10000: estimate=0.332031, error=1.30e-3
    //   n=100000: estimate=0.335074, error=1.74e-3
    //
    // --- d-D: int_[0,1]^d prod x_i dx = (1/2)^d ---
    //   d= 2: exact=2.50e-1, MC=2.50e-1, rel_error=0.07%
    //   d= 5: exact=3.12e-2, MC=3.12e-2, rel_error=0.32%
    //   d=10: exact=9.77e-4, MC=9.67e-4, rel_error=0.97%
    //   d=20: exact=9.54e-7, MC=9.88e-7, rel_error=3.57%
    //
    // --- Importance Sampling: int_0^inf x^2 e^{-x} dx = 2 ---
    //   n=  1000: estimate=1.718559, error=2.81e-1
    //   n= 10000: estimate=1.958743, error=4.13e-2
    //   n=100000: estimate=2.005046, error=5.05e-3
    //
    // --- Stratified Sampling: int_0^1 x^2 dx ---
    //   n=   100: estimate=0.332412, error=9.22e-4
    //   n=  1000: estimate=0.332436, error=8.97e-4
    //   n= 10000: estimate=0.333856, error=5.23e-4
    //
    // --- Estimating pi (quarter circle area) ---
    //   n=   1000: pi ~= 3.168000, error=2.64e-2
    //   n=  10000: pi ~= 3.129600, error=1.20e-2
    //   n= 100000: pi ~= 3.136200, error=5.39e-3
    //   n=1000000: pi ~= 3.141972, error=3.79e-4
    //
    // --- Bivariate Normal Orthant Probability ---
    //   n=  10000: P~=0.322100, exact=0.333333, error=1.12e-2
    //   n= 100000: P~=0.331680, exact=0.333333, error=1.65e-3
    //   n=1000000: P~=0.333337, exact=0.333333, error=3.67e-6
    //
    // --- 3D Ball: int_{||x||<=1} ||x||^2 dx = 4*pi/5 ---
    //   n=  10000: integral=2.410291, exact=2.513274, error=1.03e-1
    //   n= 100000: integral=2.402331, exact=2.513274, error=1.11e-1
    //   n=1000000: integral=2.400568, exact=2.513274, error=1.13e-1
    //
    // --- Asian Option Pricing (Monte Carlo) ---
    //   n= 10000: Asian call price = $6.08
    //   n=100000: Asian call price = $6.13

    Ok(())
}
