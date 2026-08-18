use std::str::FromStr;

use dioxus::prelude::*;
use mortgage_calc::Loan;
use mortgage_ports::CalculatorKind;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::components::SavedScenarios;
use crate::screens::parse_frequency;

#[derive(Serialize, Deserialize, Clone, Default)]
struct Inputs {
    principal: String,
    rate_percent: String,
    term_years: String,
    frequency: String,
}

#[component]
pub fn PaymentScreen() -> Element {
    let mut principal = use_signal(|| "400000".to_string());
    let mut rate_percent = use_signal(|| "6.5".to_string());
    let mut term_years = use_signal(|| "30".to_string());
    let mut frequency = use_signal(|| "monthly".to_string());

    let summary = use_memo(move || {
        let principal_d = Decimal::from_str(&principal.read()).ok()?;
        let rate_d = Decimal::from_str(&rate_percent.read()).ok()?;
        let term_d = Decimal::from_str(&term_years.read()).ok()?;
        let freq = parse_frequency(&frequency.read());

        let loan = Loan::builder()
            .principal(principal_d)
            .annual_rate(rate_d / Decimal::from(100))
            .term_years(term_d)
            .frequency(freq)
            .build()
            .ok()?;

        Some(mortgage_calc::payment::summarize(&loan))
    });

    let current_inputs = use_memo(move || {
        serde_json::to_string(&Inputs {
            principal: principal(),
            rate_percent: rate_percent(),
            term_years: term_years(),
            frequency: frequency(),
        })
        .unwrap_or_default()
    });

    rsx! {
        section { class: "panel",
            div { class: "panel-form",
                div { class: "field",
                    label { "Home loan amount" }
                    input { value: "{principal}", oninput: move |e| principal.set(e.value()) }
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
                    label { "Payment frequency" }
                    select {
                        value: "{frequency}",
                        onchange: move |e| frequency.set(e.value()),
                        option { value: "monthly", "Monthly" }
                        option { value: "biweekly", "Bi-weekly" }
                        option { value: "weekly", "Weekly" }
                    }
                }
            }

            match summary.read().as_ref() {
                Some(summary) => rsx! {
                    div { class: "stat-grid",
                        div { class: "stat stat-primary",
                            span { class: "stat-label", "Payment" }
                            span { class: "stat-value", "${summary.payment}" }
                        }
                        div { class: "stat",
                            span { class: "stat-label", "Total of {summary.total_periods} payments" }
                            span { class: "stat-value", "${summary.total_paid}" }
                        }
                        div { class: "stat",
                            span { class: "stat-label", "Total interest" }
                            span { class: "stat-value", "${summary.total_interest}" }
                        }
                    }
                },
                None => rsx! {
                    p { class: "error", "Enter valid loan details." }
                },
            }

            SavedScenarios {
                calculator: CalculatorKind::Payment,
                current_inputs: current_inputs(),
                on_load: move |json: String| {
                    if let Ok(inputs) = serde_json::from_str::<Inputs>(&json) {
                        principal.set(inputs.principal);
                        rate_percent.set(inputs.rate_percent);
                        term_years.set(inputs.term_years);
                        frequency.set(inputs.frequency);
                    }
                },
            }
        }
    }
}
