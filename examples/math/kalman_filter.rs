//! Kalman Filter for Linear State Estimation
//!
//! Demonstrates a Kalman filter for tracking a 1D position/velocity system
//! using matrix operations for prediction and update steps.

use cnvx::math::random::{Distribution, Normal, Rng};
use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    println!("=== Kalman Filter: 1D Position/Velocity Tracking ===\n");

    // State: x = [position, velocity]'
    // Dynamics: x_{k+1} = F * x_k + w_k,  w_k ~ N(0, Q)
    // Measurement: z_k = H * x_k + v_k,    v_k ~ N(0, R)

    let dt = 0.1; // Time step
    let mut f = Matrix::zeros(2, 2);
    f.set(0, 0, 1.0);
    f.set(0, 1, dt);
    f.set(1, 0, 0.0);
    f.set(1, 1, 1.0);

    let mut h = Matrix::zeros(1, 2);
    h.set(0, 0, 1.0);
    h.set(0, 1, 0.0); // Measure position only

    // Process noise: assume acceleration noise
    let q_var = 0.01; // Acceleration variance
    let mut q = Matrix::zeros(2, 2);
    q.set(0, 0, dt.powi(4) / 4.0 * q_var);
    q.set(0, 1, dt.powi(3) / 2.0 * q_var);
    q.set(1, 0, dt.powi(3) / 2.0 * q_var);
    q.set(1, 1, dt.powi(2) * q_var);

    // Measurement noise
    let mut r = Matrix::zeros(1, 1);
    r.set(0, 0, 0.5); // Position measurement noise

    // Initial state and covariance
    let mut x = Vector::from_slice(&[0.0, 1.0]); // Start at 0, velocity 1
    let mut p = Matrix::zeros(2, 2);
    p.set(0, 0, 10.0);
    p.set(0, 1, 0.0);
    p.set(1, 0, 0.0);
    p.set(1, 1, 10.0); // High uncertainty

    println!("Initial state: {:.2}", x);
    println!("Initial covariance:\n{:.2}\n", p);

    // Simulate true trajectory
    let n_steps = 50;
    let mut rng = Rng::new(123);
    let q_noise = Normal::new(0.0, q_var.sqrt()).unwrap();
    let r_noise = Normal::new(0.0, r.get(0, 0).sqrt()).unwrap();

    let mut true_state = x.clone();
    let mut measurements = Vec::new();
    let mut true_positions = Vec::new();

    for _ in 0..n_steps {
        // True dynamics (with process noise)
        let accel = q_noise.sample(&mut rng);
        let mut f_true = Matrix::zeros(2, 2);
        f_true.set(0, 0, 1.0);
        f_true.set(0, 1, dt);
        f_true.set(1, 0, 0.0);
        f_true.set(1, 1, 1.0);
        true_state = f_true.mul_vec(&true_state)
            + Vector::from_slice(&[dt.powi(2) / 2.0 * accel, dt * accel]);
        true_positions.push(true_state[0]);

        // Noisy measurement
        let meas = h.mul_vec(&true_state)[0] + r_noise.sample(&mut rng);
        measurements.push(meas);
    }

    // Kalman filter loop
    let mut estimates = Vec::new();
    let mut covariances = Vec::new();
    let mut k_gain = Matrix::zeros(2, 1); // Will store last gain

    for k in 0..n_steps {
        // --- Predict ---
        let x_pred = f.mul_vec(&x);
        let f_t = f.transpose();
        let p_pred = f.mul(&p)?.mul(&f_t)?.add(&q)?;

        // --- Update ---
        let z = Vector::from_slice(&[measurements[k]]);

        // Innovation covariance: S = H * P_pred * H' + R
        let h_t = h.transpose();
        let s = h.mul(&p_pred)?.mul(&h_t)?.add(&r)?;

        // Kalman gain: K = P_pred * H' * S^{-1}
        // Use pseudo_inverse for S^{-1}
        let s_inv = s.pseudo_inverse()?;
        k_gain = p_pred.mul(&h_t)?.mul(&s_inv)?;

        // Innovation
        let y = &z - &h.mul_vec(&x_pred);

        // Update state and covariance
        x = &x_pred + &k_gain.mul_vec(&y);
        let i_kh = Matrix::identity(2).sub(&k_gain.mul(&h)?)?;
        p = i_kh
            .mul(&p_pred)?
            .mul(&i_kh.transpose())?
            .add(&k_gain.mul(&r)?.mul(&k_gain.transpose())?)?;

        estimates.push(x.clone());
        covariances.push(p.clone());

        if k % 10 == 0 {
            println!(
                "Step {k:2}: meas={:.3}, est={:.3}, true={:.3}, pos_std={:.3}",
                measurements[k],
                x[0],
                true_positions[k],
                p.get(0, 0).sqrt()
            );
        }
    }

    // Compute RMSE
    let mut pos_rmse = 0.0;
    for i in 0..n_steps {
        let diff = estimates[i][0] - true_positions[i];
        pos_rmse += diff * diff;
    }
    pos_rmse = (pos_rmse / n_steps as f64).sqrt();
    println!("\nPosition RMSE: {pos_rmse:.4}");

    // Steady-state analysis
    println!("\n=== Steady-State Kalman Gain ===");
    // Solve discrete algebraic Riccati equation for steady-state P
    let mut p_ss = p.clone();
    for _ in 0..100 {
        let p_pred = f.mul(&p_ss)?.mul(&f.transpose())?.add(&q)?;
        let s = h.mul(&p_pred)?.mul(&h.transpose())?.add(&r)?;
        let s_inv = s.pseudo_inverse()?;
        let k_ss = p_pred.mul(&h.transpose())?.mul(&s_inv)?;
        let i_kh = Matrix::identity(2).sub(&k_ss.mul(&h)?)?;
        p_ss = i_kh
            .mul(&p_pred)?
            .mul(&i_kh.transpose())?
            .add(&k_ss.mul(&r)?.mul(&k_ss.transpose())?)?;
    }
    let p_pred_ss = f.mul(&p_ss)?.mul(&f.transpose())?.add(&q)?;
    let s_ss = h.mul(&p_pred_ss)?.mul(&h.transpose())?.add(&r)?;
    let s_ss_inv = s_ss.pseudo_inverse()?;
    let k_ss = f.mul(&p_ss)?.mul(&h.transpose())?.mul(&s_ss_inv)?;

    println!("Steady-state Kalman gain: [{:.4}, {:.4}]", k_ss.get(0, 0), k_ss.get(1, 0));
    println!("Steady-state position variance: {:.4}", p_ss.get(0, 0));

    // Compare with last filter estimate
    println!("\nFinal filter gain: [{:.4}, {:.4}]", k_gain.get(0, 0), k_gain.get(1, 0));
    println!("Final filter position variance: {:.4}", p.get(0, 0));

    // Expected output:
    //
    // === Kalman Filter: 1D Position/Velocity Tracking ===
    //
    // Initial state: [0.00, 1.00]
    // Initial covariance:
    // Matrix { rows: 2, cols: 2, data: [10.00, 0.00, 0.00, 10.00] }
    //
    // Step  0: meas=0.655, est=0.628, true=0.100, pos_std=0.690
    // Step 10: meas=1.267, est=1.066, true=1.101, pos_std=0.392
    // Step 20: meas=2.003, est=1.962, true=2.102, pos_std=0.297
    // Step 30: meas=3.432, est=3.137, true=3.114, pos_std=0.248
    // Step 40: meas=3.692, est=4.148, true=4.101, pos_std=0.218
    //
    // Position RMSE: 0.2297
    //
    // === Steady-State Kalman Gain ===
    // Steady-state Kalman gain: [0.0505, 0.0131]
    // Steady-state position variance: 0.0259
    //
    // Final filter gain: [0.0796, 0.0259]
    // Final filter position variance: 0.0398

    Ok(())
}
