use rust_decimal::{Decimal, RoundingStrategy};

/// Rounds a decimal amount to whole cents (half-up), the convention used
/// for every currency figure returned by this workspace.
pub fn round_currency(amount: Decimal) -> Decimal {
    amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}
