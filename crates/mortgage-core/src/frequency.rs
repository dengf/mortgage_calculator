use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// How often payments are made against the loan.
///
/// Bi-weekly payments are the classic "pay it off early" trick: 26
/// half-sized payments a year add up to 13 monthly payments instead of 12,
/// without the borrower consciously budgeting an extra payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaymentFrequency {
    #[default]
    Monthly,
    BiWeekly,
    Weekly,
}

impl PaymentFrequency {
    /// Number of payment periods in a year.
    pub fn periods_per_year(self) -> u32 {
        match self {
            PaymentFrequency::Monthly => 12,
            PaymentFrequency::BiWeekly => 26,
            PaymentFrequency::Weekly => 52,
        }
    }

    /// Converts an annual nominal interest rate (e.g. `0.065` for 6.5%)
    /// into the rate charged per payment period.
    pub fn periodic_rate(self, annual_rate: Decimal) -> Decimal {
        annual_rate / Decimal::from(self.periods_per_year())
    }

    /// Converts a term expressed in years into a number of payment periods.
    pub fn periods_in_years(self, years: Decimal) -> u32 {
        (years * Decimal::from(self.periods_per_year()))
            .round()
            .to_u32()
            .unwrap_or(0)
    }
}
