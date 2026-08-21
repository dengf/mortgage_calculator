//! `calculate_amortization_schedule` and `calculate_extra_payment_impact`.

use rust_decimal::Decimal;
use wasm_bindgen::prelude::*;

use crate::convert::{decimal_to_f64, f64_to_decimal};
use crate::dto::{
    AmortizationParams, AmortizationResult, AmortizationRowDto, ExtraPaymentImpactResult,
};
use crate::loan::build_loan;

#[wasm_bindgen]
pub fn calculate_amortization_schedule(params: JsValue) -> JsValue {
    let result = calculate_schedule_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_schedule_impl(params: JsValue) -> AmortizationResult {
    let params: AmortizationParams = match serde_wasm_bindgen::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return AmortizationResult {
                error: Some(format!("Failed to parse amortization parameters: {e:?}")),
                ..Default::default()
            }
        }
    };

    let loan = match build_loan(&params.loan) {
        Ok(loan) => loan,
        Err(e) => {
            return AmortizationResult {
                error: Some(e),
                ..Default::default()
            }
        }
    };

    let extra_payment = f64_to_decimal(params.extra_payment).max(Decimal::ZERO);

    match mortgage_calc::amortization::schedule(&loan, extra_payment) {
        Ok(rows) => AmortizationResult {
            rows: rows.into_iter().map(to_row_dto).collect(),
            error: None,
        },
        Err(e) => AmortizationResult {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

fn to_row_dto(row: mortgage_calc::amortization::AmortizationRow) -> AmortizationRowDto {
    AmortizationRowDto {
        period: row.period,
        payment: decimal_to_f64(row.payment),
        extra_payment: decimal_to_f64(row.extra_payment),
        principal_portion: decimal_to_f64(row.principal_portion),
        interest_portion: decimal_to_f64(row.interest_portion),
        remaining_balance: decimal_to_f64(row.remaining_balance),
    }
}

#[wasm_bindgen]
pub fn calculate_extra_payment_impact(params: JsValue) -> JsValue {
    let result = calculate_extra_payment_impact_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_extra_payment_impact_impl(params: JsValue) -> ExtraPaymentImpactResult {
    let params: AmortizationParams = match serde_wasm_bindgen::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return ExtraPaymentImpactResult {
                error: Some(format!("Failed to parse amortization parameters: {e:?}")),
                ..Default::default()
            }
        }
    };

    let loan = match build_loan(&params.loan) {
        Ok(loan) => loan,
        Err(e) => {
            return ExtraPaymentImpactResult {
                error: Some(e),
                ..Default::default()
            }
        }
    };

    let extra_payment = f64_to_decimal(params.extra_payment).max(Decimal::ZERO);

    match mortgage_calc::amortization::extra_payment_impact(&loan, extra_payment) {
        Ok(impact) => ExtraPaymentImpactResult {
            baseline_periods: Some(impact.baseline_periods),
            payoff_periods: Some(impact.payoff_periods),
            periods_saved: Some(impact.periods_saved),
            baseline_total_interest: Some(decimal_to_f64(impact.baseline_total_interest)),
            total_interest_with_extra: Some(decimal_to_f64(impact.total_interest_with_extra)),
            interest_saved: Some(decimal_to_f64(impact.interest_saved)),
            error: None,
        },
        Err(e) => ExtraPaymentImpactResult {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}
