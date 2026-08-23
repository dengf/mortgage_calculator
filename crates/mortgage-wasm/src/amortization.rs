//! `calculate_amortization_schedule` and `calculate_extra_payment_impact`.

use rust_decimal::Decimal;
use wasm_bindgen::prelude::*;

use crate::convert::{decimal_to_f64, f64_to_decimal, to_js};
use crate::dto::{
    AmortizationParams, AmortizationResult, AmortizationRowDto, AmortizationYearDto,
    ExtraPaymentImpactResult,
};
use crate::loan::build_loan;
use crate::message::Message;

#[wasm_bindgen]
pub fn calculate_amortization_schedule(params: JsValue) -> JsValue {
    let result = calculate_schedule_impl(params);
    to_js(&result)
}

fn calculate_schedule_impl(params: JsValue) -> AmortizationResult {
    match serde_wasm_bindgen::from_value(params) {
        Ok(params) => schedule_from_params(params),
        Err(_) => {
            let message = Message::bad_request();
            AmortizationResult {
                error: Some(message.text.clone()),
                error_message: Some(message),
                ..Default::default()
            }
        }
    }
}

/// JsValue-free core of [`calculate_schedule_impl`] — see the matching
/// comment on `payment::payment_from_params`.
fn schedule_from_params(params: AmortizationParams) -> AmortizationResult {
    let loan = match build_loan(&params.loan) {
        Ok(loan) => loan,
        Err(e) => {
            return AmortizationResult {
                error: Some(e.text.clone()),
                error_message: Some(e),
                ..Default::default()
            }
        }
    };

    let extra_payment = f64_to_decimal(params.extra_payment).max(Decimal::ZERO);

    match mortgage_calc::amortization::schedule(&loan, extra_payment) {
        Ok(rows) => {
            let yearly = mortgage_calc::amortization::summarize_by_year(
                &rows,
                loan.frequency().periods_per_year(),
            );
            AmortizationResult {
                rows: rows.into_iter().map(to_row_dto).collect(),
                yearly: yearly.into_iter().map(to_year_dto).collect(),
                error: None,
                error_message: None,
            }
        }
        Err(e) => AmortizationResult {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

pub fn to_year_dto(year: mortgage_calc::amortization::AmortizationYear) -> AmortizationYearDto {
    AmortizationYearDto {
        year: year.year,
        paid: decimal_to_f64(year.paid),
        principal: decimal_to_f64(year.principal),
        interest: decimal_to_f64(year.interest),
        remaining_balance: decimal_to_f64(year.remaining_balance),
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
    to_js(&result)
}

fn calculate_extra_payment_impact_impl(params: JsValue) -> ExtraPaymentImpactResult {
    match serde_wasm_bindgen::from_value(params) {
        Ok(params) => extra_payment_impact_from_params(params),
        Err(_) => {
            let message = Message::bad_request();
            ExtraPaymentImpactResult {
                error: Some(message.text.clone()),
                error_message: Some(message),
                ..Default::default()
            }
        }
    }
}

/// JsValue-free core of [`calculate_extra_payment_impact_impl`] — see the
/// matching comment on `payment::payment_from_params`.
fn extra_payment_impact_from_params(params: AmortizationParams) -> ExtraPaymentImpactResult {
    let loan = match build_loan(&params.loan) {
        Ok(loan) => loan,
        Err(e) => {
            return ExtraPaymentImpactResult {
                error: Some(e.text.clone()),
                error_message: Some(e),
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
            error_message: None,
        },
        Err(e) => ExtraPaymentImpactResult {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{LoanParams, RateTypeDto};

    fn valid_loan() -> LoanParams {
        LoanParams {
            principal: 400_000.0,
            rate: RateTypeDto::Fixed { rate_percent: 6.5 },
            term_years: 30.0,
            frequency: None,
        }
    }

    #[test]
    fn schedule_has_one_row_per_period_and_pays_off_to_zero() {
        let result = schedule_from_params(AmortizationParams {
            loan: valid_loan(),
            extra_payment: 0.0,
        });
        assert!(result.error.is_none());
        assert_eq!(result.rows.len(), 360);
        assert_eq!(result.rows.last().unwrap().remaining_balance, 0.0);
    }

    #[test]
    fn schedule_rejects_a_loan_term_long_enough_to_have_forced_a_huge_allocation() {
        // Regression check for the round-3 security fix: this used to
        // reach amortization::schedule()'s Vec::with_capacity with an
        // effectively unbounded period count.
        let result = schedule_from_params(AmortizationParams {
            loan: LoanParams {
                term_years: 1_000_000.0,
                ..valid_loan()
            },
            extra_payment: 0.0,
        });
        assert!(result.rows.is_empty());
        assert!(result.error.unwrap().contains("unreasonably long"));
    }

    #[test]
    fn extra_payment_impact_reports_periods_and_interest_saved() {
        let result = extra_payment_impact_from_params(AmortizationParams {
            loan: valid_loan(),
            extra_payment: 200.0,
        });
        assert!(result.error.is_none());
        assert!(result.periods_saved.unwrap() > 0);
        assert!(result.interest_saved.unwrap() > 0.0);
    }

    #[test]
    fn negative_extra_payment_is_clamped_to_zero_not_treated_as_a_payoff_reduction() {
        let with_negative = extra_payment_impact_from_params(AmortizationParams {
            loan: valid_loan(),
            extra_payment: -500.0,
        });
        let with_zero = extra_payment_impact_from_params(AmortizationParams {
            loan: valid_loan(),
            extra_payment: 0.0,
        });
        assert_eq!(with_negative.periods_saved, with_zero.periods_saved);
    }
}
