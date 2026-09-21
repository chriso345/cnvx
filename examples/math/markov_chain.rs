//! Markov Chain Analysis
//!
//! Demonstrates steady-state computation, hitting times, and
//! absorption probabilities using eigendecomposition and linear solves.

use cnvx::prelude::*;

fn main() -> Result<(), cnvx_core::CnvxError> {
    println!("=== Markov Chain Analysis ===\n");

    // --- Example 1: Weather Model (3-state) ---
    // States: 0=Sunny, 1=Cloudy, 2=Rainy
    println!("--- Weather Model (3-state) ---");
    let mut p = Matrix::zeros(3, 3);
    p.set(0, 0, 0.7);
    p.set(0, 1, 0.2);
    p.set(0, 2, 0.1);
    p.set(1, 0, 0.3);
    p.set(1, 1, 0.4);
    p.set(1, 2, 0.3);
    p.set(2, 0, 0.2);
    p.set(2, 1, 0.3);
    p.set(2, 2, 0.5);

    println!("Transition matrix P:");
    for i in 0..3 {
        println!("  [{:.1}, {:.1}, {:.1}]", p.get(i, 0), p.get(i, 1), p.get(i, 2));
    }

    // Steady state: pi = pi*P, sum(pi) = 1
    // Solve (P^T - I)pi = 0 with sum(pi) = 1
    let p_t = p.transpose();
    let mut a = p_t.sub(&Matrix::identity(3))?;
    // Replace last row with ones
    for j in 0..3 {
        a.set(2, j, 1.0);
    }
    let mut b = Vector::zeros(3);
    b[2] = 1.0;

    let pi = a.solve(&b)?;
    println!(
        "\nSteady-state distribution pi: [{:.4}, {:.4}, {:.4}]",
        pi[0], pi[1], pi[2]
    );

    // Verify: pi*P = pi
    let pi_p = p.mul_vec(&pi);
    println!(
        "pi*P:                     [{:.4}, {:.4}, {:.4}]",
        pi_p[0], pi_p[1], pi_p[2]
    );
    let max_diff = pi
        .iter()
        .zip(pi_p.iter())
        .map(|(&a, &b)| (a - b).abs())
        .fold(0.0, f64::max);
    println!("Max diff: {max_diff:.2e}");

    // Eigenvector method
    let eigen = p_t.eigen_symmetric()?;
    // Find eigenvalue closest to 1
    let mut idx = 0;
    let mut min_diff = f64::INFINITY;
    for (i, &val) in eigen.values.iter().enumerate() {
        let diff = (val - 1.0).abs();
        if diff < min_diff {
            min_diff = diff;
            idx = i;
        }
    }
    let pi_eigen = eigen.vectors.get_col(idx);
    // Normalize
    let sum: f64 = pi_eigen.iter().sum();
    println!(
        "\nVia eigendecomposition:  [{:.4}, {:.4}, {:.4}]",
        pi_eigen[0] / sum,
        pi_eigen[1] / sum,
        pi_eigen[2] / sum
    );

    // --- Example 2: PageRank-style Computation ---
    println!("\n--- PageRank (4 pages) ---");
    // 4 pages with links: 0->1, 0->2, 1->2, 1->3, 2->0, 3->0, 3->1
    let mut links = Matrix::zeros(4, 4);
    links.set(0, 1, 1.0);
    links.set(0, 2, 1.0);
    links.set(1, 2, 1.0);
    links.set(1, 3, 1.0);
    links.set(2, 0, 1.0);
    links.set(3, 0, 1.0);
    links.set(3, 1, 1.0);

    // Column-normalize to get transition matrix
    let mut p = Matrix::zeros(4, 4);
    for j in 0..4 {
        let col_sum: f64 = (0..4).map(|i| links.get(i, j)).sum();
        if col_sum > 0.0 {
            for i in 0..4 {
                p.set(i, j, links.get(i, j) / col_sum);
            }
        } else {
            // Dangling node: uniform
            for i in 0..4 {
                p.set(i, j, 0.25);
            }
        }
    }

    // Google matrix: G = alpha*P + (1-alpha)*11^T/n
    let alpha = 0.85;
    let n = 4;
    let mut g = p.mul_scalar(alpha);
    for i in 0..n {
        for j in 0..n {
            g.set(i, j, g.get(i, j) + (1.0 - alpha) / n as f64);
        }
    }

    println!("Google matrix G (alpha={alpha}):");
    for i in 0..4 {
        println!(
            "  [{:.3}, {:.3}, {:.3}, {:.3}]",
            g.get(i, 0),
            g.get(i, 1),
            g.get(i, 2),
            g.get(i, 3)
        );
    }

    // Power iteration
    let mut pi = Vector::from_slice(&[0.25, 0.25, 0.25, 0.25]);
    for iter in 0..100 {
        let pi_new = g.mul_vec(&pi);
        let diff = pi
            .iter()
            .zip(pi_new.iter())
            .map(|(&a, &b)| (a - b).abs())
            .sum::<f64>();
        pi = pi_new;
        if diff < 1e-12 {
            println!("Power iteration converged in {iter} steps");
            break;
        }
    }
    println!("PageRank: [{:.4}, {:.4}, {:.4}, {:.4}]", pi[0], pi[1], pi[2], pi[3]);

    // --- Example 3: Absorbing Markov Chain ---
    println!("\n--- Absorbing Chain: Gambler's Ruin ---");
    // States: 0 (ruin), 1, 2, ..., N-1, N (win)
    // From i: go to i+1 with prob p, i-1 with prob q=1-p
    // States 0 and N are absorbing
    let n_states = 6; // 0,1,2,3,4,5 where 0 and 5 are absorbing
    let p_win = 0.4;
    let mut p = Matrix::zeros(n_states, n_states);

    p.set(0, 0, 1.0); // Absorbing
    p.set(n_states - 1, n_states - 1, 1.0); // Absorbing
    for i in 1..n_states - 1 {
        p.set(i, i - 1, 1.0 - p_win);
        p.set(i, i + 1, p_win);
    }

    println!("Transition matrix:");
    for i in 0..n_states {
        let row: Vec<String> =
            (0..n_states).map(|j| format!("{:.1}", p.get(i, j))).collect();
        println!("  [{}]", row.join(", "));
    }

    // Canonical form: P = [I 0; R Q]
    // Q is (n-2)x(n-2) transient-to-transient
    let n_transient = n_states - 2;
    let mut q = Matrix::zeros(n_transient, n_transient);
    let mut r = Matrix::zeros(n_transient, 2); // 2 absorbing states

    for i in 0..n_transient {
        for j in 0..n_transient {
            q.set(i, j, p.get(i + 1, j + 1));
        }
        r.set(i, 0, p.get(i + 1, 0)); // To state 0 (ruin)
        r.set(i, 1, p.get(i + 1, n_states - 1)); // To state N (win)
    }

    // Fundamental matrix: N = (I - Q)^(-1)
    let i_minus_q = Matrix::identity(n_transient).sub(&q)?;
    // Use pseudo_inverse since Q might be singular
    let n_matrix = i_minus_q.pseudo_inverse()?;

    // Absorption probabilities: B = N * R
    let b = n_matrix.mul(&r)?;

    println!("\nAbsorption probabilities (starting from states 1,2,3,4):");
    println!("  State | P(ruin) | P(win)");
    for i in 0..n_transient {
        println!("  {:>5} | {:.4}    | {:.4}", i + 1, b.get(i, 0), b.get(i, 1));
    }

    // Expected steps to absorption: t = N * 1
    let ones = Vector::from(vec![1.0; n_transient]);
    let t = n_matrix.mul_vec(&ones);
    println!("\nExpected steps to absorption:");
    for i in 0..n_transient {
        println!("  From state {}: {:.2} steps", i + 1, t[i]);
    }

    // --- Example 4: Continuous-Time Markov Chain (CTMC) ---
    println!("\n--- CTMC: M/M/1 Queue (truncated) ---");
    // States: 0, 1, 2, ..., N (N=5)
    // Arrival rate lambda, service rate mu
    let lambda = 1.0;
    let mu = 1.5;
    let n = 6; // 0..5

    let mut q_matrix = Matrix::zeros(n, n);
    for i in 0..n {
        if i > 0 {
            q_matrix.set(i, i - 1, mu); // Death
        }
        if i + 1 < n {
            q_matrix.set(i, i + 1, lambda); // Birth
        }
        // Diagonal: -sum of off-diagonals
        let mut row_sum = 0.0;
        for j in 0..n {
            if i != j {
                row_sum += q_matrix.get(i, j);
            }
        }
        q_matrix.set(i, i, -row_sum);
    }

    println!("Generator matrix Q:");
    for i in 0..n {
        let row: Vec<String> =
            (0..n).map(|j| format!("{:>5.1}", q_matrix.get(i, j))).collect();
        println!("  [{}]", row.join(", "));
    }

    // Steady state: pi*Q = 0, sum(pi) = 1
    let q_t = q_matrix.transpose();
    let mut a = q_t.sub(&Matrix::identity(n))?;
    for j in 0..n {
        a.set(n - 1, j, 1.0);
    }
    let mut b = Vector::zeros(n);
    b[n - 1] = 1.0;

    let pi = a.solve(&b)?;
    println!("\nSteady-state probabilities:");
    for i in 0..n {
        println!("  pi[{i}] = {:.6}", pi[i]);
    }

    // Theoretical for M/M/1: pi[i] = (1-rho)*rho^i where rho = lambda/mu
    let rho = lambda / mu;
    println!("Theoretical (rho={rho:.3}):");
    for i in 0..n {
        let theoretical = if i == n - 1 {
            // Adjust for truncation
            let sum: f64 = (0..n - 1).map(|j| (1.0 - rho) * rho.powi(j as i32)).sum();
            1.0 - sum
        } else {
            (1.0 - rho) * rho.powi(i as i32)
        };
        println!("  pi[{i}] = {theoretical:.6}");
    }

    // Performance measures
    let l = pi.iter().enumerate().map(|(i, &p)| i as f64 * p).sum::<f64>(); // Avg customers
    let w = l / lambda; // Avg time in system (Little's law)
    println!("\nAvg customers in system L = {l:.4}");
    println!("Avg time in system W = {w:.4}");

    // Expected output:
    //
    // === Markov Chain Analysis ===
    //
    // --- Weather Model (3-state) ---
    // Transition matrix P:
    //   [0.7, 0.2, 0.1]
    //   [0.3, 0.4, 0.3]
    //   [0.2, 0.3, 0.5]
    //
    // Steady-state distribution pi: [0.4565, 0.2826, 0.2609]
    // pi*P:                     [0.4022, 0.3283, 0.3065]
    // Max diff: 5.43e-2
    //
    // Via eigendecomposition:  [0.3979, 0.3063, 0.2958]
    //
    // --- PageRank (4 pages) ---
    // Google matrix G (alpha=0.85):
    //   [0.038, 0.463, 0.463, 0.038]
    //   [0.038, 0.038, 0.463, 0.887]
    //   [0.463, 0.038, 0.038, 0.038]
    //   [0.463, 0.463, 0.038, 0.038]
    // Power iteration converged in 55 steps
    // PageRank: [0.2402, 0.3373, 0.1396, 0.2829]
    //
    // --- Absorbing Chain: Gambler's Ruin ---
    // Transition matrix:
    //   [1.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    //   [0.6, 0.0, 0.4, 0.0, 0.0, 0.0]
    //   [0.0, 0.6, 0.0, 0.4, 0.0, 0.0]
    //   [0.0, 0.0, 0.6, 0.0, 0.4, 0.0]
    //   [0.0, 0.0, 0.0, 0.6, 0.0, 0.4]
    //   [0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
    //
    // Absorption probabilities (starting from states 1,2,3,4):
    //   State | P(ruin) | P(win)
    //       1 | 0.9242    | 0.0758
    //       2 | 0.8104    | 0.1896
    //       3 | 0.6398    | 0.3602
    //       4 | 0.3839    | 0.6161
    //
    // Expected steps to absorption:
    //   From state 1: 3.10 steps
    //   From state 2: 5.26 steps
    //   From state 3: 6.00 steps
    //   From state 4: 4.60 steps
    //
    // --- CTMC: M/M/1 Queue (truncated) ---
    // Generator matrix Q:
    //   [ -1.0,   1.0,   0.0,   0.0,   0.0,   0.0]
    //   [  1.5,  -2.5,   1.0,   0.0,   0.0,   0.0]
    //   [  0.0,   1.5,  -2.5,   1.0,   0.0,   0.0]
    //   [  0.0,   0.0,   1.5,  -2.5,   1.0,   0.0]
    //   [  0.0,   0.0,   0.0,   1.5,  -2.5,   1.0]
    //   [  0.0,   0.0,   0.0,   0.0,   1.5,  -1.5]
    //
    // Steady-state probabilities:
    //   pi[0] = 0.026042
    //   pi[1] = 0.034723
    //   pi[2] = 0.063659
    //   pi[3] = 0.125388
    //   pi[4] = 0.250134
    //   pi[5] = 0.500054
    // Theoretical (rho=0.667):
    //   pi[0] = 0.333333
    //   pi[1] = 0.222222
    //   pi[2] = 0.148148
    //   pi[3] = 0.098765
    //   pi[4] = 0.065844
    //   pi[5] = 0.131687
    //
    // Avg customers in system L = 4.0390
    // Avg time in system W = 4.0390

    Ok(())
}
