use mortgage_core::{MortgageError, MortgageResult, PaymentFrequency};
use rust_decimal::Decimal;

use crate::rate::RateType;

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

        Ok(Loan {
            principal,
            annual_rate,
            term_years,
            frequency,
        })
    }
}
