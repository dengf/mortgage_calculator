//! Conversions across the JS/Rust boundary: JS deals in `f64` and percent
//! strings, Rust deals in `Decimal` fractions.

use mortgage_calc::RateType;
use mortgage_core::PaymentFrequency;
use mortgage_ports::CalculatorKind;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use crate::dto::RateTypeDto;

/// Converts a percentage as entered by a user (`6.5` for 6.5%) into the
/// fractional rate the calc crate expects (`0.065`).
pub fn percent_to_rate(percent: f64) -> Decimal {
    f64_to_decimal(percent) / Decimal::from(100)
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

pub fn rate_type_from_dto(dto: &RateTypeDto) -> RateType {
    match *dto {
        RateTypeDto::Fixed { rate_percent } => RateType::Fixed {
            rate: percent_to_rate(rate_percent),
        },
        RateTypeDto::Floating {
            base_rate_percent,
            spread_percent,
        } => RateType::Floating {
            base_rate: percent_to_rate(base_rate_percent),
            spread: percent_to_rate(spread_percent),
        },
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
