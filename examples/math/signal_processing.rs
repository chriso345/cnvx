//! Signal Processing: Filtering and Frequency Analysis
//!
//! Demonstrates signal processing using linear algebra:
//! - FIR filter design via least squares
//! - Frequency response analysis via eigendecomposition
//! - Signal denoising using SVD

use cnvx::math::random::{Distribution, Normal, Rng};
use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    println!("=== FIR Filter Design (Least Squares) ===\n");

    // Design a lowpass filter: passband [0, 0.2*pi], stopband [0.3*pi, pi]
    let n_taps = 31; // Filter length (odd for linear phase)
    let n_freq = 512;
    let mut freqs = Vec::new();
    let mut desired = Vec::new();
    let mut weights = Vec::new();

    for i in 0..n_freq {
        let w = std::f64::consts::PI * i as f64 / n_freq as f64;
        freqs.push(w);

        if w < 0.2 * std::f64::consts::PI {
            desired.push(1.0);
            weights.push(1.0);
        } else if w > 0.3 * std::f64::consts::PI {
            desired.push(0.0);
            weights.push(10.0); // Higher weight in stopband
        } else {
            // Transition band - don't care
            desired.push(0.5);
            weights.push(0.01);
        }
    }

    // Build least squares problem: A*h = d
    // A[i,k] = cos(w_i * k) for Type I linear phase filter
    let mut a = Matrix::zeros(n_freq, n_taps);
    for (i, &w) in freqs.iter().enumerate() {
        for k in 0..n_taps {
            a.set(i, k, (w * k as f64).cos());
        }
    }

    let d = Vector::from(desired);
    let w_vec = Vector::from(weights);

    // Weighted least squares: (A^T W A) h = A^T W d
    let a_t = a.transpose();
    let mut ata = a_t.mul(&a)?;
    let atd = a_t.mul_vec(&d);

    for i in 0..n_taps {
        let wi = w_vec[i];
        for j in 0..n_taps {
            ata.set(i, j, ata.get(i, j) * wi);
        }
    }
    // Apply weights to atd as well
    let mut atd_weighted = Vector::zeros(n_taps);
    for i in 0..n_taps {
        atd_weighted[i] = atd[i] * w_vec[i];
    }

    let h = ata.solve(&atd_weighted)?;
    println!("Filter coefficients (h[0]..h[{}]):", n_taps - 1);
    for (i, &coeff) in h.iter().enumerate() {
        println!("  h[{i}] = {coeff:.6}");
    }

    // Verify frequency response
    println!("\nFrequency response at key points:");
    for &w in &[
        0.0,
        0.1 * std::f64::consts::PI,
        0.2 * std::f64::consts::PI,
        0.3 * std::f64::consts::PI,
        0.5 * std::f64::consts::PI,
        std::f64::consts::PI,
    ] {
        let mut response = 0.0;
        for k in 0..n_taps {
            response += h[k] * (w * k as f64).cos();
        }
        let db = 20.0 * response.abs().log10();
        println!("  w = {w:.3} rad: |H| = {response:.4} ({db:.1} dB)");
    }

    // ===== Signal Denoising via SVD =====
    println!("\n=== Signal Denoising via SVD ===\n");

    // Generate a clean signal: sum of sinusoids
    let n = 200;
    let t: Vec<f64> = (0..n)
        .map(|i| 2.0 * std::f64::consts::PI * i as f64 / n as f64)
        .collect();
    let clean: Vector = t
        .iter()
        .map(|&ti| (ti).sin() + 0.5 * (3.0 * ti).sin() + 0.3 * (5.0 * ti).sin())
        .collect();

    // Add noise
    let mut rng = Rng::new(42);
    let noise_dist = Normal::new(0.0, 0.3).unwrap();
    let noisy: Vector = clean.iter().map(|&c| c + noise_dist.sample(&mut rng)).collect();

    // Form Hankel matrix for SVD-based denoising
    let m = 50; // Window size
    let k = n - m + 1;
    let mut hankel = Matrix::zeros(m, k);
    for i in 0..m {
        for j in 0..k {
            hankel.set(i, j, noisy[i + j]);
        }
    }

    // SVD
    let svd = hankel.svd()?;
    let s = &svd.s;

    println!("Singular values (top 10):");
    for i in 0..10.min(s.len()) {
        println!("  s[{i}] = {:.4}", s[i]);
    }

    // Keep only significant singular values (signal subspace)
    let threshold = s[0] * 0.1; // 10% of largest
    let r = s.iter().filter(|&&v| v > threshold).count();
    println!("\nKeeping {r} significant components (threshold = {threshold:.4})");

    // Reconstruct denoised signal
    let mut s_denoised = vec![0.0; s.len()];
    for i in 0..r {
        s_denoised[i] = s[i];
    }

    // Reconstruct hankel matrix: U * diag(s) * V^T
    // svd.vt is already V^T
    let u = &svd.u;
    let vt = &svd.vt;
    let mut hankel_denoised = Matrix::zeros(m, k);
    for i in 0..m {
        for j in 0..k {
            let mut sum = 0.0;
            for (l, &s_val) in s_denoised.iter().enumerate().take(r) {
                sum += u.get(i, l) * s_val * vt.get(l, j);
            }
            hankel_denoised.set(i, j, sum);
        }
    }

    // Diagonal averaging to get 1D signal back
    let mut denoised = vec![0.0; n];
    let mut counts = vec![0; n];
    for i in 0..m {
        for j in 0..k {
            denoised[i + j] += hankel_denoised.get(i, j);
            counts[i + j] += 1;
        }
    }
    for i in 0..n {
        denoised[i] /= counts[i] as f64;
    }

    // Compute MSE
    let mut mse_clean = 0.0;
    let mut mse_denoised = 0.0;
    for i in 0..n {
        let diff_clean = clean[i] - noisy[i];
        mse_clean += diff_clean * diff_clean;
        let diff_denoised = clean[i] - denoised[i];
        mse_denoised += diff_denoised * diff_denoised;
    }
    mse_clean /= n as f64;
    mse_denoised /= n as f64;

    println!("Noisy MSE: {mse_clean:.4}");
    println!("Denoised MSE: {mse_denoised:.4}");
    println!("Improvement: {:.1}x", mse_clean / mse_denoised);

    // ===== Frequency Analysis of a Covariance Matrix =====
    println!("\n=== Spectral Analysis of AR(1) Process ===\n");

    // AR(1): x_t = 0.9 * x_{t-1} + noise
    // Theoretical autocovariance: gamma(k) = sigma^2 * 0.9^|k| / (1 - 0.9^2)
    let phi = 0.9_f64;
    let n = 100;
    let mut cov = Matrix::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            cov.set(i, j, phi.powi((i as i32 - j as i32).abs()));
        }
    }

    let eigen = cov.eigen_symmetric()?;
    println!("Eigenvalues (power spectrum) - top 10:");
    for i in 0..10 {
        println!("  lambda[{i}] = {:.4}", eigen.values[i]);
    }

    // The eigenvalues approximate the power spectral density
    // For AR(1), spectrum is S(w) = 1 / |1 - phi * e^{-iw}|^2
    println!("\nTheoretical spectrum at w = 0: {:.4}", 1.0 / (1.0 - phi).powi(2));
    println!("Theoretical spectrum at w = pi: {:.4}", 1.0 / (1.0 + phi).powi(2));

    // Expected output:
    //
    // === FIR Filter Design (Least Squares) ===
    //
    // Filter coefficients (h[0]..h[30]):
    //   h[0] = 0.250004
    //   h[1] = 0.444672
    //   h[2] = 0.302862
    //   h[3] = 0.133837
    //   h[4] = 0.000009
    //   h[5] = -0.063847
    //   h[6] = -0.062676
    //   h[7] = -0.029431
    //   h[8] = 0.000009
    //   h[9] = 0.008110
    //   h[10] = 0.000400
    //   h[11] = -0.006118
    //   h[12] = 0.000009
    //   h[13] = 0.015490
    //   h[14] = 0.026428
    //   h[15] = 0.021044
    //   h[16] = 0.000009
    //   h[17] = -0.023466
    //   h[18] = -0.033522
    //   h[19] = -0.023357
    //   h[20] = 0.000009
    //   h[21] = 0.021240
    //   h[22] = 0.027669
    //   h[23] = 0.017589
    //   h[24] = 0.000009
    //   h[25] = -0.012927
    //   h[26] = -0.014713
    //   h[27] = -0.007811
    //   h[28] = 0.000009
    //   h[29] = 0.002716
    //   h[30] = 0.000400
    //
    // Frequency response at key points:
    //   w = 0.000 rad: |H| = 0.9947 (-0.0 dB)
    //   w = 0.314 rad: |H| = 1.0096 (0.1 dB)
    //   w = 0.628 rad: |H| = 0.7686 (-2.3 dB)
    //   w = 0.942 rad: |H| = 0.2312 (-12.7 dB)
    //   w = 1.571 rad: |H| = 0.0032 (-49.8 dB)
    //   w = 3.142 rad: |H| = -0.0008 (-61.6 dB)
    //
    // === Signal Denoising via SVD ===
    //
    // Singular values (top 10):
    //   s[0] = 60.6485
    //   s[1] = 29.9347
    //   s[2] = 17.9087
    //   s[3] = 11.2744
    //   s[4] = 5.9725
    //   s[5] = 5.8137
    //   s[6] = 5.4672
    //   s[7] = 5.4327
    //   s[8] = 5.4101
    //   s[9] = 5.3165
    //
    // Keeping 4 significant components (threshold = 6.0649)
    // Noisy MSE: 0.0854
    // Denoised MSE: 0.0043
    // Improvement: 19.8x
    //
    // === Spectral Analysis of AR(1) Process ===
    //
    // Eigenvalues (power spectrum) - top 10:
    //   lambda[0] = 17.8717
    //   lambda[1] = 15.1163
    //   lambda[2] = 11.9362
    //   lambda[3] = 9.1391
    //   lambda[4] = 6.9673
    //   lambda[5] = 5.3651
    //   lambda[6] = 4.1989
    //   lambda[7] = 3.3456
    //   lambda[8] = 2.7127
    //   lambda[9] = 2.2355
    //
    // Theoretical spectrum at w = 0: 100.0000
    // Theoretical spectrum at w = pi: 0.2770

    Ok(())
}
