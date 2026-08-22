//! `summarize_scenario`, `down_payment_for_percent`: the figures that follow
//! from a price and a deposit.
//!
//! Both were arithmetic in the React components until now. They are here so
//! there is one answer to "what is the loan amount" rather than one per
//! front end.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use mortgage_calc::scenario::Scenario;

use crate::convert::{decimal_to_f64, f64_to_decimal, to_js};

#[derive(Debug, Default, Deserialize)]
pub struct ScenarioParams {
    #[serde(default)]
    pub home_price: f64,
    #[serde(default)]
    pub down_payment: f64,
}

#[derive(Debug, Default, Serialize)]
pub struct ScenarioSummaryResult {
    pub principal: f64,
    /// `null` rather than `0` when there is no price to divide by --
    /// "0.0% of price" would state something the inputs do not.
    pub down_payment_percent: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PercentParams {
    #[serde(default)]
    pub home_price: f64,
    #[serde(default)]
    pub percent: f64,
}

#[derive(Debug, Default, Serialize)]
pub struct DownPaymentResult {
    pub down_payment: f64,
}

#[wasm_bindgen]
pub fn summarize_scenario(params: JsValue) -> JsValue {
    let result = summarize_scenario_impl(params);
    to_js(&result)
}

fn summarize_scenario_impl(params: JsValue) -> ScenarioSummaryResult {
    // A half-typed form is the normal state of this input, not an error
    // worth reporting: the fields are read on every keystroke. Unparseable
    // input summarizes an empty scenario, which is what an empty form is.
    let params: ScenarioParams = serde_wasm_bindgen::from_value(params).unwrap_or_default();
    summary_of(&params)
}

/// The JsValue-free core, so this is testable on the native host.
fn summary_of(params: &ScenarioParams) -> ScenarioSummaryResult {
    let summary = Scenario::new(
        f64_to_decimal(params.home_price),
        f64_to_decimal(params.down_payment),
    )
    .summary();
    ScenarioSummaryResult {
        principal: decimal_to_f64(summary.principal),
        down_payment_percent: summary.down_payment_percent.map(decimal_to_f64),
    }
}

#[wasm_bindgen]
pub fn down_payment_for_percent(params: JsValue) -> JsValue {
    let result = down_payment_for_percent_impl(params);
    to_js(&result)
}

fn down_payment_for_percent_impl(params: JsValue) -> DownPaymentResult {
    let params: PercentParams = serde_wasm_bindgen::from_value(params).unwrap_or_default();
    deposit_for(&params)
}

fn deposit_for(params: &PercentParams) -> DownPaymentResult {
    DownPaymentResult {
        down_payment: decimal_to_f64(Scenario::down_payment_for_percent(
            f64_to_decimal(params.home_price),
            f64_to_decimal(params.percent),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_price_and_deposit() {
        let s = summary_of(&ScenarioParams {
            home_price: 500_000.0,
            down_payment: 100_000.0,
        });
        assert_eq!(s.principal, 400_000.0);
        assert_eq!(s.down_payment_percent, Some(20.0));
    }

    #[test]
    fn reports_no_percentage_without_a_price() {
        let s = summary_of(&ScenarioParams::default());
        assert_eq!(s.principal, 0.0);
        assert_eq!(s.down_payment_percent, None);
    }

    #[test]
    fn converts_a_percentage_to_a_deposit() {
        let d = deposit_for(&PercentParams {
            home_price: 500_000.0,
            percent: 20.0,
        });
        assert_eq!(d.down_payment, 100_000.0);
    }
}
