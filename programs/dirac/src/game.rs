use crate::config::{Config, RECENT_WINDOW};
use crate::state::{Commit, GameState, Match, MatchState, MatchView, PlayerRecord, PlayerStats};
use crate::types::{Move, Outcome, RoundResult};
use dirac_logic::commit::verify_commit;
use dirac_logic::elo::{rating_delta, score_milli};
use dirac_logic::escrow::{decisive_payout, draw_payout, stake_to_vara};
use dirac_logic::house::house_move;
use dirac_logic::leaderboard::update_leaderboard;
use dirac_logic::rng::{seed_from, SplitMix64};
use dirac_logic::rps::{resolve, Move as LogicMove, Outcome as LogicOutcome};
use sails_rs::cell::RefCell;
use sails_rs::gstd::{exec, msg};
use sails_rs::prelude::*;

#[derive(Encode, Decode, TypeInfo, Clone, Copy, Debug, PartialEq, Eq)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
pub struct LeaderboardEntry {
    pub player: ActorId,
    pub rating: i32,
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
    ChallengeOpened {
        match_id: u64,
        challenger: ActorId,
        opponent: ActorId,
        stake_vara: u128,
        deadline_block: u32,
    },
    ChallengeAccepted {
        match_id: u64,
        deadline_block: u32,
    },
    PvpResolved {
        match_id: u64,
        winner: Option<ActorId>,
        challenger_move: Move,
        opponent_move: Move,
        payout: u128,
    },
    MatchForfeited {
        match_id: u64,
        winner: ActorId,
        loser: ActorId,
    },
    MatchRefunded {
        match_id: u64,
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
    #[export(unwrap_result)]
    pub fn play(&mut self, player_move: Move) -> Result<RoundResult, String> {
        let caller = msg::source();
        let timestamp = exec::block_timestamp();
        let logic_move: LogicMove = player_move.into();
        let id_bytes = caller.into_bytes();

        let mut state = self.state.borrow_mut();
        if state.paused {
            return Err("arena is paused".into());
        }
        let config = state.config;
        let match_id = state.next_match_id;
        state.next_match_id = match_id.wrapping_add(1);

        let record = state.record_mut(caller);
        let mut rng = SplitMix64::new(seed_from(timestamp, match_id, &id_bytes));
        let house = house_move(record.move_counts, &record.recent, config.house_epsilon_bps, &mut rng);
        let outcome = resolve(logic_move, house);
        let delta = rating_delta(record.rating, config.house_rating, score_milli(outcome), config.elo_k);

        record.rating = record.rating.saturating_add(delta);
        record.games = record.games.saturating_add(1);
        bump_outcome(record, outcome);
        record.move_counts[logic_move.as_byte() as usize] += 1;
        record.recent.push(logic_move);
        if record.recent.len() > RECENT_WINDOW {
            record.recent.remove(0);
        }
        let new_rating = record.rating;

        let capacity = config.leaderboard_capacity as usize;
        let champion_changed = update_leaderboard(&mut state.leaderboard, id_bytes, new_rating, capacity);
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

        Ok(RoundResult { player_move, house_move: house.into(), outcome: outcome.into(), new_rating })
    }

    #[export(unwrap_result)]
    pub fn challenge(&mut self, opponent: ActorId, move_commit: Commit, stake_usd: u64) -> Result<u64, String> {
        let caller = msg::source();
        let paid = Syscall::message_value();

        let mut state = self.state.borrow_mut();
        if state.paused {
            return Err("arena is paused".into());
        }
        if opponent == caller || opponent == ActorId::zero() {
            return Err("invalid opponent".into());
        }
        let config = state.config;
        if stake_usd < config.min_stake_usd || stake_usd > config.max_stake_usd {
            return Err("stake out of bounds".into());
        }
        let stake_vara = self.fresh_stake_vara(&state, stake_usd)?;
        if paid < stake_vara {
            return Err("insufficient stake escrowed".into());
        }

        let match_id = state.next_match_id;
        state.next_match_id = match_id.wrapping_add(1);
        let deadline_block = exec::block_height().saturating_add(config.reveal_deadline_blocks);
        state.matches.insert(
            match_id,
            Match {
                challenger: caller,
                opponent,
                challenger_commit: move_commit,
                opponent_commit: None,
                challenger_reveal: None,
                opponent_reveal: None,
                stake_vara,
                deadline_block,
                state: MatchState::AwaitingOpponent,
            },
        );
        drop(state);

        refund(caller, paid.saturating_sub(stake_vara));
        let _ = self.emit_event(GameEvents::ChallengeOpened {
            match_id,
            challenger: caller,
            opponent,
            stake_vara,
            deadline_block,
        });
        Ok(match_id)
    }

    #[export(unwrap_result)]
    pub fn accept_challenge(&mut self, match_id: u64, move_commit: Commit) -> Result<(), String> {
        let caller = msg::source();
        let paid = Syscall::message_value();

        let mut state = self.state.borrow_mut();
        if state.paused {
            return Err("arena is paused".into());
        }
        let config = state.config;
        let deadline_block = exec::block_height().saturating_add(config.reveal_deadline_blocks);

        let stake_vara = {
            let game_match = state.matches.get(&match_id).ok_or("no such match")?;
            if game_match.state != MatchState::AwaitingOpponent {
                return Err("match not open".into());
            }
            if game_match.opponent != caller {
                return Err("not the challenged opponent".into());
            }
            game_match.stake_vara
        };
        if paid < stake_vara {
            return Err("insufficient stake escrowed".into());
        }
        if let Some(game_match) = state.matches.get_mut(&match_id) {
            game_match.opponent_commit = Some(move_commit);
            game_match.deadline_block = deadline_block;
            game_match.state = MatchState::AwaitingReveal;
        }
        drop(state);

        refund(caller, paid.saturating_sub(stake_vara));
        let _ = self.emit_event(GameEvents::ChallengeAccepted { match_id, deadline_block });
        Ok(())
    }

    #[export(unwrap_result)]
    pub fn reveal(&mut self, match_id: u64, player_move: Move, salt: Commit) -> Result<(), String> {
        let caller = msg::source();
        let logic_move: LogicMove = player_move.into();

        let mut state = self.state.borrow_mut();
        let config = state.config;

        let both_revealed = {
            let game_match = state.matches.get_mut(&match_id).ok_or("no such match")?;
            if game_match.state != MatchState::AwaitingReveal {
                return Err("match not awaiting reveal".into());
            }
            if caller == game_match.challenger {
                if !verify_commit(logic_move, &salt, &game_match.challenger_commit) {
                    return Err("reveal does not match commit".into());
                }
                game_match.challenger_reveal = Some(logic_move);
            } else if caller == game_match.opponent {
                let commit = game_match.opponent_commit.ok_or("opponent has not committed")?;
                if !verify_commit(logic_move, &salt, &commit) {
                    return Err("reveal does not match commit".into());
                }
                game_match.opponent_reveal = Some(logic_move);
            } else {
                return Err("not a participant".into());
            }
            game_match.challenger_reveal.is_some() && game_match.opponent_reveal.is_some()
        };

        if !both_revealed {
            return Ok(());
        }
        let events = settle(&mut state, match_id, config);
        drop(state);
        for event in events {
            let _ = self.emit_event(event);
        }
        Ok(())
    }

    #[export(unwrap_result)]
    pub fn claim_timeout(&mut self, match_id: u64) -> Result<(), String> {
        let mut state = self.state.borrow_mut();
        let config = state.config;
        let block = exec::block_height();

        let game_match = state.matches.get(&match_id).ok_or("no such match")?;
        if block <= game_match.deadline_block {
            return Err("deadline not reached".into());
        }
        match game_match.state {
            MatchState::AwaitingOpponent => {
                let (challenger, stake) = (game_match.challenger, game_match.stake_vara);
                if let Some(m) = state.matches.get_mut(&match_id) {
                    m.state = MatchState::Refunded;
                }
                drop(state);
                refund(challenger, stake);
                let _ = self.emit_event(GameEvents::MatchRefunded { match_id });
                Ok(())
            }
            MatchState::AwaitingReveal => {
                let events = settle_timeout(&mut state, match_id, config);
                drop(state);
                for event in events {
                    let _ = self.emit_event(event);
                }
                Ok(())
            }
            _ => Err("match already settled".into()),
        }
    }

    #[export]
    pub fn get_leaderboard(&self, top: u32) -> Vec<LeaderboardEntry> {
        let state = self.state.borrow();
        state
            .leaderboard
            .iter()
            .take(top as usize)
            .map(|entry| LeaderboardEntry { player: ActorId::from(entry.player), rating: entry.rating })
            .collect()
    }

    #[export]
    pub fn get_player(&self, who: ActorId) -> Option<PlayerStats> {
        self.state.borrow().players.get(&who).map(PlayerStats::from)
    }

    #[export]
    pub fn get_match(&self, match_id: u64) -> Option<MatchView> {
        self.state.borrow().matches.get(&match_id).map(MatchView::from)
    }

    #[export]
    pub fn get_pot(&self) -> u128 {
        self.state.borrow().pot
    }
}

impl GameService<'_> {
    fn fresh_stake_vara(&self, state: &GameState, stake_usd: u64) -> Result<u128, String> {
        let block = exec::block_height();
        let age = block.saturating_sub(state.rate.set_at_block);
        if state.rate.rate == 0 || age > state.config.max_rate_age_blocks {
            return Err("price rate is stale".into());
        }
        stake_to_vara(stake_usd, state.rate.rate).ok_or("stake conversion failed".into())
    }
}

fn settle(state: &mut GameState, match_id: u64, config: Config) -> Vec<GameEvents> {
    let Some((challenger, opponent, c_move, o_move, stake)) = read_revealed(state, match_id) else {
        return Vec::new();
    };
    let c_outcome = resolve(c_move, o_move);
    let champion = apply_ratings(state, config, challenger, c_outcome, opponent, resolve(o_move, c_move));
    mark_settled(state, match_id);

    let (winner, payout) = match winner_of(c_outcome, challenger, opponent) {
        Some(addr) => (Some(addr), pay_winner(state, addr, stake, config.rake_bps)),
        None => {
            let split = draw_payout(stake);
            refund(challenger, split.challenger_refund);
            refund(opponent, split.opponent_refund);
            (None, split.challenger_refund.saturating_add(split.opponent_refund))
        }
    };

    let mut events = vec![GameEvents::PvpResolved {
        match_id,
        winner,
        challenger_move: c_move.into(),
        opponent_move: o_move.into(),
        payout,
    }];
    push_champion(&mut events, champion);
    events
}

fn settle_timeout(state: &mut GameState, match_id: u64, config: Config) -> Vec<GameEvents> {
    let Some((challenger, opponent, challenger_revealed, opponent_revealed, stake)) =
        read_reveal_status(state, match_id)
    else {
        return Vec::new();
    };
    match (challenger_revealed, opponent_revealed) {
        (true, true) => settle(state, match_id, config),
        (true, false) => award_forfeit(state, match_id, config, challenger, opponent, stake),
        (false, true) => award_forfeit(state, match_id, config, opponent, challenger, stake),
        (false, false) => {
            if let Some(game_match) = state.matches.get_mut(&match_id) {
                game_match.state = MatchState::Refunded;
            }
            refund(challenger, stake);
            refund(opponent, stake);
            vec![GameEvents::MatchRefunded { match_id }]
        }
    }
}

fn award_forfeit(
    state: &mut GameState,
    match_id: u64,
    config: Config,
    winner: ActorId,
    loser: ActorId,
    stake: u128,
) -> Vec<GameEvents> {
    let champion = apply_ratings(state, config, winner, LogicOutcome::Win, loser, LogicOutcome::Loss);
    mark_settled(state, match_id);
    pay_winner(state, winner, stake, config.rake_bps);

    let mut events = vec![GameEvents::MatchForfeited { match_id, winner, loser }];
    push_champion(&mut events, champion);
    events
}

fn winner_of(challenger_outcome: LogicOutcome, challenger: ActorId, opponent: ActorId) -> Option<ActorId> {
    match challenger_outcome {
        LogicOutcome::Win => Some(challenger),
        LogicOutcome::Loss => Some(opponent),
        LogicOutcome::Draw => None,
    }
}

fn apply_ratings(
    state: &mut GameState,
    config: Config,
    a: ActorId,
    a_outcome: LogicOutcome,
    b: ActorId,
    b_outcome: LogicOutcome,
) -> Option<(ActorId, i32)> {
    let a_rating = state.record_mut(a).rating;
    let b_rating = state.record_mut(b).rating;
    let a_delta = rating_delta(a_rating, b_rating, score_milli(a_outcome), config.elo_k);
    let b_delta = rating_delta(b_rating, a_rating, score_milli(b_outcome), config.elo_k);
    apply_pvp_result(state.record_mut(a), a_delta, a_outcome);
    apply_pvp_result(state.record_mut(b), b_delta, b_outcome);
    let a_new = state.record_mut(a).rating;
    let b_new = state.record_mut(b).rating;

    let capacity = config.leaderboard_capacity as usize;
    let champ_a = update_leaderboard(&mut state.leaderboard, a.into_bytes(), a_new, capacity);
    let champ_b = update_leaderboard(&mut state.leaderboard, b.into_bytes(), b_new, capacity);
    if champ_a {
        Some((a, a_new))
    } else if champ_b {
        Some((b, b_new))
    } else {
        None
    }
}

fn pay_winner(state: &mut GameState, winner: ActorId, stake: u128, rake_bps: u16) -> u128 {
    let payout = decisive_payout(stake, rake_bps);
    state.pot = state.pot.saturating_add(payout.rake);
    refund(winner, payout.winner_amount);
    payout.winner_amount
}

fn mark_settled(state: &mut GameState, match_id: u64) {
    if let Some(game_match) = state.matches.get_mut(&match_id) {
        game_match.state = MatchState::Settled;
    }
}

fn push_champion(events: &mut Vec<GameEvents>, champion: Option<(ActorId, i32)>) {
    if let Some((player, rating)) = champion {
        events.push(GameEvents::NewChampion { player, rating });
    }
}

fn read_reveal_status(state: &GameState, match_id: u64) -> Option<(ActorId, ActorId, bool, bool, u128)> {
    let game_match = state.matches.get(&match_id)?;
    Some((
        game_match.challenger,
        game_match.opponent,
        game_match.challenger_reveal.is_some(),
        game_match.opponent_reveal.is_some(),
        game_match.stake_vara,
    ))
}

fn read_revealed(state: &GameState, match_id: u64) -> Option<(ActorId, ActorId, LogicMove, LogicMove, u128)> {
    let game_match = state.matches.get(&match_id)?;
    let c_move = game_match.challenger_reveal?;
    let o_move = game_match.opponent_reveal?;
    Some((game_match.challenger, game_match.opponent, c_move, o_move, game_match.stake_vara))
}

fn bump_outcome(record: &mut PlayerRecord, outcome: LogicOutcome) {
    match outcome {
        LogicOutcome::Win => record.wins = record.wins.saturating_add(1),
        LogicOutcome::Loss => record.losses = record.losses.saturating_add(1),
        LogicOutcome::Draw => record.draws = record.draws.saturating_add(1),
    }
}

fn apply_pvp_result(record: &mut PlayerRecord, delta: i32, outcome: LogicOutcome) {
    record.rating = record.rating.saturating_add(delta);
    record.games = record.games.saturating_add(1);
    bump_outcome(record, outcome);
}

fn refund(to: ActorId, amount: u128) {
    if amount == 0 {
        return;
    }
    let _ = msg::send_bytes(to, [], amount);
}
