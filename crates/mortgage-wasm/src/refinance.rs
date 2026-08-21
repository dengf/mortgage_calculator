//! `calculate_refinance`: break-even and lifetime savings analysis.

use mortgage_calc::refinance::RefinanceInput;
use wasm_bindgen::prelude::*;

use crate::convert::{decimal_to_f64, f64_to_decimal, parse_frequency, percent_to_rate};
use crate::dto::{RefinanceParams, RefinanceResultDto};

#[wasm_bindgen]
pub fn calculate_refinance(params: JsValue) -> JsValue {
    let result = calculate_refinance_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_refinance_impl(params: JsValue) -> RefinanceResultDto {
    let params: RefinanceParams = match serde_wasm_bindgen::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return RefinanceResultDto {
                error: Some(format!("Failed to parse refinance parameters: {e:?}")),
                ..Default::default()
            }
        }
    };

    let (current_annual_rate, new_annual_rate) = match (
        percent_to_rate(params.current_annual_rate_percent),
        percent_to_rate(params.new_annual_rate_percent),
    ) {
        (Some(current), Some(new)) => (current, new),
        _ => {
            return RefinanceResultDto {
                error: Some("rate percentages must be finite numbers".to_string()),
                ..Default::default()
            }
        }
    };

    let input = RefinanceInput {
        current_balance: f64_to_decimal(params.current_balance),
        current_annual_rate,
        remaining_periods: params.remaining_periods,
        new_annual_rate,
        new_term_years: f64_to_decimal(params.new_term_years),
        closing_costs: f64_to_decimal(params.closing_costs),
        frequency: parse_frequency(params.frequency.as_deref()),
    };

    match mortgage_calc::refinance::analyze_refinance(&input) {
        Ok(result) => RefinanceResultDto {
            current_payment: Some(decimal_to_f64(result.current_payment)),
            new_payment: Some(decimal_to_f64(result.new_payment)),
            payment_savings: Some(decimal_to_f64(result.payment_savings)),
            break_even_periods: result.break_even_periods,
            remaining_interest_on_current_loan: Some(decimal_to_f64(
                result.remaining_interest_on_current_loan,
            )),
            total_interest_on_new_loan: Some(decimal_to_f64(result.total_interest_on_new_loan)),
            lifetime_savings: Some(decimal_to_f64(result.lifetime_savings)),
            error: None,
        },
        Err(e) => RefinanceResultDto {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}
