//! `calculate_comparison` and `get_common_rate_presets`.

use mortgage_calc::comparison::{ComparisonEntry, ComparisonInput, Tradeoff};
use mortgage_calc::{PresetLabel, RateIndex, RateType};
use mortgage_core::Region;
use rust_decimal::Decimal;
use wasm_bindgen::prelude::*;

use crate::convert::{
    decimal_to_f64, f64_to_decimal, parse_frequency, rate_to_percent, rate_type_from_dto,
    rate_type_to_dto, to_js,
};
use crate::dto::{
    ComparisonParams, ComparisonResult, ComparisonRowDto, ComparisonVerdictDto, DescribeRateParams,
    RatePresetDto,
};
use crate::message::Message;

#[wasm_bindgen]
pub fn calculate_comparison(params: JsValue) -> JsValue {
    let result = calculate_comparison_impl(params);
    to_js(&result)
}

fn calculate_comparison_impl(params: JsValue) -> ComparisonResult {
    match serde_wasm_bindgen::from_value(params) {
        Ok(params) => comparison_from_params(params),
        Err(_) => {
            let message = Message::bad_request();
            ComparisonResult {
                error: Some(message.text.clone()),
                error_message: Some(message),
                ..Default::default()
            }
        }
    }
}

/// JsValue-free core of [`calculate_comparison_impl`] — see the matching
/// comment on `payment::payment_from_params`.
fn comparison_from_params(params: ComparisonParams) -> ComparisonResult {
    let input = ComparisonInput {
        principal: f64_to_decimal(params.principal),
        frequency: parse_frequency(params.frequency.as_deref()),
    };

    let entries: Vec<ComparisonEntry> = match params
        .entries
        .iter()
        .map(|entry| {
            Ok(ComparisonEntry {
                label: entry.label.clone(),
                rate_type: rate_type_from_dto(&entry.rate_type)?,
                term_years: f64_to_decimal(entry.term_years),
            })
        })
        .collect::<Result<Vec<_>, String>>()
    {
        Ok(entries) => entries,
        Err(e) => {
            return ComparisonResult {
                error: Some(e),
                ..Default::default()
            }
        }
    };

    match mortgage_calc::comparison::compare(&input, &entries) {
        Ok(rows) => ComparisonResult {
            verdict: mortgage_calc::comparison::verdict(&rows).map(to_verdict_dto),
            rows: rows
                .into_iter()
                .map(|row| ComparisonRowDto {
                    label: row.label,
                    effective_rate_percent: rate_to_percent(row.effective_rate),
                    thereafter_rate_percent: row.thereafter_rate.map(rate_to_percent),
                    payment_after_reversion: row.payment_after_reversion.map(decimal_to_f64),
                    term_years: decimal_to_f64(row.term_years),
                    payment: decimal_to_f64(row.payment),
                    total_periods: row.total_periods,
                    total_paid: decimal_to_f64(row.total_paid),
                    total_interest: decimal_to_f64(row.total_interest),
                })
                .collect(),
            error: None,
            error_message: None,
        },
        Err(e) => ComparisonResult {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

/// Flattens the verdict for the wire.
///
/// `Tradeoff` is an enum in Rust; JS gets a `kind` code plus the fields, so
/// the UI can switch on a stable string rather than sniff which keys are
/// present. `cheaper`/`lighter` collapse onto the single winning row when
/// there is no trade-off to weigh, which keeps the shape uniform.
fn to_verdict_dto(v: mortgage_calc::comparison::ComparisonVerdict) -> ComparisonVerdictDto {
    let (kind, cheaper, lighter, payment_delta, interest_delta) = match v.tradeoff {
        Tradeoff::Outright { row } => ("outright", row, row, 0.0, 0.0),
        Tradeoff::Split {
            cheaper,
            lighter,
            payment_delta,
            interest_delta,
        } => (
            "split",
            cheaper,
            lighter,
            decimal_to_f64(payment_delta),
            decimal_to_f64(interest_delta),
        ),
    };
    ComparisonVerdictDto {
        cheapest_payment: v.cheapest_payment,
        cheapest_interest: v.cheapest_interest,
        cheapest_total_paid: v.cheapest_total_paid,
        kind: kind.to_string(),
        cheaper,
        lighter,
        payment_delta,
        interest_delta,
    }
}

/// Formats a spread for display, e.g. `0.005` as `"0.50"`.
fn spread_percent(spread: Decimal) -> String {
    format!("{:.2}", rate_to_percent(spread))
}

/// The preset's name, as a code plus its values and an English rendering.
fn label_message(label: PresetLabel) -> Message {
    match label {
        PresetLabel::Fixed { years } => Message::with_params(
            "preset.fixed",
            [("years".to_string(), years.to_string())],
            format!("{years}-Year Fixed"),
        ),
        PresetLabel::Floating { index, spread } => {
            let (index, spread) = (index.as_str().to_string(), spread_percent(spread));
            Message::with_params(
                "preset.floating",
                [
                    ("index".to_string(), index.clone()),
                    ("spread".to_string(), spread.clone()),
                ],
                format!("Floating: {index} + {spread}%"),
            )
        }
        PresetLabel::Reverting {
            index,
            initial_spread,
            initial_years,
            thereafter_spread,
        } => {
            let index = index.as_str().to_string();
            let initial = spread_percent(initial_spread);
            let thereafter = spread_percent(thereafter_spread);
            let years = initial_years.normalize().to_string();
            Message::with_params(
                "preset.reverting",
                [
                    ("index".to_string(), index.clone()),
                    ("initial".to_string(), initial.clone()),
                    ("years".to_string(), years.clone()),
                    ("thereafter".to_string(), thereafter.clone()),
                ],
                format!("{index} + {initial}% for {years} yr, then + {thereafter}%"),
            )
        }
        PresetLabel::HdbConcessionary => {
            Message::bare("preset.hdbConcessionary", "HDB concessionary")
        }
    }
}

/// The benchmark a preset names, if it names one.
fn preset_index(label: PresetLabel) -> Option<RateIndex> {
    match label {
        PresetLabel::Floating { index, .. } | PresetLabel::Reverting { index, .. } => Some(index),
        PresetLabel::Fixed { .. } | PresetLabel::HdbConcessionary => None,
    }
}

/// Names a rate the way its preset would, from the row's current figures.
///
/// A preset seeds a row with a name built from its numbers -- "3M SORA +
/// 0.30% for 2 yr, then + 0.60%". Editing those numbers used to leave the
/// name behind, so a row could read "then + 0.60%" while computing 1.50%.
/// The name is regenerated here rather than in the front end so it is built
/// by the same code, with the same rounding, that produced it originally.
///
/// `index` is the published benchmark the row was seeded with; a row the
/// user built from scratch has none and keeps whatever name they gave it.
#[wasm_bindgen]
pub fn describe_rate(params: JsValue) -> JsValue {
    let result = describe_rate_impl(params);
    to_js(&result)
}

fn describe_rate_impl(params: JsValue) -> Option<Message> {
    let params: DescribeRateParams = serde_wasm_bindgen::from_value(params).ok()?;
    describe(&params)
}

/// The JsValue-free core, so this is testable on the native host.
fn describe(params: &DescribeRateParams) -> Option<Message> {
    let index = RateIndex::parse_name(params.index.as_deref()?)?;
    let label = match rate_type_from_dto(&params.rate_type).ok()? {
        RateType::Fixed { .. } => PresetLabel::Fixed {
            years: params.term_years.round().max(0.0) as u32,
        },
        RateType::Floating { spread, .. } => PresetLabel::Floating { index, spread },
        RateType::Reverting {
            initial_spread,
            initial_years,
            thereafter_spread,
            ..
        } => PresetLabel::Reverting {
            index,
            initial_spread,
            initial_years,
            thereafter_spread,
        },
    };
    Some(label_message(label))
}

/// Starting points for the market the buyer is shopping in.
///
/// Takes a region because the floating index is a fact about that market
/// rather than a preference -- see `mortgage_calc::rate::RateIndex`. An
/// unrecognized or absent region falls back to the default rather than
/// erroring: this seeds a comparison, and an empty quick-add list would look
/// like a broken tab.
#[wasm_bindgen]
pub fn get_common_rate_presets(region: Option<String>) -> JsValue {
    let region = region.as_deref().map(Region::parse).unwrap_or_default();

    let presets: Vec<RatePresetDto> = mortgage_calc::common_presets(region)
        .into_iter()
        .map(|preset| {
            let message = label_message(preset.label);
            RatePresetDto {
                label: message.text.clone(),
                label_message: message,
                index: preset_index(preset.label).map(|i| i.as_str().to_string()),
                rate_type: rate_type_to_dto(&preset.rate_type),
                term_years: decimal_to_f64(preset.term_years),
            }
        })
        .collect();

    to_js(&presets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{ComparisonEntryParams, RateTypeDto};

    fn entry(label: &str, rate_percent: f64, term_years: f64) -> ComparisonEntryParams {
        ComparisonEntryParams {
            label: label.to_string(),
            rate_type: RateTypeDto::Fixed { rate_percent },
            term_years,
        }
    }

    #[test]
    fn compares_two_entries_against_the_same_principal() {
        let result = comparison_from_params(ComparisonParams {
            principal: 400_000.0,
            frequency: None,
            entries: vec![
                entry("30-Year Fixed", 6.5, 30.0),
                entry("15-Year Fixed", 6.0, 15.0),
            ],
        });
        assert!(result.error.is_none());
        assert_eq!(result.rows.len(), 2);
        // Shorter term at a similar rate pays off with less total interest.
        assert!(result.rows[1].total_interest < result.rows[0].total_interest);
    }

    #[test]
    fn rejects_an_entry_with_a_non_finite_rate() {
        let result = comparison_from_params(ComparisonParams {
            principal: 400_000.0,
            frequency: None,
            entries: vec![entry("Bad", f64::NAN, 30.0)],
        });
        assert!(result.rows.is_empty());
        assert!(result.error.is_some());
    }

    fn describe_params(
        rate_type: RateTypeDto,
        term_years: f64,
        index: Option<&str>,
    ) -> DescribeRateParams {
        DescribeRateParams {
            rate_type,
            term_years,
            index: index.map(str::to_string),
        }
    }

    #[test]
    fn renames_a_stepping_row_from_its_current_figures() {
        // The defect: a row seeded as "then + 0.60%" that still said so
        // after the spread was edited to 1.50%.
        let described = describe(&describe_params(
            RateTypeDto::Reverting {
                base_rate_percent: 1.12,
                base_floats: true,
                initial_spread_percent: 0.3,
                initial_years: 2.0,
                thereafter_spread_percent: 1.5,
            },
            25.0,
            Some("3M SORA"),
        ))
        .unwrap();

        assert_eq!(described.code, "preset.reverting");
        assert_eq!(described.params.get("thereafter").unwrap(), "1.50");
        assert_eq!(described.params.get("initial").unwrap(), "0.30");
        assert_eq!(described.params.get("index").unwrap(), "3M SORA");
    }

    #[test]
    fn a_row_naming_no_benchmark_keeps_the_name_it_was_given() {
        // Built from scratch rather than seeded, so there is nothing to
        // regenerate from and the user's own name stands.
        assert!(describe(&describe_params(
            RateTypeDto::Fixed { rate_percent: 6.5 },
            30.0,
            None
        ))
        .is_none());
    }

    #[test]
    fn renames_a_floating_row_from_its_spread() {
        let described = describe(&describe_params(
            RateTypeDto::Floating {
                base_rate_percent: 3.63,
                spread_percent: 2.25,
            },
            30.0,
            Some("SOFR"),
        ))
        .unwrap();

        assert_eq!(described.code, "preset.floating");
        assert_eq!(described.params.get("spread").unwrap(), "2.25");
    }

    #[test]
    fn a_preset_publishes_the_benchmark_it_names() {
        // Without this the row has nothing to regenerate its name from.
        assert_eq!(
            preset_index(PresetLabel::Reverting {
                index: RateIndex::Sora,
                initial_spread: Decimal::new(3, 3),
                initial_years: Decimal::from(2),
                thereafter_spread: Decimal::new(6, 3),
            }),
            Some(RateIndex::Sora)
        );
        assert_eq!(preset_index(PresetLabel::HdbConcessionary), None);
    }
}
