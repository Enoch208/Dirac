use colosseum_logic::rps::{Move as LogicMove, Outcome as LogicOutcome};
use sails_rs::prelude::*;

#[derive(Encode, Decode, TypeInfo, Clone, Copy, PartialEq, Eq, Debug)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Move {
    Rock,
    Paper,
    Scissors,
}

#[derive(Encode, Decode, TypeInfo, Clone, Copy, PartialEq, Eq, Debug)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum Outcome {
    Win,
    Loss,
    Draw,
}

#[derive(Encode, Decode, TypeInfo, Clone, Copy, PartialEq, Eq, Debug)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct RoundResult {
    pub player_move: Move,
    pub house_move: Move,
    pub outcome: Outcome,
    pub new_rating: i32,
}

impl From<Move> for LogicMove {
    fn from(value: Move) -> Self {
        match value {
            Move::Rock => LogicMove::Rock,
            Move::Paper => LogicMove::Paper,
            Move::Scissors => LogicMove::Scissors,
        }
    }
}

impl From<LogicMove> for Move {
    fn from(value: LogicMove) -> Self {
        match value {
            LogicMove::Rock => Move::Rock,
            LogicMove::Paper => Move::Paper,
            LogicMove::Scissors => Move::Scissors,
        }
    }
}

impl From<LogicOutcome> for Outcome {
    fn from(value: LogicOutcome) -> Self {
        match value {
            LogicOutcome::Win => Outcome::Win,
            LogicOutcome::Loss => Outcome::Loss,
            LogicOutcome::Draw => Outcome::Draw,
        }
    }
}
