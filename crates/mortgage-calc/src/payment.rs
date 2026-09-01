//! Standard amortizing payment formula and lifetime cost summary.

use mortgage_core::round_currency;
use rust_decimal::Decimal;
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

    let base = Decimal::ONE + periodic_rate;
    let mut growth = Decimal::ONE;
    for _ in 0..total_periods {
        growth = match growth.checked_mul(base) {
            Some(next) => next,
            // (1+r)^n has overflowed Decimal's range -- no loan in this app
            // is meant to reach a rate this extreme, but nothing upstream
            // guarantees it. At that scale k = r*g/(g-1) is converging on
            // r anyway (g dominates both numerator and denominator), so
            // returning the limit is the correct answer, not a panic.
            None => return periodic_rate,
        };
    }
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
    /// What the borrower pays each period to begin with.
    pub payment: Decimal,
    /// What it becomes after the rate reverts, if it does. `None` for a loan
    /// whose rate holds for the whole term.
    pub payment_after_reversion: Option<Decimal>,
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

    // A loan whose rate holds throughout pays the same amount every period,
    // so the total is a multiplication.
    if loan.reversion().is_none() {
        let total_paid = round_currency(payment * Decimal::from(total_periods));
        return PaymentSummary {
            payment,
            payment_after_reversion: None,
            total_periods,
            total_paid,
            total_interest: round_currency((total_paid - loan.principal()).max(Decimal::ZERO)),
        };
    }

    // A reverting loan does not, so the total is read off the schedule
    // rather than derived a second way. Two routes to the same figure is how
    // a summary ends up disagreeing with the rows it summarizes -- and the
    // schedule is the one that forces the final payment to clear the balance
    // exactly.
    let rows = crate::amortization::schedule(loan, Decimal::ZERO).unwrap_or_default();
    let total_paid = round_currency(rows.iter().map(|r| r.payment).sum::<Decimal>());
    let payment_after_reversion = loan
        .reversion()
        .and_then(|r| rows.get(r.after_periods as usize))
        .map(|row| row.payment);

    PaymentSummary {
        payment,
        payment_after_reversion,
        total_periods: rows.len() as u32,
        total_paid,
        total_interest: round_currency(rows.iter().map(|r| r.interest_portion).sum::<Decimal>()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loan::Reversion;
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
            payment_after_reversion: None,
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
            payment_after_reversion: None,
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
            payment_after_reversion: None,
            total_periods: 12,
            total_paid: dec!(12000),
            total_interest: Decimal::ZERO,
        };
        assert_eq!(free.interest_share(), Some(Decimal::ZERO));
    }

    /// S$400,000 over 25 years: 3M SORA at 1.12% plus 0.30% for two years,
    /// then plus 0.60% -- the shape every Singapore package takes.
    fn sg_package() -> Loan {
        Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.0142))
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Monthly)
            .reversion(Reversion {
                after_periods: 24,
                annual_rate: dec!(0.0172),
            })
            .build()
            .unwrap()
    }

    #[test]
    fn a_reverting_loan_reports_both_payments() {
        let summary = summarize(&sg_package());

        // The promotional instalment amortizes the full 25 years at 1.42%.
        assert_eq!(summary.payment, dec!(1584.75));

        // The reverted one clears what is actually left over the periods
        // actually remaining, at the higher rate -- so it rises, but by far
        // less than the teaser-to-thereafter gap would suggest if the loan
        // were re-quoted from scratch.
        let after = summary.payment_after_reversion.unwrap();
        assert!(
            after > summary.payment,
            "{after} should exceed {}",
            summary.payment
        );
        assert_eq!(after, dec!(1637.12));
    }

    #[test]
    fn a_reverting_loan_costs_more_than_its_teaser_implies() {
        // The whole point. Quoting only the promotional rate understates the
        // lifetime cost, and it is the number a buyer decides on.
        let teaser_only = Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.0142))
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();

        let understated = summarize(&teaser_only).total_paid;
        let actual = summarize(&sg_package()).total_paid;

        assert!(actual > understated, "{actual} vs {understated}");
    }

    #[test]
    fn a_summary_of_a_reverting_loan_matches_its_own_schedule() {
        // The summary is read off the schedule precisely so these cannot
        // drift apart.
        let loan = sg_package();
        let rows = crate::amortization::schedule(&loan, Decimal::ZERO).unwrap();
        let summary = summarize(&loan);

        assert_eq!(
            summary.total_paid,
            round_currency(rows.iter().map(|r| r.payment).sum::<Decimal>())
        );
        assert_eq!(summary.total_periods, rows.len() as u32);
        assert_eq!(rows.last().unwrap().remaining_balance, Decimal::ZERO);
    }

    #[test]
    fn a_loan_with_no_reversion_is_untouched() {
        let plain = Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.065))
            .term_years(dec!(30))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();

        let summary = summarize(&plain);
        assert_eq!(summary.payment_after_reversion, None);
        assert_eq!(
            summary.total_paid,
            round_currency(summary.payment * Decimal::from(360))
        );
    }

    #[test]
    fn a_reversion_that_never_arrives_is_dropped() {
        // Lock-in at or beyond the end of the term. Keeping it would make
        // every downstream calculation re-check the same boundary.
        let loan = Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.0142))
            .term_years(dec!(2))
            .frequency(PaymentFrequency::Monthly)
            .reversion(Reversion {
                after_periods: 24,
                annual_rate: dec!(0.05),
            })
            .build()
            .unwrap();

        assert_eq!(loan.reversion(), None);
        assert_eq!(summarize(&loan).payment_after_reversion, None);
    }
}
