use crate::config::{Config, STARTING_RATING};
use crate::types::Move;
use dirac_logic::leaderboard::Entry;
use dirac_logic::rps::Move as LogicMove;
use sails_rs::collections::BTreeMap;
use sails_rs::prelude::*;

pub type Commit = [u8; 32];

pub struct PlayerRecord {
    pub rating: i32,
    pub games: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub move_counts: [u32; 3],
    pub recent: Vec<LogicMove>,
}

impl PlayerRecord {
    pub fn new() -> Self {
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

#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct PlayerStats {
    pub rating: i32,
    pub games: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

impl From<&PlayerRecord> for PlayerStats {
    fn from(record: &PlayerRecord) -> Self {
        Self {
            rating: record.rating,
            games: record.games,
            wins: record.wins,
            losses: record.losses,
            draws: record.draws,
        }
    }
}

#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub enum MatchState {
    AwaitingOpponent,
    AwaitingReveal,
    Settled,
    Refunded,
}

pub struct Match {
    pub challenger: ActorId,
    pub opponent: ActorId,
    pub challenger_commit: Commit,
    pub opponent_commit: Option<Commit>,
    pub challenger_reveal: Option<LogicMove>,
    pub opponent_reveal: Option<LogicMove>,
    pub stake_vara: u128,
    pub deadline_block: u32,
    pub state: MatchState,
}

#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct MatchView {
    pub challenger: ActorId,
    pub opponent: ActorId,
    pub stake_vara: u128,
    pub deadline_block: u32,
    pub state: MatchState,
}

impl From<&Match> for MatchView {
    fn from(value: &Match) -> Self {
        Self {
            challenger: value.challenger,
            opponent: value.opponent,
            stake_vara: value.stake_vara,
            deadline_block: value.deadline_block,
            state: value.state,
        }
    }
}

#[derive(Default)]
pub struct VaraUsdRate {
    pub rate: u128,
    pub set_at_block: u32,
}

#[derive(Default)]
pub struct GameState {
    pub operator: ActorId,
    pub paused: bool,
    pub config: Config,
    pub rate: VaraUsdRate,
    pub pot: u128,
    pub next_match_id: u64,
    pub players: BTreeMap<ActorId, PlayerRecord>,
    pub leaderboard: Vec<Entry>,
    pub matches: BTreeMap<u64, Match>,
}

impl GameState {
    pub fn record_mut(&mut self, who: ActorId) -> &mut PlayerRecord {
        self.players.entry(who).or_insert_with(PlayerRecord::new)
    }
}
