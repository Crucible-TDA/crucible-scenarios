//! Deterministic randomness for scenario generation and replay.
//!
//! Every randomized scenario declares a [`Seed`]. The seed is reported with a
//! scenario result, so any failure can be replayed with
//! `crucible-scenarios replay --scenario <ID> --seed <SEED>` and reconstruct
//! the same fixtures, operation sequences, and randomized values.
//!
//! [`DeterministicRng`] is a small, dependency-free splitmix64 generator:
//! splitmix64 is a well-known bijective integer mixer with good statistical
//! behavior, trivially portable across platforms, and — unlike
//! `rand`/`ChaCha` — introduces no external dependency. Seeding it with a
//! [`Seed`] yields an identical stream on every machine and every Rust
//! version, which is exactly what reproducible testing requires.
//!
//! Seeds never encode secrets: they are sequence selectors, not key material.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::{Error, Result};

/// A 64-bit scenario seed, displayed and serialized as 16 lowercase hex
/// characters so seeds can be copied between the CLI, reports, and CI logs
/// without ambiguity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Seed(u64);

impl Seed {
    /// The canonical zero seed, useful for fully deterministic scenarios that
    /// never consult randomness.
    pub const ZERO: Seed = Seed(0);

    /// Wrap a raw 64-bit value as a seed.
    pub const fn new(value: u64) -> Self {
        Seed(value)
    }

    /// Parse a 1–16 hex character seed (`0x` prefix optional).
    pub fn from_hex(input: &str) -> Result<Self> {
        let hex = input.strip_prefix("0x").unwrap_or(input);
        if hex.is_empty() || hex.len() > 16 {
            return Err(Error::InvalidSeed(
                input.to_string(),
                "expected between 1 and 16 hexadecimal characters".to_string(),
            ));
        }
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::InvalidSeed(
                input.to_string(),
                "contains non-hexadecimal characters".to_string(),
            ));
        }
        let value = u64::from_str_radix(hex, 16)
            .map_err(|e| Error::InvalidSeed(input.to_string(), e.to_string()))?;
        Ok(Seed(value))
    }

    /// Raw 64-bit value.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Canonical 16-character lowercase hex representation.
    pub fn to_hex(self) -> String {
        format!("{:016x}", self.0)
    }

    /// Derive a child seed for a numbered sub-stream (e.g. per-fixture or
    /// per-operation-sequence randomness). Child derivation uses splitmix64
    /// so different indices always yield different, well-mixed seeds.
    pub fn child(self, index: u64) -> Self {
        let mut state = self.0 ^ 0x9E37_79B9_7F4A_7C15;
        let mut mixed = splitmix64(&mut state);
        // Fold the index into the state so children also differ across
        // parents that are close together.
        state = state.wrapping_add(index.wrapping_mul(0xBF58_476D_1CE4_E5B9));
        mixed ^= splitmix64(&mut state);
        Seed(mixed)
    }
}

impl Default for Seed {
    fn default() -> Self {
        Seed::ZERO
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Same representation as Display: seeds are public metadata and safe
        // to print, but a single canonical form avoids ambiguity in logs.
        f.write_str(&self.to_hex())
    }
}

impl FromStr for Seed {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Seed::from_hex(s)
    }
}

impl Serialize for Seed {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Seed {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Seed::from_hex(&s).map_err(serde::de::Error::custom)
    }
}

/// Deterministic, dependency-free pseudorandom generator (splitmix64).
///
/// The generator is a plain value type: clone it to branch a random stream,
/// or seed one per consumer from [`Seed::child`] to keep sub-streams isolated
/// and reproducible.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    /// Create a generator from a [`Seed`].
    pub const fn new(seed: Seed) -> Self {
        DeterministicRng { state: seed.get() }
    }

    /// Next raw 64-bit output (splitmix64 finalizer applied to advancing state).
    pub fn next_u64(&mut self) -> u64 {
        splitmix64(&mut self.state)
    }

    /// Next value in `0..=u32::MAX`.
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform value in `0..bound` for `bound > 0`, using Lemire's
    /// multiply-shift method: exact for every bound, no division needed.
    pub fn next_below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0, "next_below requires a positive bound");
        // Lemire's method rejects the multiply only when the leftover would
        // bias the result (that is, when `low < 2^64 mod bound`).
        let mut x = self.next_u64();
        let mut m = (x as u128).wrapping_mul(bound as u128);
        let mut low = m as u64;
        if low < bound {
            let threshold = bound.wrapping_neg() % bound;
            while low < threshold {
                x = self.next_u64();
                m = (x as u128).wrapping_mul(bound as u128);
                low = m as u64;
            }
        }
        (m >> 64) as u64
    }

    /// Uniform value in the inclusive range `lo..=hi`.
    pub fn gen_range(&mut self, lo: u64, hi: u64) -> u64 {
        debug_assert!(lo <= hi, "gen_range requires lo <= hi");
        lo + self.next_below(hi - lo + 1)
    }

    /// Bernoulli draw with the given probability.
    pub fn next_bool(&mut self, probability: f64) -> bool {
        debug_assert!((0.0..=1.0).contains(&probability));
        // 53 bits of mantissa give exact f64 conversion of the fraction.
        let r = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        r < probability
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_hex_round_trip() {
        for raw in [0u64, 1, 0xDEAD_BEEF, u64::MAX] {
            let seed = Seed::new(raw);
            assert_eq!(Seed::from_hex(&seed.to_hex()).unwrap(), seed);
        }
    }

    #[test]
    fn seed_from_hex_accepts_prefix_and_rejects_garbage() {
        assert_eq!(Seed::from_hex("0x0000000000000001").unwrap(), Seed::new(1));
        assert!(Seed::from_hex("").is_err());
        assert!(Seed::from_hex("zz").is_err());
        assert!(Seed::from_hex("0123456789abcdef0").is_err()); // 17 chars
    }

    #[test]
    fn same_seed_same_stream() {
        let mut a = DeterministicRng::new(Seed::new(42));
        let mut b = DeterministicRng::new(Seed::new(42));
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = DeterministicRng::new(Seed::new(1));
        let mut b = DeterministicRng::new(Seed::new(2));
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn child_seeds_are_stable_and_distinct() {
        let parent = Seed::new(7);
        let c1 = parent.child(0);
        let c2 = parent.child(1);
        assert_eq!(parent.child(0), c1);
        assert_ne!(c1, c2);
        assert_ne!(parent, c1);
    }

    #[test]
    fn next_below_respects_bound() {
        let mut rng = DeterministicRng::new(Seed::new(99));
        for _ in 0..10_000 {
            let v = rng.next_below(7);
            assert!(v < 7);
        }
        assert_eq!(rng.next_below(1), 0);
    }

    #[test]
    fn gen_range_is_inclusive() {
        let mut rng = DeterministicRng::new(Seed::new(5));
        for _ in 0..10_000 {
            let v = rng.gen_range(3, 5);
            assert!((3..=5).contains(&v));
        }
    }

    #[test]
    fn serde_round_trip_uses_hex_string() {
        let seed = Seed::new(0xABCD);
        let json = serde_json::to_string(&seed).unwrap();
        assert_eq!(json, "\"000000000000abcd\"");
        assert_eq!(serde_json::from_str::<Seed>(&json).unwrap(), seed);
    }
}
