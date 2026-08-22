//! `calculate_comparison` and `get_common_rate_presets`.

use mortgage_calc::comparison::{ComparisonEntry, ComparisonInput};
use wasm_bindgen::prelude::*;

use crate::convert::{
    decimal_to_f64, f64_to_decimal, parse_frequency, rate_to_percent, rate_type_from_dto,
    rate_type_to_dto,
};
use crate::dto::{ComparisonParams, ComparisonResult, ComparisonRowDto, RatePresetDto};

#[wasm_bindgen]
pub fn calculate_comparison(params: JsValue) -> JsValue {
    let result = calculate_comparison_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_comparison_impl(params: JsValue) -> ComparisonResult {
    match serde_wasm_bindgen::from_value(params) {
        Ok(params) => comparison_from_params(params),
        Err(e) => ComparisonResult {
            error: Some(format!("Failed to parse comparison parameters: {e:?}")),
            ..Default::default()
        },
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
