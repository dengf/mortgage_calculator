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
    let loan_params: LoanParams = match serde_wasm_bindgen::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return PaymentSummaryResult {
                error: Some(format!("Failed to parse loan parameters: {e:?}")),
                ..Default::default()
            }
        }
    };

    let loan = match build_loan(&loan_params) {
        Ok(loan) => loan,
        Err(e) => {
            return PaymentSummaryResult {
                error: Some(e),
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
    }
}
