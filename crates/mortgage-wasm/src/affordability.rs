//! `calculate_affordability`: maximum affordable home price.

use mortgage_calc::affordability::AffordabilityInput;
use wasm_bindgen::prelude::*;

use crate::convert::{decimal_to_f64, f64_to_decimal, percent_to_rate, rate_to_percent};
use crate::dto::{AffordabilityParams, AffordabilityResultDto};
use crate::message::Message;

#[wasm_bindgen]
pub fn calculate_affordability(params: JsValue) -> JsValue {
    let result = calculate_affordability_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_affordability_impl(params: JsValue) -> AffordabilityResultDto {
    match serde_wasm_bindgen::from_value(params) {
        Ok(params) => affordability_from_params(params),
        Err(_) => {
            let message = Message::bad_request();
            AffordabilityResultDto {
                error: Some(message.text.clone()),
                error_message: Some(message),
                ..Default::default()
            }
        }
    }
}

/// JsValue-free core of [`calculate_affordability_impl`] — see the matching
/// comment on `payment::payment_from_params`.
fn affordability_from_params(params: AffordabilityParams) -> AffordabilityResultDto {
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
            error_message: None,
        },
        Err(e) => AffordabilityResultDto {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_params() -> AffordabilityParams {
        AffordabilityParams {
            gross_monthly_income: 10_000.0,
            monthly_debts: 500.0,
            down_payment: 60_000.0,
            annual_rate_percent: 6.5,
            term_years: 30.0,
            max_dti_percent: 36.0,
            annual_property_tax_rate_percent: 1.2,
            annual_insurance: 1500.0,
            monthly_hoa: 0.0,
        }
    }

    #[test]
    fn computes_a_max_home_price_for_valid_params() {
        let result = affordability_from_params(valid_params());
        assert!(result.error.is_none());
        assert!(result.max_home_price.unwrap() > 0.0);
    }

    #[test]
    fn rejects_a_non_finite_rate_instead_of_treating_it_as_zero() {
        let result = affordability_from_params(AffordabilityParams {
            annual_rate_percent: f64::NAN,
            ..valid_params()
        });
        assert!(result.max_home_price.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn forwards_the_specific_reason_for_an_invalid_dti_ratio() {
        let result = affordability_from_params(AffordabilityParams {
            max_dti_percent: 0.0,
            ..valid_params()
        });
        let error = result.error.unwrap();
        assert!(
            error.contains("debt-to-income"),
            "expected a DTI-specific reason, got: {error}"
        );
    }
}
