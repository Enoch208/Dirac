use sails_rs::prelude::*;

pub const STARTING_RATING: i32 = 1500;
pub const RECENT_WINDOW: usize = 8;

const DEFAULT_HOUSE_EPSILON_BPS: u64 = 2000;
const DEFAULT_ELO_K: i32 = 32;
const DEFAULT_HOUSE_RATING: i32 = 1500;
const DEFAULT_LEADERBOARD_CAPACITY: u32 = 100;
const DEFAULT_MAX_RATE_AGE_BLOCKS: u32 = 1800;
const DEFAULT_RAKE_BPS: u16 = 250;
const DEFAULT_REVEAL_DEADLINE_BLOCKS: u32 = 1200;
const DEFAULT_MIN_STAKE_USD: u64 = 1;
const DEFAULT_MAX_STAKE_USD: u64 = 1000;

#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct Config {
    pub house_epsilon_bps: u64,
    pub elo_k: i32,
    pub house_rating: i32,
    pub leaderboard_capacity: u32,
    pub max_rate_age_blocks: u32,
    pub rake_bps: u16,
    pub reveal_deadline_blocks: u32,
    pub min_stake_usd: u64,
    pub max_stake_usd: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            house_epsilon_bps: DEFAULT_HOUSE_EPSILON_BPS,
            elo_k: DEFAULT_ELO_K,
            house_rating: DEFAULT_HOUSE_RATING,
            leaderboard_capacity: DEFAULT_LEADERBOARD_CAPACITY,
            max_rate_age_blocks: DEFAULT_MAX_RATE_AGE_BLOCKS,
            rake_bps: DEFAULT_RAKE_BPS,
            reveal_deadline_blocks: DEFAULT_REVEAL_DEADLINE_BLOCKS,
            min_stake_usd: DEFAULT_MIN_STAKE_USD,
            max_stake_usd: DEFAULT_MAX_STAKE_USD,
        }
    }
}
