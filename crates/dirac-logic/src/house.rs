use crate::rng::Rng;
use crate::rps::Move;

const MOVES: [Move; 3] = [Move::Rock, Move::Paper, Move::Scissors];

const EPSILON_DENOMINATOR: u64 = 10_000;

pub fn counter(predicted: Move) -> Move {
    match predicted {
        Move::Rock => Move::Paper,
        Move::Paper => Move::Scissors,
        Move::Scissors => Move::Rock,
    }
}

pub fn predict(move_counts: [u32; 3], recent: &[Move]) -> Move {
    let mut score = [0i64; 3];
    for (slot, count) in score.iter_mut().zip(move_counts) {
        *slot = count as i64;
    }
    for (position, played) in recent.iter().enumerate() {
        score[played.as_byte() as usize] += position as i64 + 1;
    }
    let mut best = 0usize;
    for candidate in 1..MOVES.len() {
        if score[candidate] > score[best] {
            best = candidate;
        }
    }
    MOVES[best]
}

pub fn house_move(
    move_counts: [u32; 3],
    recent: &[Move],
    epsilon_bps: u64,
    rng: &mut impl Rng,
) -> Move {
    let total: u32 = move_counts.iter().sum();
    if total == 0 {
        return random_move(rng);
    }
    if rng.next_bounded(EPSILON_DENOMINATOR) < epsilon_bps {
        return random_move(rng);
    }
    counter(predict(move_counts, recent))
}

fn random_move(rng: &mut impl Rng) -> Move {
    MOVES[rng.next_bounded(MOVES.len() as u64) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rps::{resolve, Outcome};

    struct ScriptedRng {
        values: alloc::vec::Vec<u64>,
        index: usize,
    }

    impl ScriptedRng {
        fn new(values: &[u64]) -> Self {
            Self { values: values.to_vec(), index: 0 }
        }
    }

    impl Rng for ScriptedRng {
        fn next_u64(&mut self) -> u64 {
            let v = self.values[self.index % self.values.len()];
            self.index += 1;
            v
        }
    }

    const NO_EPSILON: u64 = 0;
    const ALWAYS_EPSILON: u64 = 10_000;

    #[test]
    fn counter_beats_the_predicted_move() {
        for m in MOVES {
            assert_eq!(resolve(counter(m), m), Outcome::Win);
        }
    }

    #[test]
    fn predict_follows_dominant_frequency() {
        assert_eq!(predict([10, 0, 0], &[]), Move::Rock);
        assert_eq!(predict([0, 10, 0], &[]), Move::Paper);
        assert_eq!(predict([0, 0, 10], &[]), Move::Scissors);
    }

    #[test]
    fn recent_streak_overrides_long_run_frequency() {
        let recent = [Move::Paper, Move::Paper, Move::Paper, Move::Paper, Move::Paper];
        assert_eq!(predict([5, 0, 0], &recent), Move::Paper);
    }

    #[test]
    fn no_history_plays_rng_move() {
        let mut rng = ScriptedRng::new(&[2]);
        assert_eq!(house_move([0, 0, 0], &[], NO_EPSILON, &mut rng), Move::Scissors);
    }

    #[test]
    fn epsilon_branch_plays_random_not_counter() {
        let mut rng = ScriptedRng::new(&[0, 0]);
        let played = house_move([10, 0, 0], &[], ALWAYS_EPSILON, &mut rng);
        assert_eq!(played, Move::Rock);
    }

    #[test]
    fn non_epsilon_branch_counters_prediction() {
        let mut rng = ScriptedRng::new(&[9999]);
        let played = house_move([10, 0, 0], &[], 2000, &mut rng);
        assert_eq!(played, Move::Paper);
    }

    #[test]
    fn one_track_player_loses_majority_over_time() {
        use crate::rng::SplitMix64;
        let mut rng = SplitMix64::new(0xC0FFEE);
        let mut counts = [0u32; 3];
        let mut recent: alloc::vec::Vec<Move> = alloc::vec::Vec::new();
        let mut house_wins = 0;
        let rounds = 1000;
        for _ in 0..rounds {
            let house = house_move(counts, &recent, 2000, &mut rng);
            if resolve(Move::Rock, house) == Outcome::Loss {
                house_wins += 1;
            }
            counts[Move::Rock.as_byte() as usize] += 1;
            recent.push(Move::Rock);
            if recent.len() > 8 {
                recent.remove(0);
            }
        }
        assert!(house_wins * 100 / rounds >= 70, "house won {house_wins}/{rounds}");
    }
}
