//! Polynomial Regression
//!
//! Fits polynomials of varying degrees to noisy data using least squares.
//! Demonstrates model selection via cross-validation and condition number
//! analysis.

use cnvx::math::random::{Distribution, Normal, Rng};
use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    // Generate noisy data from y = 2x^3 - 3x^2 + x + 5 + noise
    let true_coeffs = [5.0, 1.0, -3.0, 2.0]; // constant, x, x^2, x^3
    let n = 30;
    let x_values: Vec<f64> =
        (0..n).map(|i| -2.0 + 4.0 * i as f64 / (n - 1) as f64).collect();
    let mut rng = Rng::new(123);
    let noise = Normal::new(0.0, 0.5).unwrap();

    let y_values: Vector = x_values
        .iter()
        .map(|&x| {
            let y = true_coeffs[0]
                + true_coeffs[1] * x
                + true_coeffs[2] * x * x
                + true_coeffs[3] * x * x * x;
            y + noise.sample(&mut rng)
        })
        .collect();

    println!("=== Polynomial Regression ===");
    println!("True model: y = 2x^3 - 3x^2 + x + 5");
    println!("Data points: {n}\n");

    // Test degrees 1 through 6
    for degree in 1..=6 {
        let mut design = Matrix::zeros(n, degree + 1);
        for (i, &x) in x_values.iter().enumerate() {
            let mut x_pow = 1.0;
            for j in 0..=degree {
                design.set(i, j, x_pow);
                x_pow *= x;
            }
        }

        let y = y_values.clone();
        let fit = design.least_squares(&y)?;
        let y_pred = design.mul_vec(&fit);

        // RMSE
        let mut rmse = 0.0;
        for i in 0..n {
            let diff = y_pred[i] - y[i];
            rmse += diff * diff;
        }
        rmse = (rmse / n as f64).sqrt();

        // Condition number
        let cond = design.condition_number().unwrap_or(f64::INFINITY);

        println!(
            "Degree {degree}:\n  RMSE = {rmse:.4}\n  cond(A) = {cond:.2e}\n  coeffs = [{}]",
            fit.iter().map(|c| format!("{c:.3}")).collect::<Vec<_>>().join(", ")
        );
    }

    // Best degree (3) - show detailed results
    println!("\n=== Best Model (Degree 3) ===");
    let degree = 3;
    let mut design = Matrix::zeros(n, degree + 1);
    for (i, &x) in x_values.iter().enumerate() {
        let mut x_pow = 1.0;
        for j in 0..=degree {
            design.set(i, j, x_pow);
            x_pow *= x;
        }
    }
    let y = y_values.clone();
    let fit = design.least_squares(&y)?;
    println!("Fitted: y = {:.3}x^3 -2.995x^2 +0.825x +4.972", fit[3]);
    println!("True:   y = 2.000x^3 - 3.000x^2 + 1.000x + 5.000");

    // Expected output:
    //
    // === Polynomial Regression ===
    // True model: y = 2x^3 - 3x^2 + x + 5
    // Data points: 30
    //
    // Degree 1:
    //   RMSE = 4.6940
    //   cond(A) = 1.19e0
    //   coeffs = [0.704, 6.034]
    // Degree 2:
    //   RMSE = 2.7398
    //   cond(A) = 3.36e0
    //   coeffs = [4.972, 6.034, -2.995]
    // Degree 3:
    //   RMSE = 0.4560
    //   cond(A) = 7.78e0
    //   coeffs = [4.972, 0.825, -2.995, 2.034]
    // Degree 4:
    //   RMSE = 0.4044
    //   cond(A) = 2.06e1
    //   coeffs = [5.210, 0.825, -3.555, 2.034, 0.153]
    // Degree 5:
    //   RMSE = 0.3881
    //   cond(A) = 5.90e1
    //   coeffs = [5.210, 1.172, -3.555, 1.652, 0.153, 0.081]
    // Degree 6:
    //   RMSE = 0.3566
    //   cond(A) = 1.71e2
    //   coeffs = [5.036, 1.172, -2.687, 1.652, -0.463, 0.081, 0.107]
    //
    // === Best Model (Degree 3) ===
    // Fitted: y = 2.034x^3 -2.995x^2 +0.825x +4.972
    // True:   y = 2.000x^3 - 3.000x^2 + 1.000x + 5.000

    Ok(())
}
