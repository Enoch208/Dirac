use crate::rps::Move;
use sha2::{Digest, Sha256};

pub type Commit = [u8; 32];
pub type Salt = [u8; 32];

pub fn commit_hash(player_move: Move, salt: &Salt) -> Commit {
    let mut hasher = Sha256::new();
    hasher.update([player_move.as_byte()]);
    hasher.update(salt);
    hasher.finalize().into()
}

pub fn verify_commit(player_move: Move, salt: &Salt, stored: &Commit) -> bool {
    commit_hash(player_move, salt) == *stored
}

#[cfg(test)]
mod tests {
    use super::*;

    const SALT: Salt = [7u8; 32];

    #[test]
    fn valid_reveal_verifies() {
        let c = commit_hash(Move::Rock, &SALT);
        assert!(verify_commit(Move::Rock, &SALT, &c));
    }

    #[test]
    fn wrong_move_fails() {
        let c = commit_hash(Move::Rock, &SALT);
        assert!(!verify_commit(Move::Paper, &SALT, &c));
    }

    #[test]
    fn wrong_salt_fails() {
        let c = commit_hash(Move::Rock, &SALT);
        assert!(!verify_commit(Move::Rock, &[9u8; 32], &c));
    }

    #[test]
    fn distinct_moves_produce_distinct_commits() {
        assert_ne!(commit_hash(Move::Rock, &SALT), commit_hash(Move::Paper, &SALT));
        assert_ne!(commit_hash(Move::Paper, &SALT), commit_hash(Move::Scissors, &SALT));
    }

    #[test]
    fn hash_is_deterministic() {
        assert_eq!(commit_hash(Move::Scissors, &SALT), commit_hash(Move::Scissors, &SALT));
    }
}
