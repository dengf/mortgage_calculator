//! Standard amortizing payment formula and lifetime cost summary.

use mortgage_core::round_currency;
use rust_decimal::{Decimal, MathematicalOps};
use serde::{Deserialize, Serialize};

use crate::Loan;

/// The periodic-payment factor: how much a borrower pays back per dollar
/// borrowed, given a periodic rate `r` over `n` periods.
///
/// `k = r / (1 - (1+r)^-n)`, with the well-known interest-free limit
/// `k = 1/n` as `r -> 0`.
pub(crate) fn payment_factor(periodic_rate: Decimal, total_periods: u32) -> Decimal {
    if periodic_rate == Decimal::ZERO {
        return Decimal::ONE / Decimal::from(total_periods);
    }

    let growth = (Decimal::ONE + periodic_rate).powi(total_periods as i64);
    periodic_rate * growth / (growth - Decimal::ONE)
}

/// The fixed payment amount due each period for `loan`.
pub fn payment_amount(loan: &Loan) -> Decimal {
    let factor = payment_factor(loan.periodic_rate(), loan.total_periods());
    round_currency(loan.principal() * factor)
}

/// Lifetime cost summary for a loan paid on schedule with no extra
/// payments.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaymentSummary {
    pub payment: Decimal,
    pub total_periods: u32,
    pub total_paid: Decimal,
    pub total_interest: Decimal,
}

impl PaymentSummary {
    /// Interest as a percentage of everything the borrower hands over.
    ///
    /// The figure behind "where the money goes": on a long loan at a high
    /// rate it is more than half, which is the single most surprising thing
    /// a first-time buyer learns.
    ///
    /// `None` when nothing is paid at all -- a share of zero is undefined,
    /// not zero percent, and a bar drawn at 0% would assert that none of the
    /// money is interest rather than that there is no money.
    pub fn interest_share(&self) -> Option<Decimal> {
        if self.total_paid <= Decimal::ZERO {
            return None;
        }
        Some(self.total_interest / self.total_paid * Decimal::ONE_HUNDRED)
    }
}

/// Computes the payment amount plus total paid / total interest over the
/// full, unmodified term of `loan`.
pub fn summarize(loan: &Loan) -> PaymentSummary {
    let payment = payment_amount(loan);
    let total_periods = loan.total_periods();
    let total_paid = round_currency(payment * Decimal::from(total_periods));
    let total_interest = round_currency((total_paid - loan.principal()).max(Decimal::ZERO));

    PaymentSummary {
        payment,
        total_periods,
        total_paid,
        total_interest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mortgage_core::PaymentFrequency;
    use rust_decimal_macros::dec;

    #[test]
    fn thirty_year_fixed_matches_known_payment() {
        // $400,000 at 6.5% APR, 30-year monthly — a widely-cited reference
        // value for this formula (~$2,528.27/mo).
        let loan = Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.065))
            .term_years(dec!(30))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();

        let payment = payment_amount(&loan);
        assert_eq!(payment, dec!(2528.27));
    }

    #[test]
    fn zero_rate_loan_splits_principal_evenly() {
        let loan = Loan::builder()
            .principal(dec!(12000))
            .annual_rate(dec!(0))
            .term_years(dec!(1))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();

        assert_eq!(payment_amount(&loan), dec!(1000.00));
        let summary = summarize(&loan);
        assert_eq!(summary.total_interest, dec!(0.00));
    }

    #[test]
    fn summary_interest_is_total_paid_minus_principal() {
        let loan = Loan::builder()
            .principal(dec!(300000))
            .annual_rate(dec!(0.07))
            .term_years(dec!(15))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();

        let summary = summarize(&loan);
        assert_eq!(
            summary.total_paid,
            round_currency(summary.payment * Decimal::from(summary.total_periods))
        );
        assert!(summary.total_interest > Decimal::ZERO);
    }

    #[test]
    fn interest_share_is_the_part_of_the_total_that_is_not_the_house() {
        let summary = PaymentSummary {
            payment: dec!(2528.27),
            total_periods: 360,
            total_paid: dec!(910177.20),
            total_interest: dec!(510177.20),
        };

        // More than half, which is the point of showing it.
        let share = summary.interest_share().unwrap();
        assert!(share > dec!(56) && share < dec!(57), "{share}");
    }

    #[test]
    fn nothing_paid_has_no_share_rather_than_zero_percent() {
        // A bar at 0% asserts that none of the money is interest. There is
        // no money.
        let empty = PaymentSummary {
            payment: Decimal::ZERO,
            total_periods: 0,
            total_paid: Decimal::ZERO,
            total_interest: Decimal::ZERO,
        };
        assert_eq!(empty.interest_share(), None);
    }

    #[test]
    fn an_interest_free_loan_is_a_zero_share_not_an_absent_one() {
        let free = PaymentSummary {
            payment: dec!(1000),
            total_periods: 12,
            total_paid: dec!(12000),
            total_interest: Decimal::ZERO,
        };
        assert_eq!(free.interest_share(), Some(Decimal::ZERO));
    }
}
