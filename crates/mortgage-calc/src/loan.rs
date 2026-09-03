use mortgage_core::{MortgageError, MortgageResult, PaymentFrequency};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::rate::RateType;

/// No real mortgage runs anywhere near this long. This exists purely as a
/// ceiling so a mistyped or adversarial term can't force
/// `amortization::schedule()` into a multi-gigabyte `Vec::with_capacity`
/// allocation — which aborts the whole process on failure rather than
/// returning a catchable error.
const MAX_TOTAL_PERIODS: u32 = 52 * 100; // 100 years, even at the most frequent (weekly) schedule

/// No real mortgage anywhere charges anywhere near this. Without a ceiling, a
/// fat-fingered or adversarial rate (`999` typed into a field meant for
/// `9.99`) produces a fully-formed, confidently-displayed report -- payment
/// table, amortization schedule, rate-rise stress test -- built on a number
/// nobody intended. That is exactly the "quietly wrong" failure this crate
/// exists to prevent, so it is rejected the same way an impossible term is.
const MAX_ANNUAL_RATE: Decimal = dec!(1); // 100%

/// The point at which a loan's rate changes, and what it changes to.
///
/// Singapore packages are built this way and it is not a detail: a bank
/// quotes a promotional spread over SORA for the first two or three years
/// and a higher one for the remaining twenty-odd. Modelling only the
/// promotional rate overstates what the borrower can afford for the whole
/// term, which is the error MAS Notice 645 exists to prevent -- its stress
/// floor is the higher of 4% and the *thereafter* rate, not the teaser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reversion {
    /// Number of periods charged at the initial rate. The period after this
    /// is the first at the new one.
    pub after_periods: u32,
    /// The rate from then until the end of the term.
    pub annual_rate: Decimal,
}

/// An amortizing loan: the shared input to every calculation in this crate.
///
/// The rate is fixed for the term unless a [`Reversion`] says otherwise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loan {
    principal: Decimal,
    annual_rate: Decimal,
    term_years: Decimal,
    frequency: PaymentFrequency,
    reversion: Option<Reversion>,
    floating_base: Option<Decimal>,
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

    pub fn term_years(&self) -> Decimal {
        self.term_years
    }

    /// Interest rate charged per payment period, before any reversion.
    pub fn periodic_rate(&self) -> Decimal {
        self.frequency.periodic_rate(self.annual_rate)
    }

    /// When and to what the rate changes, if it does.
    pub fn reversion(&self) -> Option<Reversion> {
        self.reversion
    }

    /// The moving benchmark the rate was quoted over, if it was quoted over
    /// one. See [`RateType::floating_base`].
    ///
    /// Carried on the loan rather than left behind with the `RateType` for
    /// the same reason [`Reversion`] is: a builder that flattens a quote to
    /// a number lets every calculation downstream present an assumption as
    /// a fact. The rate is the arithmetic; this is what the arithmetic
    /// rests on, and a report that prints one without the other is telling
    /// the reader less than it knows.
    pub fn floating_base(&self) -> Option<Decimal> {
        self.floating_base
    }

    /// The rate charged for the remainder of the term -- the one the loan
    /// spends most of its life at.
    ///
    /// For a flat loan this is just the rate. For one that steps up it is
    /// the thereafter rate, which is what a lender qualifies the borrower on
    /// rather than the promotional rate they open at.
    pub fn final_annual_rate(&self) -> Decimal {
        self.reversion
            .map(|r| r.annual_rate)
            .unwrap_or(self.annual_rate)
    }

    /// The annual rate charged in `period`, counting from 1.
    ///
    /// A reversion takes effect *after* `after_periods` payments, so period
    /// `after_periods` is still at the initial rate and `after_periods + 1`
    /// is the first at the new one.
    pub fn annual_rate_at(&self, period: u32) -> Decimal {
        match self.reversion {
            Some(r) if period > r.after_periods => r.annual_rate,
            _ => self.annual_rate,
        }
    }

    /// Periodic rate charged in `period`, counting from 1.
    pub fn periodic_rate_at(&self, period: u32) -> Decimal {
        self.frequency.periodic_rate(self.annual_rate_at(period))
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
    reversion: Option<Reversion>,
    reversion_after_years: Option<(Decimal, Decimal)>,
    floating_base: Option<Decimal>,
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

    /// Sets the rate from a [`RateType`] instead of a bare `Decimal`.
    ///
    /// A reverting package also sets the reversion, so a caller cannot take
    /// the rate and silently drop the step-up -- which would model the
    /// promotional rate as lasting the whole term.
    /// A quote resting on a moving benchmark also records that it does, so
    /// a caller cannot flatten it to a number and present the result as
    /// contractual.
    pub fn rate_type(mut self, rate_type: RateType) -> Self {
        self.annual_rate = Some(rate_type.effective_rate());
        if let (Some(years), Some(rate)) = (rate_type.initial_years(), rate_type.thereafter_rate())
        {
            self.reversion_after_years = Some((years, rate));
        }
        self.floating_base = rate_type.floating_base();
        self
    }

    /// Sets a rate change expressed in years, converted to periods once the
    /// payment frequency is known.
    pub fn reversion_after_years(mut self, years: Decimal, annual_rate: Decimal) -> Self {
        self.reversion_after_years = Some((years, annual_rate));
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

    /// Sets a rate change partway through the term. See [`Reversion`].
    pub fn reversion(mut self, reversion: Reversion) -> Self {
        self.reversion = Some(reversion);
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
        if annual_rate > MAX_ANNUAL_RATE {
            return Err(MortgageError::RateTooHigh(annual_rate.to_string()));
        }

        let reversion = self.reversion.or_else(|| {
            self.reversion_after_years
                .map(|(years, annual_rate)| Reversion {
                    after_periods: frequency.periods_in_years(years),
                    annual_rate,
                })
        });

        if let Some(r) = reversion {
            if r.annual_rate < Decimal::ZERO {
                return Err(MortgageError::InvalidRate(r.annual_rate.to_string()));
            }
            if r.annual_rate > MAX_ANNUAL_RATE {
                return Err(MortgageError::RateTooHigh(r.annual_rate.to_string()));
            }
        }

        let total_periods = frequency.periods_in_years(term_years);
        if total_periods == 0 {
            return Err(MortgageError::InvalidTerm(total_periods));
        }
        if total_periods > MAX_TOTAL_PERIODS {
            return Err(MortgageError::TermTooLong(total_periods));
        }

        // A reversion at or past the end of the term never takes effect;
        // dropping it keeps every downstream calculation on the simple path
        // rather than having each one re-check the same boundary.
        let reversion = reversion.filter(|r| r.after_periods < total_periods);

        Ok(Loan {
            principal,
            annual_rate,
            term_years,
            frequency,
            reversion,
            floating_base: self.floating_base,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_loan_built_from_a_benchmark_quote_remembers_that_it_was() {
        let sora = |base_floats| {
            Loan::builder()
                .principal(dec!(400000))
                .rate_type(RateType::Reverting {
                    base_rate: dec!(0.0112),
                    base_floats,
                    initial_spread: dec!(0.003),
                    initial_years: dec!(2),
                    thereafter_spread: dec!(0.006),
                })
                .term_years(dec!(25))
                .frequency(PaymentFrequency::Monthly)
                .build()
                .unwrap()
        };

        assert_eq!(sora(true).floating_base(), Some(dec!(0.0112)));
        assert_eq!(sora(false).floating_base(), None);
        // The disclosure travels; the arithmetic is untouched.
        assert_eq!(sora(true).annual_rate(), sora(false).annual_rate());
        assert_eq!(
            sora(true).final_annual_rate(),
            sora(false).final_annual_rate()
        );
    }

    #[test]
    fn a_rate_given_as_a_bare_number_claims_no_benchmark() {
        // `annual_rate()` is how the tabs that only ever had a flat rate
        // build a loan. Inventing a benchmark under it would put a caveat on
        // a figure the user typed as final.
        let loan = Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.065))
            .term_years(dec!(30))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();

        assert_eq!(loan.floating_base(), None);
    }

    use mortgage_core::{MortgageError, PaymentFrequency};

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

    #[test]
    fn rejects_a_fat_fingered_rate_instead_of_reporting_it_with_confidence() {
        // A stray extra digit -- 999 typed where 9.99 was meant -- must not
        // sail through and produce a fully-rendered, exact-looking report.
        let result = Loan::builder()
            .principal(dec!(500_000))
            .annual_rate(dec!(9.99))
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Monthly)
            .build();

        assert_eq!(
            result,
            Err(MortgageError::RateTooHigh(dec!(9.99).to_string()))
        );
    }

    #[test]
    fn accepts_a_rate_right_at_the_ceiling() {
        let loan = Loan::builder()
            .principal(dec!(500_000))
            .annual_rate(dec!(1.0))
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();

        assert_eq!(loan.annual_rate(), dec!(1.0));
    }

    #[test]
    fn rejects_a_reversion_rate_above_the_ceiling_even_when_the_initial_rate_is_fine() {
        let result = Loan::builder()
            .principal(dec!(500_000))
            .annual_rate(dec!(0.02))
            .reversion_after_years(dec!(2), dec!(9.99))
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Monthly)
            .build();

        assert_eq!(
            result,
            Err(MortgageError::RateTooHigh(dec!(9.99).to_string()))
        );
    }
}
