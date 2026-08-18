//! Conversions across the JS/Rust boundary: JS deals in `f64` and percent
//! strings, Rust deals in `Decimal` fractions.

use mortgage_core::PaymentFrequency;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

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
