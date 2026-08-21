use mortgage_core::{MortgageError, MortgageResult, PaymentFrequency};
use rust_decimal::Decimal;

use crate::rate::RateType;

/// No real mortgage runs anywhere near this long. This exists purely as a
/// ceiling so a mistyped or adversarial term can't force
/// `amortization::schedule()` into a multi-gigabyte `Vec::with_capacity`
/// allocation — which aborts the whole process on failure rather than
/// returning a catchable error.
const MAX_TOTAL_PERIODS: u32 = 52 * 100; // 100 years, even at the most frequent (weekly) schedule

/// A fixed-rate amortizing loan: the shared input to every calculation in
/// this crate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loan {
    principal: Decimal,
    annual_rate: Decimal,
    term_years: Decimal,
    frequency: PaymentFrequency,
}

impl Loan {
    pub fn builder() -> LoanBuilder {
        LoanBuilder::default()
    }

    pub fn principal(&self) -> Decimal {
        self.principal
    }

    pub fn annual_rate(&self) -> Decimal {
        self.annual_rate
    }

    pub fn frequency(&self) -> PaymentFrequency {
        self.frequency
    }

    /// Interest rate charged per payment period.
    pub fn periodic_rate(&self) -> Decimal {
        self.frequency.periodic_rate(self.annual_rate)
    }

    /// Total number of scheduled payments over the life of the loan.
    pub fn total_periods(&self) -> u32 {
        self.frequency.periods_in_years(self.term_years)
    }
}

/// Validating builder for [`Loan`]: construction is the single place inputs
/// are checked, so every downstream calculation can assume a valid loan.
#[derive(Debug, Default, Clone, Copy)]
pub struct LoanBuilder {
    principal: Option<Decimal>,
    annual_rate: Option<Decimal>,
    term_years: Option<Decimal>,
    frequency: Option<PaymentFrequency>,
}

impl LoanBuilder {
    pub fn principal(mut self, principal: Decimal) -> Self {
        self.principal = Some(principal);
        self
    }

    pub fn annual_rate(mut self, annual_rate: Decimal) -> Self {
        self.annual_rate = Some(annual_rate);
        self
    }

    /// Sets the rate via a [`RateType`] (fixed, or floating base+spread)
    /// instead of a bare `Decimal`. Equivalent to
    /// `.annual_rate(rate_type.effective_rate())`.
    pub fn rate_type(mut self, rate_type: RateType) -> Self {
        self.annual_rate = Some(rate_type.effective_rate());
        self
    }

    pub fn term_years(mut self, term_years: Decimal) -> Self {
        self.term_years = Some(term_years);
        self
    }

    pub fn frequency(mut self, frequency: PaymentFrequency) -> Self {
        self.frequency = Some(frequency);
        self
    }

    pub fn build(self) -> MortgageResult<Loan> {
        let principal = self.principal.unwrap_or_default();
        let annual_rate = self.annual_rate.unwrap_or_default();
        let term_years = self.term_years.unwrap_or_default();
        let frequency = self.frequency.unwrap_or_default();

        if principal <= Decimal::ZERO {
            return Err(MortgageError::InvalidPrincipal(principal.to_string()));
        }
        if annual_rate < Decimal::ZERO {
            return Err(MortgageError::InvalidRate(annual_rate.to_string()));
        }

        let total_periods = frequency.periods_in_years(term_years);
        if total_periods == 0 {
            return Err(MortgageError::InvalidTerm(total_periods));
        }
        if total_periods > MAX_TOTAL_PERIODS {
            return Err(MortgageError::TermTooLong(total_periods));
        }

        Ok(Loan {
            principal,
            annual_rate,
            term_years,
            frequency,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mortgage_core::{MortgageError, PaymentFrequency};
    use rust_decimal_macros::dec;

    #[test]
    fn rejects_a_term_long_enough_to_have_forced_a_multi_gigabyte_allocation() {
        // At weekly frequency this is well under u32::MAX periods, so it
        // used to sail past the old `total_periods == 0` check straight
        // into `amortization::schedule()`'s `Vec::with_capacity`.
        let result = Loan::builder()
            .principal(dec!(500_000))
            .annual_rate(dec!(0.06))
            .term_years(dec!(1_000_000))
            .frequency(PaymentFrequency::Weekly)
            .build();

        assert_eq!(result, Err(MortgageError::TermTooLong(52_000_000)));
    }

    #[test]
    fn accepts_a_realistic_long_term() {
        let loan = Loan::builder()
            .principal(dec!(500_000))
            .annual_rate(dec!(0.06))
            .term_years(dec!(50))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();

        assert_eq!(loan.total_periods(), 600);
    }
}
