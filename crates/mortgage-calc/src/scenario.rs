//! The loan a buyer is describing, and the figures derived from it.
//!
//! Price, deposit and loan amount are three views of one thing, and only two
//! of them are ever entered. Which two depends on how the buyer thinks:
//! "we've saved $80,000" and "we're putting 20% down" are the same fact, and
//! the deposit field accepts either.
//!
//! Deriving the third view is arithmetic on money, so it lives here rather
//! than in a front end. It had been reimplemented in JavaScript, and a second
//! copy of a rule is a rule that can disagree with itself -- silently, since
//! the result is a plausible number rather than a crash.

use rust_decimal::Decimal;

use mortgage_core::round_currency;

/// A price and what the buyer is putting down against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scenario {
    pub home_price: Decimal,
    pub down_payment: Decimal,
}

/// Every figure that follows from a [`Scenario`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioSummary {
    /// The amount actually borrowed.
    pub principal: Decimal,
    /// The deposit as a share of price, or `None` when there is no price to
    /// divide by. `None` is not zero: a buyer who has not entered a price
    /// has no deposit percentage, and rendering "0.0% of price" would state
    /// something the inputs do not say.
    pub down_payment_percent: Option<Decimal>,
}

impl Scenario {
    pub fn new(home_price: Decimal, down_payment: Decimal) -> Self {
        Scenario {
            home_price,
            down_payment,
        }
    }

    /// The amount actually borrowed.
    ///
    /// Never negative, however the fields are set. A deposit larger than the
    /// price is a half-typed entry, not a loan the bank owes the buyer.
    pub fn principal(&self) -> Decimal {
        (self.home_price - self.down_payment).max(Decimal::ZERO)
    }

    /// The deposit as a percentage of price.
    pub fn down_payment_percent(&self) -> Option<Decimal> {
        if self.home_price <= Decimal::ZERO {
            return None;
        }
        Some(self.down_payment / self.home_price * Decimal::ONE_HUNDRED)
    }

    /// The deposit a given percentage of the price comes to, in whole cents.
    ///
    /// The percentage is a lens on the amount rather than a second stored
    /// value, so this is what the toggle writes back when someone types
    /// "20" instead of "100000". Rounding to cents here keeps the stored
    /// amount a real sum of money rather than a repeating decimal that
    /// prints as one.
    pub fn down_payment_for_percent(home_price: Decimal, percent: Decimal) -> Decimal {
        round_currency(home_price * percent / Decimal::ONE_HUNDRED)
    }

    pub fn summary(&self) -> ScenarioSummary {
        ScenarioSummary {
            principal: self.principal(),
            down_payment_percent: self.down_payment_percent(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn principal_is_price_less_deposit() {
        let s = Scenario::new(dec!(500000), dec!(100000));
        assert_eq!(s.principal(), dec!(400000));
        assert_eq!(s.down_payment_percent(), Some(dec!(20)));
    }

    #[test]
    fn a_deposit_larger_than_the_price_borrows_nothing() {
        // Reachable mid-typing, and a negative loan would propagate into
        // every payment figure on the page.
        let s = Scenario::new(dec!(250000), dec!(300000));
        assert_eq!(s.principal(), Decimal::ZERO);
    }

    #[test]
    fn a_missing_price_has_no_deposit_percentage() {
        assert_eq!(
            Scenario::new(dec!(0), dec!(50000)).down_payment_percent(),
            None
        );
        assert_eq!(
            Scenario::new(dec!(-10), dec!(50000)).down_payment_percent(),
            None
        );
    }

    #[test]
    fn a_percentage_converts_back_to_whole_cents() {
        assert_eq!(
            Scenario::down_payment_for_percent(dec!(500000), dec!(20)),
            dec!(100000)
        );
        // 33.333% of 987,654.32 is 329,214.8145..., which must not be
        // stored as a fraction of a cent.
        let d = Scenario::down_payment_for_percent(dec!(987654.32), dec!(33.333));
        assert_eq!(d, dec!(329214.81));
    }

    #[test]
    fn percent_round_trips_through_the_amount_it_names() {
        let price = dec!(750000);
        let amount = Scenario::down_payment_for_percent(price, dec!(15));
        assert_eq!(
            Scenario::new(price, amount).down_payment_percent(),
            Some(dec!(15))
        );
    }
}
