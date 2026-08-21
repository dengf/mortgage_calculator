//! Conversions across the JS/Rust boundary: JS deals in `f64` and percent
//! strings, Rust deals in `Decimal` fractions.

use mortgage_calc::RateType;
use mortgage_core::PaymentFrequency;
use mortgage_ports::CalculatorKind;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use crate::dto::RateTypeDto;

/// Converts a percentage as entered by a user (`6.5` for 6.5%) into the
/// fractional rate the calc crate expects (`0.065`). Returns `None` for
/// non-finite input (NaN/Infinity, e.g. from a failed `parseFloat` on the
/// JS side) instead of silently treating it as `0.0` — unlike
/// principal/term-years, a rate of exactly zero is a legitimate value the
/// calc crate accepts, so a NaN-coerced-to-zero rate would otherwise slip
/// through as a plausible-looking (but wrong) 0% calculation.
pub fn percent_to_rate(percent: f64) -> Option<Decimal> {
    if !percent.is_finite() {
        return None;
    }
    Some(f64_to_decimal(percent) / Decimal::from(100))
}

/// Converts a fractional rate (`0.065`) back into a display percentage
/// (`6.5`).
pub fn rate_to_percent(rate: Decimal) -> f64 {
    decimal_to_f64(rate * Decimal::from(100))
}

pub fn f64_to_decimal(value: f64) -> Decimal {
    Decimal::from_f64(value).unwrap_or_default()
}

pub fn decimal_to_f64(value: Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}

/// Parses the frontend's `"monthly" | "biweekly" | "weekly"` strings,
/// defaulting to monthly for anything unrecognized (including `None`).
pub fn parse_frequency(frequency: Option<&str>) -> PaymentFrequency {
    match frequency.map(str::to_lowercase).as_deref() {
        Some("biweekly") | Some("bi-weekly") => PaymentFrequency::BiWeekly,
        Some("weekly") => PaymentFrequency::Weekly,
        _ => PaymentFrequency::Monthly,
    }
}

pub fn rate_type_from_dto(dto: &RateTypeDto) -> Result<RateType, String> {
    const NOT_FINITE: &str = "rate percentages must be finite numbers";
    match *dto {
        RateTypeDto::Fixed { rate_percent } => Ok(RateType::Fixed {
            rate: percent_to_rate(rate_percent).ok_or(NOT_FINITE)?,
        }),
        RateTypeDto::Floating {
            base_rate_percent,
            spread_percent,
        } => Ok(RateType::Floating {
            base_rate: percent_to_rate(base_rate_percent).ok_or(NOT_FINITE)?,
            spread: percent_to_rate(spread_percent).ok_or(NOT_FINITE)?,
        }),
    }
}

pub fn rate_type_to_dto(rate_type: &RateType) -> RateTypeDto {
    match *rate_type {
        RateType::Fixed { rate } => RateTypeDto::Fixed {
            rate_percent: rate_to_percent(rate),
        },
        RateType::Floating { base_rate, spread } => RateTypeDto::Floating {
            base_rate_percent: rate_to_percent(base_rate),
            spread_percent: rate_to_percent(spread),
        },
    }
}

/// Parses the frontend's `"payment" | "amortization" | "affordability" |
/// "refinance" | "comparison"` strings.
pub fn parse_calculator_kind(calculator: &str) -> Result<CalculatorKind, String> {
    match calculator {
        "payment" => Ok(CalculatorKind::Payment),
        "amortization" => Ok(CalculatorKind::Amortization),
        "affordability" => Ok(CalculatorKind::Affordability),
        "refinance" => Ok(CalculatorKind::Refinance),
        "comparison" => Ok(CalculatorKind::Comparison),
        other => Err(format!("unknown calculator kind: {other}")),
    }
}

pub fn calculator_kind_to_str(kind: CalculatorKind) -> &'static str {
    match kind {
        CalculatorKind::Payment => "payment",
        CalculatorKind::Amortization => "amortization",
        CalculatorKind::Affordability => "affordability",
        CalculatorKind::Refinance => "refinance",
        CalculatorKind::Comparison => "comparison",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_to_rate_converts_a_normal_value() {
        assert_eq!(
            percent_to_rate(6.5),
            Some(Decimal::try_from(0.065).unwrap())
        );
    }

    #[test]
    fn percent_to_rate_rejects_nan_and_infinity_instead_of_defaulting_to_zero() {
        assert_eq!(percent_to_rate(f64::NAN), None);
        assert_eq!(percent_to_rate(f64::INFINITY), None);
        assert_eq!(percent_to_rate(f64::NEG_INFINITY), None);
    }

    #[test]
    fn rate_type_from_dto_propagates_a_non_finite_fixed_rate_as_an_error() {
        let dto = RateTypeDto::Fixed {
            rate_percent: f64::NAN,
        };
        assert!(rate_type_from_dto(&dto).is_err());
    }

    #[test]
    fn rate_type_from_dto_propagates_a_non_finite_floating_rate_as_an_error() {
        let dto = RateTypeDto::Floating {
            base_rate_percent: 4.5,
            spread_percent: f64::NAN,
        };
        assert!(rate_type_from_dto(&dto).is_err());
    }
}
