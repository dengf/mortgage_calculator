use rust_decimal::{Decimal, RoundingStrategy};

/// Rounds a decimal amount to whole cents (half-up), the convention used
/// for every currency figure returned by this workspace.
pub fn round_currency(amount: Decimal) -> Decimal {
    amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn rounds_a_midpoint_away_from_zero_in_both_directions() {
        assert_eq!(round_currency(dec!(1.005)), dec!(1.01));
        assert_eq!(round_currency(dec!(-1.005)), dec!(-1.01));
    }

    #[test]
    fn rounds_below_and_above_the_midpoint_toward_the_nearer_cent() {
        assert_eq!(round_currency(dec!(1.004)), dec!(1.00));
        assert_eq!(round_currency(dec!(1.006)), dec!(1.01));
    }

    #[test]
    fn leaves_an_already_whole_cent_amount_unchanged() {
        assert_eq!(round_currency(dec!(42.50)), dec!(42.50));
        assert_eq!(round_currency(dec!(0)), dec!(0));
    }
}
