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
}
