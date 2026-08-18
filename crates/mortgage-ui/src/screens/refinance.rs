use std::str::FromStr;

use dioxus::prelude::*;
use mortgage_calc::refinance::RefinanceInput;
use mortgage_calc::PaymentFrequency;
use mortgage_ports::CalculatorKind;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::components::SavedScenarios;

#[derive(Serialize, Deserialize, Clone, Default)]
struct Inputs {
    current_balance: String,
    current_rate_percent: String,
    remaining_periods: String,
    new_rate_percent: String,
    new_term_years: String,
    closing_costs: String,
}

#[component]
pub fn RefinanceScreen() -> Element {
    let mut current_balance = use_signal(|| "300000".to_string());
    let mut current_rate_percent = use_signal(|| "7.5".to_string());
    let mut remaining_periods = use_signal(|| "300".to_string());
    let mut new_rate_percent = use_signal(|| "6.0".to_string());
    let mut new_term_years = use_signal(|| "30".to_string());
    let mut closing_costs = use_signal(|| "6000".to_string());

    let result = use_memo(move || {
        let input = RefinanceInput {
            current_balance: Decimal::from_str(&current_balance.read()).ok()?,
            current_annual_rate: Decimal::from_str(&current_rate_percent.read()).ok()? / Decimal::from(100),
            remaining_periods: remaining_periods.read().parse::<u32>().ok()?,
            new_annual_rate: Decimal::from_str(&new_rate_percent.read()).ok()? / Decimal::from(100),
            new_term_years: Decimal::from_str(&new_term_years.read()).ok()?,
            closing_costs: Decimal::from_str(&closing_costs.read()).ok()?,
            frequency: PaymentFrequency::Monthly,
        };
        mortgage_calc::refinance::analyze_refinance(&input).ok()
    });

    let current_inputs = use_memo(move || {
        serde_json::to_string(&Inputs {
            current_balance: current_balance(),
            current_rate_percent: current_rate_percent(),
            remaining_periods: remaining_periods(),
            new_rate_percent: new_rate_percent(),
            new_term_years: new_term_years(),
            closing_costs: closing_costs(),
        })
        .unwrap_or_default()
    });

    rsx! {
        section { class: "panel",
            div { class: "panel-form",
                div { class: "field",
                    label { "Current balance" }
                    input { value: "{current_balance}", oninput: move |e| current_balance.set(e.value()) }
                }
                div { class: "field",
                    label { "Current rate (%)" }
                    input { value: "{current_rate_percent}", oninput: move |e| current_rate_percent.set(e.value()) }
                }
                div { class: "field",
                    label { "Payments remaining" }
                    input { value: "{remaining_periods}", oninput: move |e| remaining_periods.set(e.value()) }
                }
                div { class: "field",
                    label { "New rate (%)" }
                    input { value: "{new_rate_percent}", oninput: move |e| new_rate_percent.set(e.value()) }
                }
                div { class: "field",
                    label { "New term (years)" }
                    input { value: "{new_term_years}", oninput: move |e| new_term_years.set(e.value()) }
                }
                div { class: "field",
                    label { "Closing costs" }
                    input { value: "{closing_costs}", oninput: move |e| closing_costs.set(e.value()) }
                }
            }

            match result.read().as_ref() {
                Some(result) => rsx! {
                    div { class: "stat-grid",
                        div { class: "stat",
                            span { class: "stat-label", "Current payment" }
                            span { class: "stat-value", "${result.current_payment}" }
                        }
                        div { class: "stat",
                            span { class: "stat-label", "New payment" }
                            span { class: "stat-value", "${result.new_payment}" }
                        }
                        div { class: "stat stat-primary",
                            span { class: "stat-label", "Monthly savings" }
                            span { class: "stat-value", "${result.payment_savings}" }
                        }
                        div { class: "stat",
                            span { class: "stat-label", "Break-even" }
                            span {
                                class: "stat-value",
                                if let Some(periods) = result.break_even_periods {
                                    "{periods} months"
                                } else {
                                    "Never"
                                }
                            }
                        }
                        div { class: "stat",
                            span { class: "stat-label", "Lifetime savings (net of costs)" }
                            span { class: "stat-value", "${result.lifetime_savings}" }
                        }
                    }
                },
                None => rsx! {
                    p { class: "error", "Enter valid refinance details." }
                },
            }

            SavedScenarios {
                calculator: CalculatorKind::Refinance,
                current_inputs: current_inputs(),
                on_load: move |json: String| {
                    if let Ok(inputs) = serde_json::from_str::<Inputs>(&json) {
                        current_balance.set(inputs.current_balance);
                        current_rate_percent.set(inputs.current_rate_percent);
                        remaining_periods.set(inputs.remaining_periods);
                        new_rate_percent.set(inputs.new_rate_percent);
                        new_term_years.set(inputs.new_term_years);
                        closing_costs.set(inputs.closing_costs);
                    }
                },
            }
        }
    }
}
