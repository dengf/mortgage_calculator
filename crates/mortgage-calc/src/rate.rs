//! Rate types: a fixed rate, or a floating rate expressed as a base index
//! plus a spread (e.g. "SOFR + 2.5%"). This deliberately does *not* model
//! ARM adjustment schedules, periodic/lifetime caps, or a reset calendar —
//! it's a snapshot: "what does this loan look like at its current effective
//! rate," which is enough to compare a fixed quote against a floating quote
//! side by side. A future `RateType::Arm { .. }` variant with real
//! adjustment-period modeling can be added without touching callers that
//! only care about `effective_rate()`.

use mortgage_core::Region;
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

/// A published index a floating rate is quoted against.
///
/// Which one applies is a fact about the market the property is in, not a
/// preference. SGD loans have not referenced SIBOR since MAS discontinued it
/// immediately after 31 December 2024; SORA, which MAS administers, replaced
/// both SIBOR and SOR. Offering a Singapore buyer a SOFR or Prime quote is
/// not a translation problem -- it is the wrong index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateIndex {
    /// Secured Overnight Financing Rate (USD).
    Sofr,
    /// US bank prime loan rate.
    Prime,
    /// Singapore Overnight Rate Average, administered by MAS. Home loans
    /// quote the 3-month compounded series.
    Sora,
}

impl RateIndex {
    /// How the index is written. These are proper nouns and stay in Latin
    /// script in every locale we ship, including the Chinese ones -- a
    /// Singapore bank's own term sheet says "3M SORA".
    pub fn as_str(self) -> &'static str {
        match self {
            RateIndex::Sofr => "SOFR",
            RateIndex::Prime => "Prime",
            RateIndex::Sora => "3M SORA",
        }
    }
}

/// What a preset is called, as a structure rather than a sentence.
///
/// These used to be `&'static str` English labels baked into the core, which
/// meant a reader in Chinese was offered "30-Year Fixed" in the middle of a
/// translated page. The UI composes the wording from this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetLabel {
    Fixed {
        years: u32,
    },
    Floating {
        index: RateIndex,
        spread: Decimal,
    },
    /// HDB's concessionary loan, which is a policy rate rather than a market
    /// quote: pegged at 0.1% above the CPF Ordinary Account rate.
    HdbConcessionary,
}

/// A named, ready-to-use rate + term combination for quick-adding to a
/// scenario comparison. Illustrative starting points, not live market
/// data — every field is fully editable once added.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RatePreset {
    pub label: PresetLabel,
    pub rate_type: RateType,
    pub term_years: Decimal,
}

/// Starting points for the region the buyer is shopping in.
///
/// Rates are illustrative and every field is editable once added, but the
/// *index* is not a matter of taste, and neither are the term limits. Values
/// were checked on 2026-08-22; the sources are named per region below.
pub fn common_presets(region: Region) -> Vec<RatePreset> {
    match region {
        Region::US => us_presets(),
        Region::SG => sg_presets(),
    }
}

fn us_presets() -> Vec<RatePreset> {
    use rust_decimal_macros::dec;

    vec![
        RatePreset {
            label: PresetLabel::Fixed { years: 30 },
            rate_type: RateType::Fixed { rate: dec!(0.065) },
            term_years: dec!(30),
        },
        RatePreset {
            label: PresetLabel::Fixed { years: 20 },
            rate_type: RateType::Fixed { rate: dec!(0.0625) },
            term_years: dec!(20),
        },
        RatePreset {
            label: PresetLabel::Fixed { years: 15 },
            rate_type: RateType::Fixed { rate: dec!(0.06) },
            term_years: dec!(15),
        },
        // SOFR 3.63% (20 Aug 2026) and Prime 6.75% (Federal Reserve H.15,
        // release of 21 Aug 2026). Both were stale by roughly 70 basis
        // points before this: SOFR sat at 4.30% and Prime at 7.50%.
        RatePreset {
            label: PresetLabel::Floating {
                index: RateIndex::Sofr,
                spread: dec!(0.02),
            },
            rate_type: RateType::Floating {
                base_rate: dec!(0.0363),
                spread: dec!(0.02),
            },
            term_years: dec!(30),
        },
        RatePreset {
            label: PresetLabel::Floating {
                index: RateIndex::Sofr,
                spread: dec!(0.025),
            },
            rate_type: RateType::Floating {
                base_rate: dec!(0.0363),
                spread: dec!(0.025),
            },
            term_years: dec!(30),
        },
        RatePreset {
            label: PresetLabel::Floating {
                index: RateIndex::Prime,
                spread: dec!(0.005),
            },
            rate_type: RateType::Floating {
                base_rate: dec!(0.0675),
                spread: dec!(0.005),
            },
            term_years: dec!(30),
        },
    ]
}

/// Singapore starting points.
///
/// Deliberately no "fixed for the whole term" package. A Singapore fixed-rate
/// home loan fixes for two or three years and then reverts to a floating
/// spread, and this module models a single rate for the life of the loan. A
/// 1.35% teaser stretched across 25 years would produce a payment no bank
/// would honour, and would contradict the affordability tab in this same app,
/// which stress-tests at MAS Notice 645's floor of the higher of 4% and the
/// thereafter rate.
///
/// 3-month compounded SORA was 1.12% on 6 August 2026; spreads on offer run
/// roughly 0.50%–1.00%. The HDB concessionary rate is 2.6%, pegged at 0.1%
/// above the CPF Ordinary Account rate, which sits at its 2.5% floor for
/// 1 July–30 September 2026 (CPF Board). Unlike the bank packages it really
/// is fixed for the life of the loan.
fn sg_presets() -> Vec<RatePreset> {
    use rust_decimal_macros::dec;

    vec![
        RatePreset {
            label: PresetLabel::Floating {
                index: RateIndex::Sora,
                spread: dec!(0.005),
            },
            rate_type: RateType::Floating {
                base_rate: dec!(0.0112),
                spread: dec!(0.005),
            },
            term_years: dec!(30),
        },
        RatePreset {
            label: PresetLabel::Floating {
                index: RateIndex::Sora,
                spread: dec!(0.01),
            },
            rate_type: RateType::Floating {
                base_rate: dec!(0.0112),
                spread: dec!(0.01),
            },
            term_years: dec!(30),
        },
        RatePreset {
            // 25 years, not 30: that is the ceiling on an HDB loan.
            label: PresetLabel::HdbConcessionary,
            rate_type: RateType::Fixed { rate: dec!(0.026) },
            term_years: dec!(25),
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

    #[test]
    fn singapore_is_never_offered_a_us_index() {
        // SIBOR was discontinued after 31 December 2024 and SORA replaced it.
        // Quoting a Singapore buyer SOFR or Prime is not a wording problem.
        for preset in common_presets(Region::SG) {
            if let PresetLabel::Floating { index, .. } = preset.label {
                assert_eq!(index, RateIndex::Sora, "{preset:?}");
            }
        }
    }

    #[test]
    fn the_us_keeps_its_own_indices() {
        let indices: Vec<_> = common_presets(Region::US)
            .into_iter()
            .filter_map(|p| match p.label {
                PresetLabel::Floating { index, .. } => Some(index),
                _ => None,
            })
            .collect();

        assert!(indices.contains(&RateIndex::Sofr));
        assert!(indices.contains(&RateIndex::Prime));
        assert!(!indices.contains(&RateIndex::Sora));
    }

    #[test]
    fn no_singapore_preset_fixes_a_teaser_rate_for_the_whole_term() {
        // A SGD fixed package fixes for two or three years and then reverts.
        // Stretching the teaser across the term would produce a payment no
        // bank would honour, and would contradict this app's own MAS Notice
        // 645 stress floor of the higher of 4% and the thereafter rate.
        for preset in common_presets(Region::SG) {
            if let RateType::Fixed { rate } = preset.rate_type {
                assert_eq!(
                    preset.label,
                    PresetLabel::HdbConcessionary,
                    "the only genuinely term-long fixed SGD rate is HDB's"
                );
                assert_eq!(rate, dec!(0.026));
            }
        }
    }

    #[test]
    fn the_hdb_preset_respects_the_25_year_ceiling() {
        let hdb = common_presets(Region::SG)
            .into_iter()
            .find(|p| p.label == PresetLabel::HdbConcessionary)
            .expect("SG presets include the HDB concessionary loan");

        assert_eq!(hdb.term_years, dec!(25));
    }

    #[test]
    fn every_preset_resolves_to_a_usable_rate() {
        for region in [Region::US, Region::SG] {
            for preset in common_presets(region) {
                let rate = preset.rate_type.effective_rate();
                assert!(rate > Decimal::ZERO, "{region:?} {preset:?}");
                assert!(rate < dec!(0.25), "{region:?} {preset:?}");
                assert!(preset.term_years > Decimal::ZERO);
            }
        }
    }

    #[test]
    fn a_floating_label_reports_the_spread_it_was_built_with() {
        // The label and the arithmetic must not drift: a row reading
        // "3M SORA + 0.50%" that computes a different spread is worse than
        // no label at all.
        for region in [Region::US, Region::SG] {
            for preset in common_presets(region) {
                if let (
                    PresetLabel::Floating {
                        spread: labelled, ..
                    },
                    RateType::Floating { spread: used, .. },
                ) = (preset.label, preset.rate_type)
                {
                    assert_eq!(labelled, used, "{preset:?}");
                }
            }
        }
    }
}
