//! `calculate_refinance`: break-even and lifetime savings analysis.

use mortgage_calc::refinance::RefinanceInput;
use wasm_bindgen::prelude::*;

use crate::convert::{decimal_to_f64, f64_to_decimal, parse_frequency, percent_to_rate, to_js};
use crate::dto::{RefinanceParams, RefinanceResultDto};
use crate::message::Message;

#[wasm_bindgen]
pub fn calculate_refinance(params: JsValue) -> JsValue {
    let result = calculate_refinance_impl(params);
    to_js(&result)
}

fn calculate_refinance_impl(params: JsValue) -> RefinanceResultDto {
    match serde_wasm_bindgen::from_value(params) {
        Ok(params) => refinance_from_params(params),
        Err(_) => {
            let message = Message::bad_request();
            RefinanceResultDto {
                error: Some(message.text.clone()),
                error_message: Some(message),
                ..Default::default()
            }
        }
    }
}

/// JsValue-free core of [`calculate_refinance_impl`] — see the matching
/// comment on `payment::payment_from_params`.
fn refinance_from_params(params: RefinanceParams) -> RefinanceResultDto {
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
            error_message: None,
        },
        Err(e) => RefinanceResultDto {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_params() -> RefinanceParams {
        RefinanceParams {
            current_balance: 300_000.0,
            current_annual_rate_percent: 7.5,
            remaining_periods: 300,
            new_annual_rate_percent: 6.0,
            new_term_years: 30.0,
            closing_costs: 6000.0,
            frequency: None,
        }
    }

    #[test]
    fn computes_savings_when_the_new_rate_is_lower() {
        let result = refinance_from_params(valid_params());
        assert!(result.error.is_none());
        assert!(result.payment_savings.unwrap() > 0.0);
        assert!(result.break_even_periods.is_some());
    }

    #[test]
    fn rejects_a_non_finite_rate_instead_of_treating_it_as_zero() {
        let result = refinance_from_params(RefinanceParams {
            new_annual_rate_percent: f64::INFINITY,
            ..valid_params()
        });
        assert!(result.payment_savings.is_none());
        assert!(result.error.is_some());
    }
}
