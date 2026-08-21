//! `calculate_affordability`: maximum affordable home price.

use mortgage_calc::affordability::AffordabilityInput;
use wasm_bindgen::prelude::*;

use crate::convert::{decimal_to_f64, f64_to_decimal, percent_to_rate, rate_to_percent};
use crate::dto::{AffordabilityParams, AffordabilityResultDto};

#[wasm_bindgen]
pub fn calculate_affordability(params: JsValue) -> JsValue {
    let result = calculate_affordability_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_affordability_impl(params: JsValue) -> AffordabilityResultDto {
    let params: AffordabilityParams = match serde_wasm_bindgen::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return AffordabilityResultDto {
                error: Some(format!("Failed to parse affordability parameters: {e:?}")),
                ..Default::default()
            }
        }
    };

    let (annual_rate, max_dti, annual_property_tax_rate) = match (
        percent_to_rate(params.annual_rate_percent),
        percent_to_rate(params.max_dti_percent),
        percent_to_rate(params.annual_property_tax_rate_percent),
    ) {
        (Some(rate), Some(dti), Some(tax)) => (rate, dti, tax),
        _ => {
            return AffordabilityResultDto {
                error: Some("rate percentages must be finite numbers".to_string()),
                ..Default::default()
            }
        }
    };

    let input = AffordabilityInput {
        gross_monthly_income: f64_to_decimal(params.gross_monthly_income),
        monthly_debts: f64_to_decimal(params.monthly_debts),
        down_payment: f64_to_decimal(params.down_payment),
        annual_rate,
        term_years: f64_to_decimal(params.term_years),
        max_dti,
        annual_property_tax_rate,
        annual_insurance: f64_to_decimal(params.annual_insurance),
        monthly_hoa: f64_to_decimal(params.monthly_hoa),
    };

    match mortgage_calc::affordability::max_affordable(&input) {
        Ok(result) => AffordabilityResultDto {
            max_monthly_housing_payment: Some(decimal_to_f64(result.max_monthly_housing_payment)),
            max_principal_and_interest: Some(decimal_to_f64(result.max_principal_and_interest)),
            max_loan_amount: Some(decimal_to_f64(result.max_loan_amount)),
            max_home_price: Some(decimal_to_f64(result.max_home_price)),
            front_end_dti_percent: Some(rate_to_percent(result.front_end_dti)),
            back_end_dti_percent: Some(rate_to_percent(result.back_end_dti)),
            error: None,
        },
        Err(e) => AffordabilityResultDto {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}
