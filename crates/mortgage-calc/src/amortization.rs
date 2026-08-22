//! Payment-by-payment amortization schedule and extra-payment payoff impact.

use mortgage_core::{round_currency, MortgageError, MortgageResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::payment::{payment_amount, payment_factor};
use crate::Loan;

/// One row of an amortization schedule.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AmortizationRow {
    pub period: u32,
    pub payment: Decimal,
    pub extra_payment: Decimal,
    pub principal_portion: Decimal,
    pub interest_portion: Decimal,
    pub remaining_balance: Decimal,
}

/// One year of a schedule, for readers who want the shape of the loan rather
/// than every payment in it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AmortizationYear {
    pub year: u32,
    pub paid: Decimal,
    pub principal: Decimal,
    pub interest: Decimal,
    pub remaining_balance: Decimal,
}

/// Groups a schedule into years.
///
/// The last year is short whenever the loan pays off early, which any extra
/// payment causes, so years are taken as they come rather than assumed full.
///
/// This is not here for numeric accuracy. Every row is already rounded to
/// whole cents, so summing twelve of them as `f64` in the front end agreed
/// with this to the cent when measured -- the error is around 1e-12 and
/// cannot surface at two decimal places.
///
/// It is here because the grouping is a rule, not a display detail: which
/// rows belong to which year, and the fact that the final year is short
/// whenever a loan pays off early. That rule was written once per front end,
/// which is one copy too many, and it is worth a test of its own.
pub fn summarize_by_year(rows: &[AmortizationRow], periods_per_year: u32) -> Vec<AmortizationYear> {
    if periods_per_year == 0 {
        return Vec::new();
    }
    rows.chunks(periods_per_year as usize)
        .enumerate()
        .map(|(index, chunk)| AmortizationYear {
            year: index as u32 + 1,
            paid: chunk.iter().map(|r| r.payment).sum(),
            principal: chunk.iter().map(|r| r.principal_portion).sum(),
            interest: chunk.iter().map(|r| r.interest_portion).sum(),
            // The balance at the end of the year is the last row's, not a
            // sum: balances are levels, not flows.
            remaining_balance: chunk
                .last()
                .map(|r| r.remaining_balance)
                .unwrap_or(Decimal::ZERO),
        })
        .collect()
}

/// Builds the full payment-by-payment schedule for `loan`.
///
/// `extra_payment` is applied on top of the regular payment every period
/// and goes entirely to principal. The schedule ends as soon as the balance
/// reaches zero, which — with a nonzero extra payment — is before the
/// loan's nominal term.
pub fn schedule(loan: &Loan, extra_payment: Decimal) -> MortgageResult<Vec<AmortizationRow>> {
    if extra_payment < Decimal::ZERO {
        return Err(MortgageError::InvalidExtraPayment(
            extra_payment.to_string(),
        ));
    }

    let max_periods = loan.total_periods();

    let mut periodic_rate = loan.periodic_rate();
    let mut regular_payment = payment_amount(loan);
    let mut balance = loan.principal();
    let mut rows = Vec::with_capacity(max_periods as usize);

    for period in 1..=max_periods {
        if balance <= Decimal::ZERO {
            break;
        }

        // At a reversion the bank re-amortizes: the new instalment is what
        // clears the balance actually outstanding over the periods actually
        // left, at the new rate. It is not the original payment with a
        // different rate applied, and it is not a fresh loan over a fresh
        // term -- both would misstate what the borrower starts paying.
        if let Some(r) = loan.reversion() {
            if period == r.after_periods + 1 {
                periodic_rate = loan.periodic_rate_at(period);
                let remaining = max_periods - r.after_periods;
                regular_payment =
                    round_currency(balance * payment_factor(periodic_rate, remaining));
            }
        }

        let interest_portion = round_currency(balance * periodic_rate);
        let scheduled_principal = regular_payment - interest_portion;
        let extra = extra_payment.min((balance - scheduled_principal).max(Decimal::ZERO));

        let mut principal_portion = scheduled_principal + extra;
        let mut payment = regular_payment + extra;

        // Final payment: force payoff to exactly zero, both when the
        // regular payment would overshoot the balance and on the loan's
        // last scheduled period, where rounding drift accumulated over
        // hundreds of periods could otherwise leave a stray cent owed.
        if principal_portion >= balance || period == max_periods {
            principal_portion = balance;
            payment = round_currency(balance + interest_portion);
        }

        balance = round_currency(balance - principal_portion);

        rows.push(AmortizationRow {
            period,
            payment: round_currency(payment),
            extra_payment: round_currency(extra),
            principal_portion: round_currency(principal_portion),
            interest_portion,
            remaining_balance: balance,
        });
    }

    Ok(rows)
}

/// Effect of paying `extra_payment` extra each period versus the loan's
/// unmodified schedule.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ExtraPaymentImpact {
    pub baseline_periods: u32,
    pub payoff_periods: u32,
    pub periods_saved: u32,
    pub baseline_total_interest: Decimal,
    pub total_interest_with_extra: Decimal,
    pub interest_saved: Decimal,
}

/// Compares the loan's baseline schedule against one with `extra_payment`
/// applied every period.
pub fn extra_payment_impact(
    loan: &Loan,
    extra_payment: Decimal,
) -> MortgageResult<ExtraPaymentImpact> {
    let baseline = schedule(loan, Decimal::ZERO)?;
    let with_extra = schedule(loan, extra_payment)?;

    let baseline_total_interest =
        round_currency(baseline.iter().map(|r| r.interest_portion).sum::<Decimal>());
    let total_interest_with_extra = round_currency(
        with_extra
            .iter()
            .map(|r| r.interest_portion)
            .sum::<Decimal>(),
    );

    let baseline_periods = baseline.len() as u32;
    let payoff_periods = with_extra.len() as u32;

    Ok(ExtraPaymentImpact {
        baseline_periods,
        payoff_periods,
        periods_saved: baseline_periods.saturating_sub(payoff_periods),
        baseline_total_interest,
        total_interest_with_extra,
        interest_saved: round_currency(baseline_total_interest - total_interest_with_extra),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LoanBuilder;
    use mortgage_core::PaymentFrequency;
    use rust_decimal_macros::dec;

    fn thirty_year_loan() -> Loan {
        LoanBuilder::default()
            .principal(dec!(300000))
            .annual_rate(dec!(0.06))
            .term_years(dec!(30))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap()
    }

    #[test]
    fn schedule_pays_off_to_exactly_zero() {
        let loan = thirty_year_loan();
        let rows = schedule(&loan, Decimal::ZERO).unwrap();
        assert_eq!(rows.last().unwrap().remaining_balance, Decimal::ZERO);
        assert_eq!(rows.len() as u32, loan.total_periods());
    }

    #[test]
    fn extra_payments_shorten_the_term() {
        let loan = thirty_year_loan();
        let impact = extra_payment_impact(&loan, dec!(300)).unwrap();
        assert!(impact.payoff_periods < impact.baseline_periods);
        assert!(impact.interest_saved > Decimal::ZERO);
    }

    #[test]
    fn zero_extra_payment_matches_baseline_exactly() {
        let loan = thirty_year_loan();
        let impact = extra_payment_impact(&loan, Decimal::ZERO).unwrap();
        assert_eq!(impact.periods_saved, 0);
        assert_eq!(impact.interest_saved, Decimal::ZERO);
    }

    #[test]
    fn rejects_negative_extra_payment_instead_of_corrupting_the_schedule() {
        let loan = thirty_year_loan();
        assert!(matches!(
            schedule(&loan, dec!(-100)),
            Err(MortgageError::InvalidExtraPayment(_))
        ));
        assert!(matches!(
            extra_payment_impact(&loan, dec!(-100)),
            Err(MortgageError::InvalidExtraPayment(_))
        ));
    }

    fn row(
        period: u32,
        payment: &str,
        principal: &str,
        interest: &str,
        balance: &str,
    ) -> AmortizationRow {
        AmortizationRow {
            period,
            payment: payment.parse().unwrap(),
            extra_payment: Decimal::ZERO,
            principal_portion: principal.parse().unwrap(),
            interest_portion: interest.parse().unwrap(),
            remaining_balance: balance.parse().unwrap(),
        }
    }

    #[test]
    fn groups_a_schedule_into_whole_years() {
        let rows: Vec<_> = (1..=24)
            .map(|p| {
                row(
                    p,
                    "1000.00",
                    "400.00",
                    "600.00",
                    &format!("{}", 100_000 - 400 * p),
                )
            })
            .collect();

        let years = summarize_by_year(&rows, 12);

        assert_eq!(years.len(), 2);
        assert_eq!(years[0].year, 1);
        assert_eq!(years[0].paid, dec!(12000));
        assert_eq!(years[0].principal, dec!(4800));
        assert_eq!(years[0].interest, dec!(7200));
        // A level, not a sum: the balance at the end of the year.
        assert_eq!(years[0].remaining_balance, dec!(95200));
        assert_eq!(years[1].remaining_balance, dec!(90400));
    }

    #[test]
    fn a_loan_that_pays_off_mid_year_gets_a_short_final_year() {
        // What any extra payment produces, and what a fixed 12-row chunk
        // assumption would silently drop.
        let rows: Vec<_> = (1..=14)
            .map(|p| row(p, "1000.00", "400.00", "600.00", "0.00"))
            .collect();

        let years = summarize_by_year(&rows, 12);

        assert_eq!(years.len(), 2);
        assert_eq!(years[1].paid, dec!(2000));
    }

    #[test]
    fn yearly_totals_match_the_rows_they_are_made_of() {
        // The yearly figure is displayed beside the periods it sums, so it
        // has to be their total and nothing else.
        let loan = Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.065))
            .term_years(dec!(30))
            .build()
            .unwrap();
        let rows = schedule(&loan, Decimal::ZERO).unwrap();

        let years = summarize_by_year(&rows, 12);
        let total_interest: Decimal = years.iter().map(|y| y.interest).sum();
        let row_interest: Decimal = rows.iter().map(|r| r.interest_portion).sum();

        assert_eq!(total_interest, row_interest);
        assert_eq!(years.len(), 30);
    }

    #[test]
    fn a_zero_period_year_summarizes_to_nothing_rather_than_dividing_by_zero() {
        assert!(summarize_by_year(&[row(1, "1", "1", "0", "0")], 0).is_empty());
    }
}
