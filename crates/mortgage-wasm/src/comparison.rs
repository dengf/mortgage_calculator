//! `calculate_comparison` and `get_common_rate_presets`.

use mortgage_calc::comparison::{ComparisonEntry, ComparisonInput, Tradeoff};
use wasm_bindgen::prelude::*;

use crate::convert::{
    decimal_to_f64, f64_to_decimal, parse_frequency, rate_to_percent, rate_type_from_dto,
    rate_type_to_dto,
};
use crate::dto::{
    ComparisonParams, ComparisonResult, ComparisonRowDto, ComparisonVerdictDto, RatePresetDto,
};
use crate::message::Message;

#[wasm_bindgen]
pub fn calculate_comparison(params: JsValue) -> JsValue {
    let result = calculate_comparison_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
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

#[wasm_bindgen]
pub fn get_common_rate_presets() -> JsValue {
    let presets: Vec<RatePresetDto> = mortgage_calc::common_presets()
        .into_iter()
        .map(|preset| RatePresetDto {
            label: preset.label.to_string(),
            rate_type: rate_type_to_dto(&preset.rate_type),
            term_years: decimal_to_f64(preset.term_years),
        })
        .collect();

    serde_wasm_bindgen::to_value(&presets).unwrap_or(JsValue::NULL)
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
}
