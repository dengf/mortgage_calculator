use std::str::FromStr;

use dioxus::prelude::*;
use mortgage_calc::affordability::AffordabilityInput;
use mortgage_ports::CalculatorKind;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::components::SavedScenarios;

#[derive(Serialize, Deserialize, Clone, Default)]
struct Inputs {
    income: String,
    debts: String,
    down_payment: String,
    rate_percent: String,
    term_years: String,
    max_dti_percent: String,
    tax_rate_percent: String,
    insurance: String,
    hoa: String,
}

#[component]
pub fn AffordabilityScreen() -> Element {
    let mut income = use_signal(|| "10000".to_string());
    let mut debts = use_signal(|| "500".to_string());
    let mut down_payment = use_signal(|| "60000".to_string());
    let mut rate_percent = use_signal(|| "6.5".to_string());
    let mut term_years = use_signal(|| "30".to_string());
    let mut max_dti_percent = use_signal(|| "36".to_string());
    let mut tax_rate_percent = use_signal(|| "1.2".to_string());
    let mut insurance = use_signal(|| "1500".to_string());
    let mut hoa = use_signal(|| "0".to_string());

    let result = use_memo(move || {
        let input = AffordabilityInput {
            gross_monthly_income: Decimal::from_str(&income.read()).ok()?,
            monthly_debts: Decimal::from_str(&debts.read()).ok()?,
            down_payment: Decimal::from_str(&down_payment.read()).ok()?,
            annual_rate: Decimal::from_str(&rate_percent.read()).ok()? / Decimal::from(100),
            term_years: Decimal::from_str(&term_years.read()).ok()?,
            max_dti: Decimal::from_str(&max_dti_percent.read()).ok()? / Decimal::from(100),
            annual_property_tax_rate: Decimal::from_str(&tax_rate_percent.read()).ok()? / Decimal::from(100),
            annual_insurance: Decimal::from_str(&insurance.read()).ok()?,
            monthly_hoa: Decimal::from_str(&hoa.read()).ok()?,
        };
        mortgage_calc::affordability::max_affordable(&input).ok()
    });

    let current_inputs = use_memo(move || {
        serde_json::to_string(&Inputs {
            income: income(),
            debts: debts(),
            down_payment: down_payment(),
            rate_percent: rate_percent(),
            term_years: term_years(),
            max_dti_percent: max_dti_percent(),
            tax_rate_percent: tax_rate_percent(),
            insurance: insurance(),
            hoa: hoa(),
        })
        .unwrap_or_default()
    });

    rsx! {
        section { class: "panel",
            div { class: "panel-form",
                div { class: "field",
                    label { "Gross monthly income" }
                    input { value: "{income}", oninput: move |e| income.set(e.value()) }
                }
                div { class: "field",
                    label { "Other monthly debts" }
                    input { value: "{debts}", oninput: move |e| debts.set(e.value()) }
                }
                div { class: "field",
                    label { "Down payment" }
                    input { value: "{down_payment}", oninput: move |e| down_payment.set(e.value()) }
                }
                div { class: "field",
                    label { "Interest rate (%)" }
                    input { value: "{rate_percent}", oninput: move |e| rate_percent.set(e.value()) }
                }
                div { class: "field",
                    label { "Loan term (years)" }
                    input { value: "{term_years}", oninput: move |e| term_years.set(e.value()) }
                }
                div { class: "field",
                    label { "Max debt-to-income (%)" }
                    input { value: "{max_dti_percent}", oninput: move |e| max_dti_percent.set(e.value()) }
                }
                div { class: "field",
                    label { "Property tax rate (%/yr)" }
                    input { value: "{tax_rate_percent}", oninput: move |e| tax_rate_percent.set(e.value()) }
                }
                div { class: "field",
                    label { "Annual insurance" }
                    input { value: "{insurance}", oninput: move |e| insurance.set(e.value()) }
                }
                div { class: "field",
                    label { "Monthly HOA" }
                    input { value: "{hoa}", oninput: move |e| hoa.set(e.value()) }
                }
            }

            match result.read().as_ref() {
                Some(result) => rsx! {
                    div { class: "stat-grid",
                        div { class: "stat stat-primary",
                            span { class: "stat-label", "Max home price" }
                            span { class: "stat-value", "${result.max_home_price}" }
                        }
                        div { class: "stat",
                            span { class: "stat-label", "Max loan amount" }
                            span { class: "stat-value", "${result.max_loan_amount}" }
                        }
                        div { class: "stat",
                            span { class: "stat-label", "Principal & interest" }
                            span { class: "stat-value", "${result.max_principal_and_interest}" }
                        }
                        div { class: "stat",
                            span { class: "stat-label", "Front-end DTI" }
                            span { class: "stat-value", "{result.front_end_dti * Decimal::from(100)}%" }
                        }
                        div { class: "stat",
                            span { class: "stat-label", "Back-end DTI" }
                            span { class: "stat-value", "{result.back_end_dti * Decimal::from(100)}%" }
                        }
                    }
                },
                None => rsx! {
                    p { class: "error", "Enter valid affordability details." }
                },
            }

            SavedScenarios {
                calculator: CalculatorKind::Affordability,
                current_inputs: current_inputs(),
                on_load: move |json: String| {
                    if let Ok(inputs) = serde_json::from_str::<Inputs>(&json) {
                        income.set(inputs.income);
                        debts.set(inputs.debts);
                        down_payment.set(inputs.down_payment);
                        rate_percent.set(inputs.rate_percent);
                        term_years.set(inputs.term_years);
                        max_dti_percent.set(inputs.max_dti_percent);
                        tax_rate_percent.set(inputs.tax_rate_percent);
                        insurance.set(inputs.insurance);
                        hoa.set(inputs.hoa);
                    }
                },
            }
        }
    }
}
