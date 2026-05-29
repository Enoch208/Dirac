#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Move {
    Rock,
    Paper,
    Scissors,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Win,
    Loss,
    Draw,
}

impl Move {
    pub fn as_byte(self) -> u8 {
        match self {
            Move::Rock => 0,
            Move::Paper => 1,
            Move::Scissors => 2,
        }
    }
}

pub fn resolve(player: Move, opponent: Move) -> Outcome {
    match (player, opponent) {
        (Move::Rock, Move::Scissors)
        | (Move::Paper, Move::Rock)
        | (Move::Scissors, Move::Paper) => Outcome::Win,
        (a, b) if a == b => Outcome::Draw,
        _ => Outcome::Loss,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Move; 3] = [Move::Rock, Move::Paper, Move::Scissors];

    fn beats(a: Move, b: Move) -> bool {
        matches!(
            (a, b),
            (Move::Rock, Move::Scissors)
                | (Move::Paper, Move::Rock)
                | (Move::Scissors, Move::Paper)
        )
    }

    #[test]
    fn draws_on_identical_moves() {
        for m in ALL {
            assert_eq!(resolve(m, m), Outcome::Draw);
        }
    }

    #[test]
    fn full_truth_table_matches_rps_cycle() {
        for player in ALL {
            for opponent in ALL {
                let expected = if player == opponent {
                    Outcome::Draw
                } else if beats(player, opponent) {
                    Outcome::Win
                } else {
                    Outcome::Loss
                };
                assert_eq!(resolve(player, opponent), expected, "{player:?} vs {opponent:?}");
            }
        }
    }
}
