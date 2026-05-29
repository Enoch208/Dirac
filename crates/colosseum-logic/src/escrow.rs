pub const RATE_SCALE: u128 = 1_000_000;
pub const ONE_VARA: u128 = 1_000_000_000_000;
const BPS_DENOMINATOR: u128 = 10_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Payout {
    pub winner_amount: u128,
    pub challenger_refund: u128,
    pub opponent_refund: u128,
    pub rake: u128,
}

pub fn stake_to_vara(stake_usd: u64, rate_micro_usd_per_vara: u128) -> Option<u128> {
    if rate_micro_usd_per_vara == 0 {
        return None;
    }
    (stake_usd as u128)
        .checked_mul(ONE_VARA)?
        .checked_mul(RATE_SCALE)?
        .checked_div(rate_micro_usd_per_vara)
}

pub fn decisive_payout(stake_each: u128, rake_bps: u16) -> Payout {
    let pot = stake_each.saturating_mul(2);
    let rake = pot.saturating_mul(rake_bps as u128) / BPS_DENOMINATOR;
    Payout {
        winner_amount: pot.saturating_sub(rake),
        challenger_refund: 0,
        opponent_refund: 0,
        rake,
    }
}

pub fn draw_payout(stake_each: u128) -> Payout {
    Payout {
        winner_amount: 0,
        challenger_refund: stake_each,
        opponent_refund: stake_each,
        rake: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stake_sizing_pegs_to_usd() {
        let rate = 500_000;
        assert_eq!(stake_to_vara(10, rate), Some(20 * ONE_VARA));
    }

    #[test]
    fn stake_sizing_rejects_zero_rate() {
        assert_eq!(stake_to_vara(10, 0), None);
    }

    #[test]
    fn decisive_payout_takes_rake_and_conserves_value() {
        let p = decisive_payout(100, 500);
        assert_eq!(p.winner_amount, 190);
        assert_eq!(p.rake, 10);
        assert_eq!(p.winner_amount + p.rake, 200);
        assert_eq!(p.challenger_refund, 0);
        assert_eq!(p.opponent_refund, 0);
    }

    #[test]
    fn zero_rake_gives_winner_the_whole_pot() {
        let p = decisive_payout(100, 0);
        assert_eq!(p.winner_amount, 200);
        assert_eq!(p.rake, 0);
    }

    #[test]
    fn draw_refunds_both_and_conserves_value() {
        let p = draw_payout(100);
        assert_eq!(p.challenger_refund, 100);
        assert_eq!(p.opponent_refund, 100);
        assert_eq!(p.winner_amount, 0);
        assert_eq!(p.rake, 0);
        assert_eq!(p.challenger_refund + p.opponent_refund, 200);
    }
}
