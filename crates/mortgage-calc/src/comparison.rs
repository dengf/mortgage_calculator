//! Side-by-side scenario comparison: run the same principal through several
//! (rate type, term) combinations and line up the results. Pure
//! composition over [`crate::payment`] and [`Loan`] — no new loan math.

use mortgage_core::{MortgageResult, PaymentFrequency};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::payment::summarize;
use crate::rate::RateType;
use crate::Loan;

/// The principal and payment cadence shared by every scenario in a
/// comparison — only rate type and term vary per entry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ComparisonInput {
    pub principal: Decimal,
    pub frequency: PaymentFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonEntry {
    pub label: String,
    pub rate_type: RateType,
    pub term_years: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub label: String,
    pub effective_rate: Decimal,
    pub term_years: Decimal,
    pub payment: Decimal,
    pub total_periods: u32,
    pub total_paid: Decimal,
    pub total_interest: Decimal,
}

/// Which row wins on each measure, by index into the comparison's rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonVerdict {
    pub cheapest_payment: usize,
    pub cheapest_interest: usize,
    pub cheapest_total_paid: usize,
    pub tradeoff: Tradeoff,
}

/// The relationship between the two ways of being cheapest.
///
/// A comparison table lines up two columns of figures and leaves the reader
/// to do the subtraction. The whole question being asked is "which, and by
/// how much" -- so it is answered here rather than left implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tradeoff {
    /// One option is cheapest both per payment and over the loan's life.
    /// There is nothing to weigh up.
    Outright { row: usize },
    /// The classic fixed-term trade-off: the option that costs least overall
    /// costs most each period.
    Split {
        /// Cheapest over the life of the loan.
        cheaper: usize,
        /// Lightest per payment.
        lighter: usize,
        /// What the cheaper option adds to each payment.
        payment_delta: Decimal,
        /// What the lighter option adds in total interest.
        interest_delta: Decimal,
    },
}

/// Index of the row with the lowest value of `measure`.
///
/// Ties keep the earliest row, so the answer follows the order the user
/// entered their scenarios in rather than an arbitrary one.
fn lowest_by(
    rows: &[ComparisonResult],
    measure: impl Fn(&ComparisonResult) -> Decimal,
) -> Option<usize> {
    rows.iter()
        .enumerate()
        .reduce(|best, current| {
            if measure(current.1) < measure(best.1) {
                current
            } else {
                best
            }
        })
        .map(|(index, _)| index)
}

/// Reads a finished comparison: which row wins on each measure, and what the
/// choice between them costs.
///
/// `None` for fewer than two rows -- one scenario compared against nothing
/// has no verdict to give, and saying "this one is cheapest" of a table with
/// a single line tells the reader nothing.
pub fn verdict(rows: &[ComparisonResult]) -> Option<ComparisonVerdict> {
    if rows.len() < 2 {
        return None;
    }
    let cheapest_payment = lowest_by(rows, |r| r.payment)?;
    let cheapest_interest = lowest_by(rows, |r| r.total_interest)?;
    let cheapest_total_paid = lowest_by(rows, |r| r.total_paid)?;

    let tradeoff = if cheapest_payment == cheapest_interest {
        Tradeoff::Outright {
            row: cheapest_interest,
        }
    } else {
        Tradeoff::Split {
            cheaper: cheapest_interest,
            lighter: cheapest_payment,
            payment_delta: rows[cheapest_interest].payment - rows[cheapest_payment].payment,
            interest_delta: rows[cheapest_payment].total_interest
                - rows[cheapest_interest].total_interest,
        }
    };

    Some(ComparisonVerdict {
        cheapest_payment,
        cheapest_interest,
        cheapest_total_paid,
        tradeoff,
    })
}

/// Computes a [`ComparisonResult`] row for each `entries` scenario against
/// the shared `input`. Fails on the first invalid entry (e.g. a negative
/// rate or zero term).
pub fn compare(
    input: &ComparisonInput,
    entries: &[ComparisonEntry],
) -> MortgageResult<Vec<ComparisonResult>> {
    entries
        .iter()
        .map(|entry| {
            let loan = Loan::builder()
                .principal(input.principal)
                .rate_type(entry.rate_type)
                .term_years(entry.term_years)
                .frequency(input.frequency)
                .build()?;

            let summary = summarize(&loan);

            Ok(ComparisonResult {
                label: entry.label.clone(),
                effective_rate: entry.rate_type.effective_rate(),
                term_years: entry.term_years,
                payment: summary.payment,
                total_periods: summary.total_periods,
                total_paid: summary.total_paid,
                total_interest: summary.total_interest,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn shorter_term_has_higher_payment_but_less_interest() {
        let input = ComparisonInput {
            principal: dec!(400000),
            frequency: PaymentFrequency::Monthly,
        };
        let entries = vec![
            ComparisonEntry {
                label: "30yr".into(),
                rate_type: RateType::Fixed { rate: dec!(0.065) },
                term_years: dec!(30),
            },
            ComparisonEntry {
                label: "15yr".into(),
                rate_type: RateType::Fixed { rate: dec!(0.06) },
                term_years: dec!(15),
            },
        ];

        let results = compare(&input, &entries).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[1].payment > results[0].payment);
        assert!(results[1].total_interest < results[0].total_interest);
    }

    #[test]
    fn fixed_vs_floating_at_same_effective_rate_match() {
        let input = ComparisonInput {
            principal: dec!(300000),
            frequency: PaymentFrequency::Monthly,
        };
        let entries = vec![
            ComparisonEntry {
                label: "Fixed 6.5%".into(),
                rate_type: RateType::Fixed { rate: dec!(0.065) },
                term_years: dec!(30),
            },
            ComparisonEntry {
                label: "Floating SOFR+2.5% (=6.5% today)".into(),
                rate_type: RateType::Floating {
                    base_rate: dec!(0.04),
                    spread: dec!(0.025),
                },
                term_years: dec!(30),
            },
        ];

        let results = compare(&input, &entries).unwrap();
        assert_eq!(results[0].payment, results[1].payment);
    }

    fn result(
        label: &str,
        payment: &str,
        total_paid: &str,
        total_interest: &str,
    ) -> ComparisonResult {
        ComparisonResult {
            label: label.into(),
            effective_rate: dec!(0.065),
            term_years: dec!(30),
            payment: payment.parse().unwrap(),
            total_periods: 360,
            total_paid: total_paid.parse().unwrap(),
            total_interest: total_interest.parse().unwrap(),
        }
    }

    #[test]
    fn a_single_row_has_no_verdict() {
        // "Cheapest" of a one-line table tells the reader nothing.
        assert!(verdict(&[result("30yr", "2528.27", "910177.20", "510177.20")]).is_none());
        assert!(verdict(&[]).is_none());
    }

    #[test]
    fn names_the_split_between_lighter_payments_and_less_interest() {
        let rows = vec![
            result("30yr", "2528.27", "910177.20", "510177.20"),
            result("15yr", "3375.43", "607577.40", "207577.40"),
        ];

        let v = verdict(&rows).unwrap();

        assert_eq!(v.cheapest_payment, 0);
        assert_eq!(v.cheapest_interest, 1);
        assert_eq!(v.cheapest_total_paid, 1);
        assert_eq!(
            v.tradeoff,
            Tradeoff::Split {
                cheaper: 1,
                lighter: 0,
                payment_delta: dec!(847.16),
                interest_delta: dec!(302599.80),
            }
        );
    }

    #[test]
    fn reports_an_outright_winner_when_one_row_takes_both() {
        let rows = vec![
            result("expensive", "3000.00", "1080000.00", "680000.00"),
            result("better", "2500.00", "900000.00", "500000.00"),
        ];

        let v = verdict(&rows).unwrap();

        assert_eq!(v.tradeoff, Tradeoff::Outright { row: 1 });
    }

    #[test]
    fn a_tie_keeps_the_row_the_user_entered_first() {
        let rows = vec![
            result("first", "2500.00", "900000.00", "500000.00"),
            result("identical", "2500.00", "900000.00", "500000.00"),
        ];

        let v = verdict(&rows).unwrap();

        assert_eq!(v.cheapest_payment, 0);
        assert_eq!(v.tradeoff, Tradeoff::Outright { row: 0 });
    }
}
