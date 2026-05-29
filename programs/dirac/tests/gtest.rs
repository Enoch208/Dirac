use dirac_client::admin::Admin as _;
use dirac_client::game::Game as _;
use dirac_client::{Dirac as _, DiracCtors as _, Move, RoundResult};
use dirac_logic::commit::commit_hash;
use dirac_logic::rps::Move as LogicMove;
use sails_rs::client::*;
use sails_rs::futures::StreamExt as _;
use sails_rs::gtest::{Log, System};
use sails_rs::prelude::*;

const OPERATOR: u64 = 42;
const CHALLENGER: u64 = 43;
const OPPONENT: u64 = 44;
const FUNDING: u128 = 100_000_000_000_000;
const RATE_MICRO_USD_PER_VARA: u128 = 1_000_000;
const STAKE_USD: u64 = 5;
const STAKE_VARA: u128 = 5_000_000_000_000;
const EXPECTED_RAKE: u128 = 250_000_000_000;

#[cfg(debug_assertions)]
const WASM_PATH: &str = "../../target/wasm32-gear/debug/dirac.opt.wasm";
#[cfg(not(debug_assertions))]
const WASM_PATH: &str = "../../target/wasm32-gear/release/dirac.opt.wasm";

fn create_env() -> (GtestEnv, CodeId) {
    let system = System::new();
    system.init_logger_with_default_filter("");
    system.mint_to(OPERATOR, FUNDING);
    system.mint_to(CHALLENGER, FUNDING);
    system.mint_to(OPPONENT, FUNDING);
    let code_id = system.submit_code_file(WASM_PATH);
    let env = GtestEnv::new(system, OPERATOR.into());
    (env, code_id)
}

fn player(env: &GtestEnv, who: u64, program_id: ActorId) -> Actor<dirac_client::DiracProgram, GtestEnv> {
    Actor::new(env.clone().with_actor_id(who.into()), program_id)
}

fn mailbox_has_value(env: &GtestEnv, program_id: ActorId, who: u64) -> bool {
    let mailbox = env.system().get_mailbox(who);
    mailbox.contains(&Log::builder().source(program_id).dest(ActorId::from(who)))
}

#[tokio::test]
async fn play_resolves_and_records_the_player() {
    use dirac_client::game::events::GameEvents;

    let (env, code_id) = create_env();
    let program = env.deploy(code_id, vec![]).new().await.unwrap();

    let mut game = program.game();
    let listener = game.listener();
    let mut events = listener.listen().await.unwrap();

    let result: RoundResult = game.play(Move::Rock).await.unwrap();
    assert_eq!(result.player_move, Move::Rock);

    let (source, event) = events.next().await.unwrap();
    assert_eq!(source, program.id());
    assert!(matches!(event, GameEvents::MatchPlayed { player_move: Move::Rock, .. }));

    let stats = program.game().get_player(OPERATOR.into()).await.unwrap().unwrap();
    assert_eq!(stats.games, 1);
}

#[tokio::test]
async fn operator_can_set_rate() {
    use dirac_client::admin::events::AdminEvents;

    let (env, code_id) = create_env();
    let program = env.deploy(code_id, vec![]).new().await.unwrap();

    let mut admin = program.admin();
    let listener = admin.listener();
    let mut events = listener.listen().await.unwrap();

    admin.set_vara_usd_rate(RATE_MICRO_USD_PER_VARA).await.unwrap();

    let (_source, event) = events.next().await.unwrap();
    assert!(matches!(event, AdminEvents::RateUpdated { rate: RATE_MICRO_USD_PER_VARA, .. }));
}

#[tokio::test]
async fn staked_pvp_pays_the_winner_and_takes_rake() {
    let (env, code_id) = create_env();
    let program = env.deploy(code_id, vec![]).new().await.unwrap();
    let program_id = program.id();

    program.admin().set_vara_usd_rate(RATE_MICRO_USD_PER_VARA).await.unwrap();

    let salt_c = [1u8; 32];
    let salt_o = [2u8; 32];
    let commit_c = commit_hash(LogicMove::Rock, &salt_c);
    let commit_o = commit_hash(LogicMove::Scissors, &salt_o);

    let challenger = Actor::<dirac_client::DiracProgram, _>::new(
        env.clone().with_actor_id(CHALLENGER.into()),
        program_id,
    );
    let opponent = Actor::<dirac_client::DiracProgram, _>::new(
        env.clone().with_actor_id(OPPONENT.into()),
        program_id,
    );

    let match_id = challenger
        .game()
        .challenge(OPPONENT.into(), commit_c, STAKE_USD)
        .with_value(STAKE_VARA)
        .await
        .unwrap();

    opponent
        .game()
        .accept_challenge(match_id, commit_o)
        .with_value(STAKE_VARA)
        .await
        .unwrap();

    let escrow_balance = env.system().balance_of(program_id);
    assert!(escrow_balance >= 2 * STAKE_VARA);

    let challenger_before_payout = env.system().balance_of(ActorId::from(CHALLENGER));

    challenger.game().reveal(match_id, Move::Rock, salt_c).await.unwrap();
    opponent.game().reveal(match_id, Move::Scissors, salt_o).await.unwrap();

    for _ in 0..3 {
        env.run_next_block();
    }

    let view = program.game().get_match(match_id).await.unwrap().unwrap();
    assert_eq!(view.state, dirac_client::MatchState::Settled, "match state: {:?}", view.state);

    let pot = program.game().get_pot().await.unwrap();
    assert_eq!(pot, EXPECTED_RAKE, "pot/rake: {pot}");

    let mailbox = env.system().get_mailbox(CHALLENGER);
    let payout = Log::builder().source(program_id).dest(ActorId::from(CHALLENGER));
    assert!(mailbox.contains(&payout), "winnings not delivered to winner mailbox");
    mailbox.claim_value(payout).unwrap();
    env.run_next_block();

    let challenger_after_payout = env.system().balance_of(ActorId::from(CHALLENGER));
    assert!(
        challenger_after_payout - challenger_before_payout > 9_000_000_000_000,
        "winner not paid: before={challenger_before_payout} after={challenger_after_payout}",
    );
}

#[tokio::test]
async fn staked_pvp_draw_refunds_both_players() {
    let (env, code_id) = create_env();
    let program = env.deploy(code_id, vec![]).new().await.unwrap();
    let program_id = program.id();
    program.admin().set_vara_usd_rate(RATE_MICRO_USD_PER_VARA).await.unwrap();

    let salt_c = [3u8; 32];
    let salt_o = [4u8; 32];
    let commit_c = commit_hash(LogicMove::Paper, &salt_c);
    let commit_o = commit_hash(LogicMove::Paper, &salt_o);

    let challenger = player(&env, CHALLENGER, program_id);
    let opponent = player(&env, OPPONENT, program_id);

    let match_id = challenger
        .game()
        .challenge(OPPONENT.into(), commit_c, STAKE_USD)
        .with_value(STAKE_VARA)
        .await
        .unwrap();
    opponent
        .game()
        .accept_challenge(match_id, commit_o)
        .with_value(STAKE_VARA)
        .await
        .unwrap();

    challenger.game().reveal(match_id, Move::Paper, salt_c).await.unwrap();
    opponent.game().reveal(match_id, Move::Paper, salt_o).await.unwrap();
    for _ in 0..3 {
        env.run_next_block();
    }

    assert_eq!(program.game().get_pot().await.unwrap(), 0, "draw must not take rake");
    assert!(mailbox_has_value(&env, program_id, CHALLENGER), "challenger not refunded");
    assert!(mailbox_has_value(&env, program_id, OPPONENT), "opponent not refunded");
    let view = program.game().get_match(match_id).await.unwrap().unwrap();
    assert_eq!(view.state, dirac_client::MatchState::Settled);
}

#[tokio::test]
async fn timeout_awards_forfeit_to_sole_revealer() {
    let (env, code_id) = create_env();
    let program = env.deploy(code_id, vec![]).new().await.unwrap();
    let program_id = program.id();
    program.admin().set_vara_usd_rate(RATE_MICRO_USD_PER_VARA).await.unwrap();
    program
        .admin()
        .set_config(dirac_client::Config {
            house_epsilon_bps: 2000,
            elo_k: 32,
            house_rating: 1500,
            leaderboard_capacity: 100,
            max_rate_age_blocks: 1800,
            rake_bps: 250,
            reveal_deadline_blocks: 2,
            min_stake_usd: 1,
            max_stake_usd: 1000,
        })
        .await
        .unwrap();

    let salt_c = [5u8; 32];
    let commit_c = commit_hash(LogicMove::Rock, &salt_c);
    let commit_o = commit_hash(LogicMove::Scissors, &[6u8; 32]);

    let challenger = player(&env, CHALLENGER, program_id);
    let opponent = player(&env, OPPONENT, program_id);

    let match_id = challenger
        .game()
        .challenge(OPPONENT.into(), commit_c, STAKE_USD)
        .with_value(STAKE_VARA)
        .await
        .unwrap();
    opponent
        .game()
        .accept_challenge(match_id, commit_o)
        .with_value(STAKE_VARA)
        .await
        .unwrap();

    challenger.game().reveal(match_id, Move::Rock, salt_c).await.unwrap();
    for _ in 0..4 {
        env.run_next_block();
    }

    opponent.game().claim_timeout(match_id).await.unwrap();
    for _ in 0..3 {
        env.run_next_block();
    }

    assert_eq!(program.game().get_pot().await.unwrap(), EXPECTED_RAKE, "forfeit must take rake");
    assert!(mailbox_has_value(&env, program_id, CHALLENGER), "forfeit winner not paid");
    let view = program.game().get_match(match_id).await.unwrap().unwrap();
    assert_eq!(view.state, dirac_client::MatchState::Settled);
}
