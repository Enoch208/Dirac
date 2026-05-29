use crate::rps::Move;
use sha2::{Digest, Sha256};

pub type Commit = [u8; 32];
pub type Salt = [u8; 32];

pub fn commit_hash(match_id: u64, player_move: Move, salt: &Salt) -> Commit {
    let mut hasher = Sha256::new();
    hasher.update(match_id.to_le_bytes());
    hasher.update([player_move.as_byte()]);
    hasher.update(salt);
    hasher.finalize().into()
}

pub fn verify_commit(match_id: u64, player_move: Move, salt: &Salt, stored: &Commit) -> bool {
    commit_hash(match_id, player_move, salt) == *stored
}

#[cfg(test)]
mod tests {
    use super::*;

    const SALT: Salt = [7u8; 32];

    #[test]
    fn valid_reveal_verifies() {
        let c = commit_hash(42, Move::Rock, &SALT);
        assert!(verify_commit(42, Move::Rock, &SALT, &c));
    }

    #[test]
    fn wrong_move_fails() {
        let c = commit_hash(42, Move::Rock, &SALT);
        assert!(!verify_commit(42, Move::Paper, &SALT, &c));
    }

    #[test]
    fn wrong_salt_fails() {
        let c = commit_hash(42, Move::Rock, &SALT);
        assert!(!verify_commit(42, Move::Rock, &[9u8; 32], &c));
    }

    #[test]
    fn cross_match_replay_fails() {
        let c = commit_hash(42, Move::Rock, &SALT);
        assert!(!verify_commit(43, Move::Rock, &SALT, &c));
    }

    #[test]
    fn hash_is_deterministic() {
        assert_eq!(commit_hash(1, Move::Scissors, &SALT), commit_hash(1, Move::Scissors, &SALT));
    }
}
