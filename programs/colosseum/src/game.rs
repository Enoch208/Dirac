use colosseum_logic::elo::{rating_delta, score_milli};
use colosseum_logic::house::house_move;
use colosseum_logic::leaderboard::{update_leaderboard, Entry};
use colosseum_logic::rng::{seed_from, SplitMix64};
use colosseum_logic::rps::{resolve, Move as LogicMove, Outcome as LogicOutcome};
use sails_rs::cell::RefCell;
use sails_rs::collections::BTreeMap;
use sails_rs::gstd::{exec, msg};
use sails_rs::prelude::*;

const STARTING_RATING: i32 = 1500;
const HOUSE_RATING: i32 = 1500;
const ELO_K: i32 = 32;
const HOUSE_EPSILON_BPS: u64 = 2000;
const LEADERBOARD_CAPACITY: usize = 100;
const RECENT_WINDOW: usize = 8;

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

pub struct PlayerRecord {
    rating: i32,
    games: u32,
    wins: u32,
    losses: u32,
    draws: u32,
    move_counts: [u32; 3],
    recent: Vec<LogicMove>,
}

impl PlayerRecord {
    fn new() -> Self {
        Self {
            rating: STARTING_RATING,
            games: 0,
            wins: 0,
            losses: 0,
            draws: 0,
            move_counts: [0; 3],
            recent: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct GameState {
    players: BTreeMap<ActorId, PlayerRecord>,
    leaderboard: Vec<Entry>,
    next_match_id: u64,
}

#[event]
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum GameEvents {
    MatchPlayed {
        match_id: u64,
        player: ActorId,
        player_move: Move,
        house_move: Move,
        outcome: Outcome,
        new_rating: i32,
    },
    NewChampion {
        player: ActorId,
        rating: i32,
    },
}

pub struct GameService<'a> {
    state: &'a RefCell<GameState>,
}

impl<'a> GameService<'a> {
    pub fn new(state: &'a RefCell<GameState>) -> Self {
        Self { state }
    }
}

#[service(events = GameEvents)]
impl GameService<'_> {
    #[export]
    pub fn play(&mut self, player_move: Move) -> RoundResult {
        let caller = msg::source();
        let timestamp = exec::block_timestamp();
        let logic_move: LogicMove = player_move.into();
        let id_bytes = caller.into_bytes();

        let mut state = self.state.borrow_mut();
        let match_id = state.next_match_id;
        state.next_match_id = match_id.wrapping_add(1);

        let record = state.players.entry(caller).or_insert_with(PlayerRecord::new);
        let mut rng = SplitMix64::new(seed_from(timestamp, match_id, &id_bytes));
        let house = house_move(record.move_counts, &record.recent, HOUSE_EPSILON_BPS, &mut rng);
        let outcome = resolve(logic_move, house);
        let delta = rating_delta(record.rating, HOUSE_RATING, score_milli(outcome), ELO_K);

        record.rating = record.rating.saturating_add(delta);
        record.games = record.games.saturating_add(1);
        match outcome {
            LogicOutcome::Win => record.wins = record.wins.saturating_add(1),
            LogicOutcome::Loss => record.losses = record.losses.saturating_add(1),
            LogicOutcome::Draw => record.draws = record.draws.saturating_add(1),
        }
        record.move_counts[logic_move.as_byte() as usize] += 1;
        record.recent.push(logic_move);
        if record.recent.len() > RECENT_WINDOW {
            record.recent.remove(0);
        }
        let new_rating = record.rating;

        let champion_changed =
            update_leaderboard(&mut state.leaderboard, id_bytes, new_rating, LEADERBOARD_CAPACITY);
        drop(state);

        let _ = self.emit_event(GameEvents::MatchPlayed {
            match_id,
            player: caller,
            player_move,
            house_move: house.into(),
            outcome: outcome.into(),
            new_rating,
        });
        if champion_changed {
            let _ = self.emit_event(GameEvents::NewChampion { player: caller, rating: new_rating });
        }

        RoundResult {
            player_move,
            house_move: house.into(),
            outcome: outcome.into(),
            new_rating,
        }
    }
}
