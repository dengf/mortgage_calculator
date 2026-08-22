//! `calculate_payment`: monthly payment amount and lifetime cost summary.

use wasm_bindgen::prelude::*;

use crate::convert::decimal_to_f64;
use crate::dto::{LoanParams, PaymentSummaryResult};
use crate::loan::build_loan;

#[wasm_bindgen]
pub fn calculate_payment(params: JsValue) -> JsValue {
    let result = calculate_payment_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_payment_impl(params: JsValue) -> PaymentSummaryResult {
    match serde_wasm_bindgen::from_value(params) {
        Ok(loan_params) => payment_from_params(loan_params),
        Err(e) => PaymentSummaryResult {
            error: Some(format!("Failed to parse loan parameters: {e:?}")),
            ..Default::default()
        },
    }
}

/// The JsValue-free core of [`calculate_payment_impl`] — split out so it
/// can be unit-tested with plain `LoanParams` values, without needing a
/// wasm32 target to construct a `JsValue`.
fn payment_from_params(loan_params: LoanParams) -> PaymentSummaryResult {
    let loan = match build_loan(&loan_params) {
        Ok(loan) => loan,
        Err(e) => {
            return PaymentSummaryResult {
                error: Some(e.text.clone()),
                error_message: Some(e),
                ..Default::default()
            }
        }
    };

    let summary = mortgage_calc::payment::summarize(&loan);

    PaymentSummaryResult {
        payment: Some(decimal_to_f64(summary.payment)),
        total_periods: Some(summary.total_periods),
        total_paid: Some(decimal_to_f64(summary.total_paid)),
        total_interest: Some(decimal_to_f64(summary.total_interest)),
        error: None,
        error_message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_params() -> LoanParams {
        LoanParams {
            principal: 400_000.0,
            annual_rate_percent: 6.5,
            term_years: 30.0,
            frequency: None,
        }
    }

    #[test]
    fn computes_a_payment_for_valid_params() {
        let result = payment_from_params(valid_params());
        assert!(result.error.is_none());
        assert_eq!(result.total_periods, Some(360));
        assert!(result.payment.unwrap() > 0.0);
    }

    #[test]
    fn reports_the_specific_reason_for_an_invalid_loan_instead_of_a_generic_error() {
        let result = payment_from_params(LoanParams {
            annual_rate_percent: -1.0,
            ..valid_params()
        });
        assert!(result.payment.is_none());
        let error = result.error.unwrap();
        assert!(
            error.contains("annual interest rate"),
            "expected a rate-specific reason, got: {error}"
        );
    }
}
