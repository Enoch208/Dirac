use sha2::{Digest, Sha256};

pub trait Rng {
    fn next_u64(&mut self) -> u64;

    fn next_bounded(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        self.next_u64() % bound
    }
}

pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl Rng for SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

pub fn seed_from(block_timestamp: u64, match_id: u64, player: &[u8]) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(block_timestamp.to_le_bytes());
    hasher.update(match_id.to_le_bytes());
    hasher.update(player);
    let digest = hasher.finalize();
    let mut head = [0u8; 8];
    head.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(head)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix_is_deterministic_for_same_seed() {
        let mut a = SplitMix64::new(123);
        let mut b = SplitMix64::new(123);
        for _ in 0..8 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn splitmix_differs_across_calls() {
        let mut r = SplitMix64::new(123);
        let first = r.next_u64();
        let second = r.next_u64();
        assert_ne!(first, second);
    }

    #[test]
    fn next_bounded_stays_in_range() {
        let mut r = SplitMix64::new(999);
        for _ in 0..50 {
            assert!(r.next_bounded(3) < 3);
        }
    }

    #[test]
    fn seed_changes_with_player() {
        assert_ne!(seed_from(100, 1, &[1, 2, 3]), seed_from(100, 1, &[4, 5, 6]));
    }
}
