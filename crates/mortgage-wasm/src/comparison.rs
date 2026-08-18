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
    let params: ComparisonParams = match serde_wasm_bindgen::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return ComparisonResult {
                error: Some(format!("Failed to parse comparison parameters: {e:?}")),
                ..Default::default()
            }
        }
    };

    let input = ComparisonInput {
        principal: f64_to_decimal(params.principal),
        frequency: parse_frequency(params.frequency.as_deref()),
    };

    let entries: Vec<ComparisonEntry> = params
        .entries
        .iter()
        .map(|entry| ComparisonEntry {
            label: entry.label.clone(),
            rate_type: rate_type_from_dto(&entry.rate_type),
            term_years: f64_to_decimal(entry.term_years),
        })
        .collect();

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
