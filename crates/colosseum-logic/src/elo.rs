use crate::rps::Outcome;

pub const SCORE_SCALE: i32 = 1000;
pub const SCORE_WIN: i32 = 1000;
pub const SCORE_DRAW: i32 = 500;
pub const SCORE_LOSS: i32 = 0;

const RATING_DIFF_CLAMP: i32 = 800;
const TABLE_STEP: i32 = 25;
const EXPECTED_SCORE_TABLE: [i32; 33] = [
    500, 464, 429, 394, 360, 327, 297, 267, 240, 215, 192, 170, 151, 133, 118, 104, 91, 80, 70, 61,
    53, 46, 40, 35, 31, 27, 23, 20, 17, 15, 13, 11, 10,
];

pub fn score_milli(outcome: Outcome) -> i32 {
    match outcome {
        Outcome::Win => SCORE_WIN,
        Outcome::Draw => SCORE_DRAW,
        Outcome::Loss => SCORE_LOSS,
    }
}

pub fn expected_score_milli(player: i32, opponent: i32) -> i32 {
    let diff = (opponent - player).clamp(-RATING_DIFF_CLAMP, RATING_DIFF_CLAMP);
    if diff >= 0 {
        interpolate_expected(diff)
    } else {
        SCORE_SCALE - interpolate_expected(-diff)
    }
}

fn interpolate_expected(diff: i32) -> i32 {
    let index = (diff / TABLE_STEP) as usize;
    let remainder = diff % TABLE_STEP;
    let low = EXPECTED_SCORE_TABLE[index];
    if remainder == 0 || index + 1 == EXPECTED_SCORE_TABLE.len() {
        return low;
    }
    let high = EXPECTED_SCORE_TABLE[index + 1];
    low + round_div((high - low) * remainder, TABLE_STEP)
}

pub fn rating_delta(player: i32, opponent: i32, score_milli: i32, k: i32) -> i32 {
    let expected = expected_score_milli(player, opponent);
    round_div(k * (score_milli - expected), SCORE_SCALE)
}

fn round_div(numerator: i32, denominator: i32) -> i32 {
    let half = denominator / 2;
    if numerator >= 0 {
        (numerator + half) / denominator
    } else {
        (numerator - half) / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const K: i32 = 32;

    #[test]
    fn equal_ratings_expect_half() {
        assert_eq!(expected_score_milli(1500, 1500), SCORE_DRAW);
    }

    #[test]
    fn expected_scores_are_complementary() {
        for (a, b) in [(1500, 1500), (1600, 1400), (1200, 1900), (1000, 1001)] {
            let sum = expected_score_milli(a, b) + expected_score_milli(b, a);
            assert!((sum - SCORE_SCALE).abs() <= 1, "{a} vs {b}: sum={sum}");
        }
    }

    #[test]
    fn higher_rated_player_is_favored() {
        assert!(expected_score_milli(1700, 1500) > SCORE_DRAW);
        assert!(expected_score_milli(1300, 1500) < SCORE_DRAW);
    }

    #[test]
    fn win_gains_loss_drops_draw_is_small() {
        assert!(rating_delta(1500, 1500, SCORE_WIN, K) > 0);
        assert!(rating_delta(1500, 1500, SCORE_LOSS, K) < 0);
        assert_eq!(rating_delta(1500, 1500, SCORE_DRAW, K), 0);
    }

    #[test]
    fn win_gain_shrinks_as_player_outranks_fixed_house() {
        let house = 1500;
        let low = rating_delta(1500, house, SCORE_WIN, K);
        let mid = rating_delta(1800, house, SCORE_WIN, K);
        let high = rating_delta(2100, house, SCORE_WIN, K);
        assert!(low > mid && mid > high, "{low} {mid} {high}");
        assert!(high >= 0);
    }

    #[test]
    fn pvp_exchange_is_approximately_zero_sum() {
        let (a, b) = (1640, 1480);
        let da = rating_delta(a, b, SCORE_WIN, K);
        let db = rating_delta(b, a, SCORE_LOSS, K);
        assert!((da + db).abs() <= 1, "da={da} db={db}");
    }
}
