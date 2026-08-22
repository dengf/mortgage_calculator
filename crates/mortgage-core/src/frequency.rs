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

    /// Whole months a count of payment periods spans.
    ///
    /// The inverse of [`Self::periods_in_years`], in the unit people
    /// actually think in. Nobody plans around "127 payments"; the emotional
    /// payload of an extra-payment calculator is "you finish ten and a half
    /// years early", and a payment count throws that away.
    ///
    /// Rounded, because 26 fortnightly payments do not divide into months
    /// evenly and a borrower does not care about the remainder.
    pub fn periods_to_months(self, periods: u32) -> u32 {
        (self.periods_to_years(periods) * Decimal::from(12))
            .round()
            .to_u32()
            .unwrap_or(0)
    }

    /// Years, as a fraction, that a count of payment periods spans.
    pub fn periods_to_years(self, periods: u32) -> Decimal {
        Decimal::from(periods) / Decimal::from(self.periods_per_year())
    }

    /// Converts a term expressed in years into a number of payment periods.
    pub fn periods_in_years(self, years: Decimal) -> u32 {
        (years * Decimal::from(self.periods_per_year()))
            .round()
            .to_u32()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn periods_per_year_matches_each_variant() {
        assert_eq!(PaymentFrequency::Monthly.periods_per_year(), 12);
        assert_eq!(PaymentFrequency::BiWeekly.periods_per_year(), 26);
        assert_eq!(PaymentFrequency::Weekly.periods_per_year(), 52);
    }

    #[test]
    fn periodic_rate_divides_the_annual_rate_by_periods_per_year() {
        assert_eq!(
            PaymentFrequency::Monthly.periodic_rate(dec!(0.06)),
            dec!(0.005)
        );
        assert_eq!(
            PaymentFrequency::Weekly.periodic_rate(dec!(0.52)),
            dec!(0.01)
        );
    }

    #[test]
    fn periods_in_years_converts_whole_years_exactly() {
        assert_eq!(PaymentFrequency::Monthly.periods_in_years(dec!(30)), 360);
        assert_eq!(PaymentFrequency::BiWeekly.periods_in_years(dec!(15)), 390);
        assert_eq!(PaymentFrequency::Weekly.periods_in_years(dec!(1)), 52);
    }

    #[test]
    fn periods_in_years_rounds_a_fractional_result_to_the_nearest_period() {
        // 0.05 years * 52 weeks/yr = 2.6 periods, rounds to 3.
        assert_eq!(PaymentFrequency::Weekly.periods_in_years(dec!(0.05)), 3);
    }

    #[test]
    fn periods_in_years_of_zero_is_zero() {
        assert_eq!(PaymentFrequency::Monthly.periods_in_years(dec!(0)), 0);
    }

    #[test]
    fn periods_in_years_clamps_to_zero_on_overflow_instead_of_panicking() {
        // mortgage-calc's LoanBuilder relies on exactly this behavior: an
        // overflowing term must come back as 0 (rejected as InvalidTerm),
        // not panic and not wrap to some other u32 value.
        assert_eq!(
            PaymentFrequency::Weekly.periods_in_years(dec!(999_999_999)),
            0
        );
    }

    #[test]
    fn periods_convert_back_to_the_months_they_span() {
        assert_eq!(PaymentFrequency::Monthly.periods_to_months(360), 360);
        assert_eq!(PaymentFrequency::Weekly.periods_to_months(52), 12);
        // 26 fortnightly payments are a year; 13 are half of one.
        assert_eq!(PaymentFrequency::BiWeekly.periods_to_months(26), 12);
        assert_eq!(PaymentFrequency::BiWeekly.periods_to_months(13), 6);
    }

    #[test]
    fn an_uneven_period_count_rounds_to_the_nearer_month() {
        // 27 fortnights is 12.46 months. A borrower does not care about the
        // remainder, but must not be told a year and a half.
        assert_eq!(PaymentFrequency::BiWeekly.periods_to_months(27), 12);
        assert_eq!(PaymentFrequency::BiWeekly.periods_to_months(30), 14);
    }

    #[test]
    fn no_periods_is_no_time_rather_than_a_panic() {
        assert_eq!(PaymentFrequency::Monthly.periods_to_months(0), 0);
        assert_eq!(PaymentFrequency::Monthly.periods_to_years(0), Decimal::ZERO);
    }

    #[test]
    fn months_round_trip_through_the_periods_that_span_them() {
        for freq in [
            PaymentFrequency::Monthly,
            PaymentFrequency::BiWeekly,
            PaymentFrequency::Weekly,
        ] {
            let periods = freq.periods_in_years(dec!(30));
            assert_eq!(freq.periods_to_months(periods), 360, "{freq:?}");
        }
    }
}
