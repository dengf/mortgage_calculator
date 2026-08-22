//! `calculate_sg_affordability`: how much property a Singapore buyer can
//! actually complete on.
//!
//! The US affordability model works backwards from a debt-to-income ratio.
//! None of that applies here — Singapore runs on MAS's TDSR/MSR servicing
//! ceilings, the Notice 632 LTV limits, and IRAS stamp duty payable in cash
//! at completion. This binding reaches the Singapore model instead, so the
//! Affordability tab stops answering an American question with a Singapore
//! currency symbol on it.

use wasm_bindgen::prelude::*;

use mortgage_calc::singapore::{self, BindingConstraint};
use rust_decimal::Decimal;

use crate::convert::{decimal_to_f64, f64_to_decimal};
use crate::dto::{SgAffordabilityParams, SgAffordabilityResultDto};
use crate::message::Message;

#[wasm_bindgen]
pub fn calculate_sg_affordability(params: JsValue) -> JsValue {
    let result = calculate_sg_affordability_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_sg_affordability_impl(params: JsValue) -> SgAffordabilityResultDto {
    match serde_wasm_bindgen::from_value(params) {
        Ok(p) => sg_affordability_from_params(p),
        Err(_) => {
            let message = Message::bad_request();
            SgAffordabilityResultDto {
                error: Some(message.text.clone()),
                error_message: Some(message),
                ..Default::default()
            }
        }
    }
}

fn parse_residency(s: &str) -> singapore::Residency {
    match s {
        "PR" => singapore::Residency::PermanentResident,
        "Foreigner" => singapore::Residency::Foreigner,
        "FTA" => singapore::Residency::FtaNational,
        _ => singapore::Residency::Citizen,
    }
}

fn parse_property_count(s: &str) -> singapore::PropertyCount {
    match s {
        "2nd" => singapore::PropertyCount::Second,
        "3rd+" => singapore::PropertyCount::ThirdOrMore,
        _ => singapore::PropertyCount::First,
    }
}

/// Ratios are fractions in `mortgage_calc`; the UI shows percentages.
///
/// Scaled in `Decimal` and converted once, rather than converting and then
/// multiplying by 100.0 — the latter turns 0.55 into 55.00000000000001.
fn to_percent(ratio: Decimal) -> f64 {
    decimal_to_f64(ratio * Decimal::from(100))
}

/// Stable codes rather than prose, so the UI can translate them.
fn constraint_code(c: BindingConstraint) -> &'static str {
    match c {
        BindingConstraint::Tdsr => "tdsr",
        BindingConstraint::Msr => "msr",
        BindingConstraint::Ltv => "ltv",
    }
}

/// The JsValue-free core, so it can be unit-tested without a wasm32 target.
fn sg_affordability_from_params(p: SgAffordabilityParams) -> SgAffordabilityResultDto {
    let input = singapore::SgAffordabilityInput {
        income: singapore::MonthlyIncome {
            fixed: f64_to_decimal(p.fixed_monthly_income),
            variable: f64_to_decimal(p.variable_monthly_income),
        },
        other_monthly_debts: f64_to_decimal(p.other_monthly_debts),
        cash_available: f64_to_decimal(p.cash_available),
        cpf_oa_available: f64_to_decimal(p.cpf_oa_available),
        annual_rate: f64_to_decimal(p.annual_rate_percent) / Decimal::from(100),
        term_years: f64_to_decimal(p.term_years),
        borrower_age: p
            .borrower_age
            .filter(|v| v.is_finite() && *v > 0.0)
            .map(f64_to_decimal),
        is_hdb_or_ec: p.is_hdb_or_ec,
        residency: parse_residency(&p.residency),
        property_count: parse_property_count(&p.property_count),
        outstanding_housing_loans: p.outstanding_housing_loans,
    };

    match singapore::max_affordable_sg(&input) {
        Ok(r) => SgAffordabilityResultDto {
            max_price: Some(decimal_to_f64(r.max_price)),
            max_loan: Some(decimal_to_f64(r.max_loan)),
            binding_constraint: Some(constraint_code(r.binding_constraint).to_string()),
            max_monthly_instalment: Some(decimal_to_f64(r.max_monthly_instalment)),
            assessment_rate_percent: Some(to_percent(r.assessment_rate)),
            ltv_percent: Some(to_percent(r.ltv)),
            extended_tenure: r.extended_tenure,
            deposit: Some(decimal_to_f64(r.deposit)),
            min_cash_required: Some(decimal_to_f64(r.min_cash_required)),
            bsd: Some(decimal_to_f64(r.bsd)),
            absd: Some(decimal_to_f64(r.absd)),
            cash_required: Some(decimal_to_f64(r.cash_required)),
            cpf_used: Some(decimal_to_f64(r.cpf_used)),
            cash_and_cpf_at_completion: Some(decimal_to_f64(r.cash_and_cpf_at_completion)),
            assessed_monthly_income: Some(decimal_to_f64(r.assessed_monthly_income)),
            error: None,
            error_message: None,
        },
        Err(e) => {
            let message = Message::from(&e);
            SgAffordabilityResultDto {
                error: Some(message.text.clone()),
                error_message: Some(message),
                ..Default::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> SgAffordabilityParams {
        SgAffordabilityParams {
            fixed_monthly_income: 12_000.0,
            variable_monthly_income: 0.0,
            other_monthly_debts: 0.0,
            cash_available: 500_000.0,
            cpf_oa_available: 0.0,
            annual_rate_percent: 4.0,
            term_years: 25.0,
            borrower_age: Some(35.0),
            is_hdb_or_ec: false,
            residency: "Citizen".into(),
            property_count: "1st".into(),
            outstanding_housing_loans: 0,
        }
    }

    #[test]
    fn reports_a_price_the_buyer_can_actually_complete_on() {
        let r = sg_affordability_from_params(params());
        assert!(r.error.is_none());
        assert!(r.max_price.unwrap() > 0.0);
        assert!(r.cash_required.unwrap() <= 500_000.0 + 1.0);
    }

    #[test]
    fn assesses_at_the_mas_floor_not_the_quoted_rate() {
        let r = sg_affordability_from_params(SgAffordabilityParams {
            annual_rate_percent: 1.5,
            ..params()
        });
        assert_eq!(r.assessment_rate_percent, Some(4.0));
    }

    #[test]
    fn names_the_constraint_with_a_stable_code() {
        let r = sg_affordability_from_params(params());
        assert!(matches!(
            r.binding_constraint.as_deref(),
            Some("tdsr" | "msr" | "ltv")
        ));
    }

    #[test]
    fn a_second_property_drops_the_ltv_row_and_adds_absd() {
        let first = sg_affordability_from_params(params());
        let second = sg_affordability_from_params(SgAffordabilityParams {
            property_count: "2nd".into(),
            outstanding_housing_loans: 1,
            ..params()
        });
        assert_eq!(first.ltv_percent, Some(75.0));
        assert_eq!(second.ltv_percent, Some(45.0));
        assert!(second.absd.unwrap() > 0.0);
        assert!(second.max_price.unwrap() < first.max_price.unwrap());
    }

    #[test]
    fn flags_an_extended_tenure() {
        let r = sg_affordability_from_params(SgAffordabilityParams {
            term_years: 35.0,
            ..params()
        });
        assert!(r.extended_tenure);
        assert_eq!(r.ltv_percent, Some(55.0));
    }

    #[test]
    fn a_zero_income_reports_a_translatable_error_not_a_number() {
        let r = sg_affordability_from_params(SgAffordabilityParams {
            fixed_monthly_income: 0.0,
            ..params()
        });
        assert!(r.max_price.is_none());
        assert_eq!(r.error_message.unwrap().code, "err.invalidIncome");
    }
}
