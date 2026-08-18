//! Rate types: a fixed rate, or a floating rate expressed as a base index
//! plus a spread (e.g. "SOFR + 2.5%"). This deliberately does *not* model
//! ARM adjustment schedules, periodic/lifetime caps, or a reset calendar —
//! it's a snapshot: "what does this loan look like at its current effective
//! rate," which is enough to compare a fixed quote against a floating quote
//! side by side. A future `RateType::Arm { .. }` variant with real
//! adjustment-period modeling can be added without touching callers that
//! only care about `effective_rate()`.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RateType {
    Fixed { rate: Decimal },
    Floating { base_rate: Decimal, spread: Decimal },
}

impl RateType {
    /// The single annual rate this resolves to for amortization purposes.
    pub fn effective_rate(&self) -> Decimal {
        match self {
            RateType::Fixed { rate } => *rate,
            RateType::Floating { base_rate, spread } => *base_rate + *spread,
        }
    }
}

/// A named, ready-to-use rate + term combination for quick-adding to a
/// scenario comparison. Illustrative starting points, not live market
/// data — every field is fully editable once added.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatePreset {
    pub label: &'static str,
    pub rate_type: RateType,
    pub term_years: Decimal,
}

pub fn common_presets() -> Vec<RatePreset> {
    use rust_decimal_macros::dec;

    vec![
        RatePreset {
            label: "30-Year Fixed",
            rate_type: RateType::Fixed { rate: dec!(0.065) },
            term_years: dec!(30),
        },
        RatePreset {
            label: "20-Year Fixed",
            rate_type: RateType::Fixed { rate: dec!(0.0625) },
            term_years: dec!(20),
        },
        RatePreset {
            label: "15-Year Fixed",
            rate_type: RateType::Fixed { rate: dec!(0.06) },
            term_years: dec!(15),
        },
        RatePreset {
            label: "Floating: SOFR + 2.00%",
            rate_type: RateType::Floating {
                base_rate: dec!(0.043),
                spread: dec!(0.02),
            },
            term_years: dec!(30),
        },
        RatePreset {
            label: "Floating: SOFR + 2.50%",
            rate_type: RateType::Floating {
                base_rate: dec!(0.043),
                spread: dec!(0.025),
            },
            term_years: dec!(30),
        },
        RatePreset {
            label: "Floating: Prime + 0.50%",
            rate_type: RateType::Floating {
                base_rate: dec!(0.075),
                spread: dec!(0.005),
            },
            term_years: dec!(30),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn floating_rate_sums_base_and_spread() {
        let rate = RateType::Floating {
            base_rate: dec!(0.043),
            spread: dec!(0.025),
        };
        assert_eq!(rate.effective_rate(), dec!(0.068));
    }

    #[test]
    fn fixed_rate_is_itself() {
        let rate = RateType::Fixed { rate: dec!(0.065) };
        assert_eq!(rate.effective_rate(), dec!(0.065));
    }
}
