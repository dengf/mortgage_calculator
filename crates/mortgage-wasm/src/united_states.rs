//! `calculate_united_states`: the US panel — conforming/jumbo classification,
//! ZIP-derived property tax, the PMI trigger below 20% down, the resulting
//! PITI, and an optional mortgage-interest deduction estimate.
//!
//! This mirrors what the Slint app shows on its Payment tab, so both UIs
//! present the same figures from the same `mortgage_calc::united_states`
//! logic.

use wasm_bindgen::prelude::*;

use mortgage_calc::united_states;
use mortgage_core::round_currency;
use rust_decimal::Decimal;

use crate::convert::{decimal_to_f64, f64_to_decimal};
use crate::dto::{UnitedStatesParams, UnitedStatesResult};
use crate::message::Message;

#[wasm_bindgen]
pub fn calculate_united_states(params: JsValue) -> JsValue {
    let result = calculate_united_states_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_united_states_impl(params: JsValue) -> UnitedStatesResult {
    match serde_wasm_bindgen::from_value(params) {
        Ok(p) => united_states_from_params(p),
        Err(_) => {
            let message = Message::bad_request();
            UnitedStatesResult {
                error: Some(message.text.clone()),
                error_message: Some(message),
                ..Default::default()
            }
        }
    }
}

fn finite_positive(value: Option<f64>) -> Option<f64> {
    value.filter(|v| v.is_finite() && *v > 0.0)
}

/// The JsValue-free core, so it can be unit-tested with plain
/// `UnitedStatesParams` values without needing a wasm32 target.
fn united_states_from_params(p: UnitedStatesParams) -> UnitedStatesResult {
    let principal = f64_to_decimal(p.principal).max(Decimal::ZERO);
    let home_price = f64_to_decimal(p.home_price).max(Decimal::ZERO);

    let mut result = UnitedStatesResult {
        loan_type: match united_states::classify_loan(principal) {
            united_states::LoanConformance::Conforming => "Conforming".to_string(),
            united_states::LoanConformance::Jumbo => "Jumbo".to_string(),
        },
        ..Default::default()
    };

    let tax_rate = united_states::estimate_property_tax_rate(&p.zip);
    result.property_tax_rate_percent = tax_rate.map(|rate| decimal_to_f64(rate) * 100.0);
    let monthly_property_tax = tax_rate
        .map(|rate| round_currency(home_price * rate / Decimal::from(12)))
        .unwrap_or(Decimal::ZERO);
    result.monthly_property_tax = decimal_to_f64(monthly_property_tax);

    let down_payment = (home_price - principal).max(Decimal::ZERO);
    let down_payment_percent = if home_price > Decimal::ZERO {
        down_payment / home_price
    } else {
        Decimal::ZERO
    };
    result.down_payment = decimal_to_f64(round_currency(down_payment));
    result.down_payment_percent = decimal_to_f64(down_payment_percent) * 100.0;

    // Without a home price there's no down payment to judge, so PMI can't be
    // said to be required -- rather than reading 0% down and demanding it.
    let pmi_required =
        home_price > Decimal::ZERO && united_states::requires_pmi(down_payment_percent);
    result.pmi_required = pmi_required;

    let pmi_rate = p
        .pmi_rate_percent
        .filter(|v| v.is_finite())
        .map(|v| f64_to_decimal(v) / Decimal::from(100))
        .unwrap_or(united_states::DEFAULT_PMI_ANNUAL_RATE);
    let monthly_pmi = if pmi_required {
        united_states::monthly_pmi(principal, pmi_rate)
    } else {
        Decimal::ZERO
    };
    result.monthly_pmi = decimal_to_f64(monthly_pmi);

    if let Some(pi) = finite_positive(p.monthly_pi) {
        let pi = f64_to_decimal(pi);
        let piti = round_currency(pi + monthly_property_tax + monthly_pmi);
        result.monthly_piti = Some(decimal_to_f64(piti));

        if p.use_tax_deduction {
            // The first period's interest is the whole balance times the
            // periodic rate. Deriving it here rather than in JS keeps the
            // formula on the Rust side with the rest of the math.
            let periodic_rate =
                f64_to_decimal(p.annual_rate_percent) / Decimal::from(100) / Decimal::from(12);
            let first_period_interest = round_currency(principal * periodic_rate);
            let savings = united_states::monthly_tax_savings(
                first_period_interest,
                f64_to_decimal(p.marginal_tax_rate_percent) / Decimal::from(100),
            );
            result.monthly_tax_savings = Some(decimal_to_f64(savings));
            result.net_monthly_cost = Some(decimal_to_f64(round_currency(piti - savings)));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> UnitedStatesParams {
        UnitedStatesParams {
            monthly_pi: Some(2_528.27),
            principal: 400_000.0,
            home_price: 500_000.0,
            annual_rate_percent: 6.5,
            zip: "90210".into(),
            pmi_rate_percent: None,
            use_tax_deduction: false,
            marginal_tax_rate_percent: 24.0,
        }
    }

    #[test]
    fn classifies_a_loan_under_the_baseline_limit_as_conforming() {
        assert_eq!(united_states_from_params(params()).loan_type, "Conforming");
    }

    #[test]
    fn classifies_a_loan_above_the_baseline_limit_as_jumbo() {
        let r = united_states_from_params(UnitedStatesParams {
            principal: 900_000.0,
            home_price: 1_200_000.0,
            ..params()
        });
        assert_eq!(r.loan_type, "Jumbo");
    }

    #[test]
    fn derives_property_tax_from_the_zip() {
        let r = united_states_from_params(params());
        // 90210 -> CA. The rate itself lives in mortgage-calc; assert only
        // that one was found and applied to the home price.
        assert!(r.property_tax_rate_percent.is_some());
        assert!(r.monthly_property_tax > 0.0);
    }

    #[test]
    fn reports_an_unrecognized_zip_rather_than_silently_taxing_at_zero() {
        let r = united_states_from_params(UnitedStatesParams {
            zip: "00000".into(),
            ..params()
        });
        assert!(r.property_tax_rate_percent.is_none());
        assert_eq!(r.monthly_property_tax, 0.0);
    }

    #[test]
    fn no_pmi_at_or_above_twenty_percent_down() {
        // 500k price, 400k loan = exactly 20% down.
        let r = united_states_from_params(params());
        assert_eq!(r.down_payment_percent, 20.0);
        assert!(!r.pmi_required);
        assert_eq!(r.monthly_pmi, 0.0);
    }

    #[test]
    fn pmi_kicks_in_below_twenty_percent_down() {
        let r = united_states_from_params(UnitedStatesParams {
            home_price: 440_000.0,
            ..params()
        });
        assert!(r.pmi_required);
        assert!(r.monthly_pmi > 0.0);
    }

    #[test]
    fn piti_sums_payment_tax_and_pmi() {
        let r = united_states_from_params(params());
        let piti = r.monthly_piti.unwrap();
        assert!((piti - (2_528.27 + r.monthly_property_tax + r.monthly_pmi)).abs() < 0.01);
    }

    #[test]
    fn tax_deduction_is_omitted_unless_asked_for() {
        let r = united_states_from_params(params());
        assert!(r.monthly_tax_savings.is_none());
        assert!(r.net_monthly_cost.is_none());
    }

    #[test]
    fn tax_deduction_reduces_the_net_monthly_cost() {
        let r = united_states_from_params(UnitedStatesParams {
            use_tax_deduction: true,
            ..params()
        });
        let savings = r.monthly_tax_savings.unwrap();
        // 400,000 * 6.5%/12 = 2,166.67 interest, deducted at 24%.
        assert!((savings - 520.0).abs() < 1.0, "got {savings}");
        assert!(r.net_monthly_cost.unwrap() < r.monthly_piti.unwrap());
    }

    #[test]
    fn property_tax_and_pmi_still_compute_without_a_payment() {
        let r = united_states_from_params(UnitedStatesParams {
            monthly_pi: None,
            home_price: 440_000.0,
            ..params()
        });
        assert!(r.monthly_piti.is_none());
        assert!(r.monthly_property_tax > 0.0);
        assert!(r.pmi_required);
    }

    #[test]
    fn a_zero_home_price_does_not_demand_pmi_on_a_phantom_zero_percent_down() {
        let r = united_states_from_params(UnitedStatesParams {
            home_price: 0.0,
            ..params()
        });
        assert!(!r.pmi_required);
        assert_eq!(r.monthly_pmi, 0.0);
    }
}
