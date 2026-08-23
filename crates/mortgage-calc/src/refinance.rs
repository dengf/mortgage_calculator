//! Refinance break-even and lifetime interest comparison.

use mortgage_core::{round_currency, MortgageError, MortgageResult, PaymentFrequency};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::payment::payment_factor;
use crate::rate::RateType;
use crate::Loan;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RefinanceInput {
    pub current_balance: Decimal,
    pub current_annual_rate: Decimal,
    /// Payments remaining on the current loan at its existing frequency.
    pub remaining_periods: u32,
    /// The rate shape of the loan being refinanced *into* — not a bare
    /// figure, because in Singapore the thing a borrower refinances into is
    /// a package that steps up after two or three years. Quoting it as one
    /// rate would compare a real loan against a product that does not exist.
    pub new_rate: RateType,
    pub new_term_years: Decimal,
    pub closing_costs: Decimal,
    pub frequency: PaymentFrequency,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RefinanceResult {
    pub current_payment: Decimal,
    /// The new instalment at the start. For a package that reverts, this is
    /// the promotional one — see `new_payment_after_reversion`.
    pub new_payment: Decimal,
    /// What the new instalment becomes after the lock-in. `None` when the
    /// new rate holds for its whole term.
    pub new_payment_after_reversion: Option<Decimal>,
    /// Saved each period *at the start*. For a reverting package this is the
    /// promotional saving, which is the number a switch is usually sold on
    /// and the one that stops being true first.
    pub payment_savings: Decimal,
    /// `None` when the closing costs are never recouped.
    pub break_even_periods: Option<u32>,
    pub remaining_interest_on_current_loan: Decimal,
    pub total_interest_on_new_loan: Decimal,
    /// Interest saved by switching, net of closing costs. Compares total
    /// remaining cost of the current loan against total cost of the new
    /// loan; a shorter new term is credited for the periods it no longer
    /// needs to pay interest in at all.
    pub lifetime_savings: Decimal,
}

/// Compares staying on the current loan against refinancing into a new one.
pub fn analyze_refinance(input: &RefinanceInput) -> MortgageResult<RefinanceResult> {
    if input.remaining_periods == 0 {
        return Err(MortgageError::InvalidTerm(0));
    }

    let current_periodic_rate = input.frequency.periodic_rate(input.current_annual_rate);
    let current_factor = payment_factor(current_periodic_rate, input.remaining_periods);
    let current_payment = round_currency(input.current_balance * current_factor);

    let new_loan = Loan::builder()
        .principal(input.current_balance)
        .rate_type(input.new_rate)
        .term_years(input.new_term_years)
        .frequency(input.frequency)
        .build()?;
    let new_summary = crate::payment::summarize(&new_loan);

    let payment_savings = round_currency(current_payment - new_summary.payment);

    let break_even_periods = break_even(input, current_payment, &new_loan)?;

    let remaining_interest_on_current_loan = round_currency(
        current_payment * Decimal::from(input.remaining_periods) - input.current_balance,
    );

    let lifetime_savings = round_currency(
        remaining_interest_on_current_loan - new_summary.total_interest - input.closing_costs,
    );

    Ok(RefinanceResult {
        current_payment,
        new_payment: new_summary.payment,
        new_payment_after_reversion: new_summary.payment_after_reversion,
        payment_savings,
        break_even_periods,
        remaining_interest_on_current_loan,
        total_interest_on_new_loan: new_summary.total_interest,
        lifetime_savings,
    })
}

/// The first period at which everything saved so far has covered the closing
/// costs.
///
/// Walked period by period rather than dividing the costs by a monthly
/// saving, because a saving is not a constant. A package that reverts pays
/// less than the old loan during the lock-in and can pay more afterwards, so
/// a division would extrapolate the promotional saving across a term where
/// it does not hold. The walk also stops crediting a saving once the old
/// loan would have been paid off, which the division silently kept doing
/// forever.
///
/// For a loan whose payment never changes and a term at least as long as
/// what remained, this lands on the same period the division did.
fn break_even(
    input: &RefinanceInput,
    current_payment: Decimal,
    new_loan: &Loan,
) -> MortgageResult<Option<u32>> {
    let rows = crate::amortization::schedule(new_loan, Decimal::ZERO)?;
    let mut cumulative = Decimal::ZERO;

    for period in 1..=input.remaining_periods.max(rows.len() as u32) {
        let was_paying = if period <= input.remaining_periods {
            current_payment
        } else {
            Decimal::ZERO
        };
        let now_paying = rows
            .get(period as usize - 1)
            .map(|row| row.payment)
            .unwrap_or(Decimal::ZERO);

        cumulative += was_paying - now_paying;
        if cumulative >= input.closing_costs {
            return Ok(Some(period));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn fixed(rate: Decimal) -> RateType {
        RateType::Fixed { rate }
    }

    fn input() -> RefinanceInput {
        RefinanceInput {
            current_balance: dec!(300000),
            current_annual_rate: dec!(0.075),
            remaining_periods: 300,
            new_rate: fixed(dec!(0.055)),
            new_term_years: dec!(30),
            closing_costs: dec!(6000),
            frequency: PaymentFrequency::Monthly,
        }
    }

    #[test]
    fn lower_rate_produces_positive_savings_and_break_even() {
        let result = analyze_refinance(&input()).unwrap();
        assert!(result.payment_savings > Decimal::ZERO);
        assert!(result.break_even_periods.is_some());
        assert!(result.break_even_periods.unwrap() > 0);
    }

    #[test]
    fn higher_rate_has_no_break_even() {
        let result = analyze_refinance(&RefinanceInput {
            current_annual_rate: dec!(0.045),
            new_rate: fixed(dec!(0.07)),
            ..input()
        })
        .unwrap();

        assert!(result.payment_savings < Decimal::ZERO);
        assert_eq!(result.break_even_periods, None);
    }

    #[test]
    fn rejects_zero_remaining_periods_instead_of_panicking() {
        assert!(matches!(
            analyze_refinance(&RefinanceInput {
                remaining_periods: 0,
                ..input()
            }),
            Err(MortgageError::InvalidTerm(0))
        ));
    }

    #[test]
    fn a_flat_new_loan_breaks_even_where_dividing_the_costs_would_put_it() {
        // The walk replaced a division, and for the case the division could
        // actually describe -- one unchanging saving, for longer than it
        // takes to recoup -- it has to land on the same period.
        let result = analyze_refinance(&input()).unwrap();
        let by_division = (input().closing_costs / result.payment_savings).ceil();

        assert_eq!(
            Decimal::from(result.break_even_periods.unwrap()),
            by_division
        );
    }

    #[test]
    fn a_reverting_package_reports_both_instalments() {
        let result = analyze_refinance(&RefinanceInput {
            new_rate: RateType::Reverting {
                base_rate: dec!(0.0112),
                base_floats: true,
                initial_spread: dec!(0.003),
                initial_years: dec!(2),
                thereafter_spread: dec!(0.006),
            },
            new_term_years: dec!(25),
            ..input()
        })
        .unwrap();

        let after = result.new_payment_after_reversion.unwrap();
        assert!(
            after > result.new_payment,
            "a package that steps up pays more after the lock-in: {} then {}",
            result.new_payment,
            after
        );
    }

    #[test]
    fn the_promotional_saving_is_not_extrapolated_past_the_lock_in() {
        // The expensive shape: a teaser well under the current rate for two
        // years, and a thereafter rate well over it. Costs are set to what
        // the opening saving would cover in three years -- a period the old
        // division would have named without noticing that the saving stops
        // a year earlier and turns into a loss.
        let sold_on = RefinanceInput {
            current_annual_rate: dec!(0.04),
            new_rate: RateType::Reverting {
                base_rate: dec!(0.01),
                base_floats: true,
                initial_spread: dec!(0.005),
                initial_years: dec!(2),
                thereafter_spread: dec!(0.05),
            },
            new_term_years: dec!(25),
            closing_costs: Decimal::ZERO,
            ..input()
        };

        let probe = analyze_refinance(&sold_on).unwrap();
        assert!(probe.payment_savings > Decimal::ZERO);

        let costs = probe.payment_savings * dec!(36);
        let result = analyze_refinance(&RefinanceInput {
            closing_costs: costs,
            ..sold_on
        })
        .unwrap();

        assert_eq!(
            (costs / probe.payment_savings).ceil(),
            dec!(36),
            "dividing the costs by the opening saving points at period 36"
        );
        assert_eq!(
            result.break_even_periods, None,
            "but the saving ends at period 24, and never adds up to that"
        );
    }

    #[test]
    fn nothing_is_saved_after_the_old_loan_would_have_been_paid_off() {
        // A new term far longer than what remained. Everything the old loan
        // still had to pay is the most that switching could ever save, so
        // costs above that are never recouped -- however long the new loan
        // runs on afterwards.
        let stretched = RefinanceInput {
            current_annual_rate: dec!(0.03),
            remaining_periods: 60,
            new_rate: fixed(dec!(0.0295)),
            new_term_years: dec!(30),
            closing_costs: Decimal::ZERO,
            ..input()
        };

        let probe = analyze_refinance(&stretched).unwrap();
        let ceiling = probe.payment_savings * Decimal::from(stretched.remaining_periods);
        let costs = ceiling + dec!(1000);

        let result = analyze_refinance(&RefinanceInput {
            closing_costs: costs,
            ..stretched
        })
        .unwrap();

        assert!(
            (costs / probe.payment_savings).ceil() > Decimal::from(stretched.remaining_periods),
            "a division puts break-even past the end of the loan being replaced"
        );
        assert_eq!(result.break_even_periods, None);
    }
}
