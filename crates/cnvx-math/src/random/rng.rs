/// A Xoshiro256++ pseudo-random number generator.
///
/// Construct with a fixed [`Rng::new`] seed for reproducible runs (tests,
/// deterministic experiments), or [`Rng::from_entropy`] for a
/// time-derived seed.
#[derive(Debug, Clone, PartialEq)]
pub struct Rng {
    state: [u64; 4],
}

#[inline]
fn rotl(x: u64, k: u32) -> u64 {
    x.rotate_left(k)
}

/// Expands a single `u64` seed into a well-mixed `u64` stream, used to
/// initialize Xoshiro256++'s 256-bit state from a single seed value
/// without leaving it partly zero.
fn splitmix64(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Rng {
    /// Creates a generator from a fixed 64-bit seed. Two `Rng`s created
    /// with the same seed produce the same sequence of samples.
    pub fn new(seed: u64) -> Self {
        let mut sm_state = seed;
        let state = [
            splitmix64(&mut sm_state),
            splitmix64(&mut sm_state),
            splitmix64(&mut sm_state),
            splitmix64(&mut sm_state),
        ];
        Self { state }
    }

    /// Creates a generator seeded from the system clock. Not reproducible
    /// across runs; use [`Rng::new`] with a fixed seed for that.
    pub fn from_entropy() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::new(nanos ^ count.wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }

    /// The next raw 64-bit output.
    pub fn next_u64(&mut self) -> u64 {
        let s = &mut self.state;
        let result = rotl(s[1].wrapping_mul(5), 7).wrapping_mul(9);
        let t = s[1] << 17;

        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = rotl(s[3], 45);

        result
    }

    /// The next 32-bit output (the high bits of [`Rng::next_u64`], which
    /// tend to have better statistical quality than the low bits for
    /// generators like this one).
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// A uniformly distributed `f64` in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        // Take the top 53 bits (f64's mantissa width) for a value that's
        // uniform over the representable doubles in [0, 1).
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_reproducible() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn next_f64_is_in_unit_interval() {
        let mut rng = Rng::new(7);
        for _ in 0..10_000 {
            let x = rng.next_f64();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn next_f64_mean_is_roughly_a_half() {
        let mut rng = Rng::new(123);
        let n = 100_000;
        let sum: f64 = (0..n).map(|_| rng.next_f64()).sum();
        let mean = sum / n as f64;
        assert!((mean - 0.5).abs() < 0.01, "mean = {mean}");
    }
}
