//! Payment-by-payment amortization schedule and extra-payment payoff impact.

use mortgage_core::{round_currency, MortgageError, MortgageResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::payment::payment_amount;
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

/// Builds the full payment-by-payment schedule for `loan`.
///
/// `extra_payment` is applied on top of the regular payment every period
/// and goes entirely to principal. The schedule ends as soon as the balance
/// reaches zero, which — with a nonzero extra payment — is before the
/// loan's nominal term.
pub fn schedule(loan: &Loan, extra_payment: Decimal) -> MortgageResult<Vec<AmortizationRow>> {
    if extra_payment < Decimal::ZERO {
        return Err(MortgageError::InvalidExtraPayment(extra_payment.to_string()));
    }

    let periodic_rate = loan.periodic_rate();
    let regular_payment = payment_amount(loan);
    let max_periods = loan.total_periods();

    let mut balance = loan.principal();
    let mut rows = Vec::with_capacity(max_periods as usize);

    for period in 1..=max_periods {
        if balance <= Decimal::ZERO {
            break;
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
}
