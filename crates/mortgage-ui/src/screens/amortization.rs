use std::str::FromStr;

use dioxus::prelude::*;
use mortgage_calc::amortization::AmortizationRow;
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
    extra_payment: String,
}

#[derive(Clone, Copy, PartialEq)]
struct YearRow {
    year: u32,
    paid: Decimal,
    principal: Decimal,
    interest: Decimal,
    remaining_balance: Decimal,
}

fn periods_per_year(freq: &str) -> usize {
    match freq {
        "biweekly" => 26,
        "weekly" => 52,
        _ => 12,
    }
}

fn summarize_by_year(rows: &[AmortizationRow], per_year: usize) -> Vec<YearRow> {
    rows.chunks(per_year.max(1))
        .enumerate()
        .map(|(i, chunk)| YearRow {
            year: i as u32 + 1,
            paid: chunk.iter().map(|r| r.payment).sum(),
            principal: chunk.iter().map(|r| r.principal_portion).sum(),
            interest: chunk.iter().map(|r| r.interest_portion).sum(),
            remaining_balance: chunk.last().expect("chunk is never empty").remaining_balance,
        })
        .collect()
}

#[component]
pub fn AmortizationScreen() -> Element {
    let mut principal = use_signal(|| "400000".to_string());
    let mut rate_percent = use_signal(|| "6.5".to_string());
    let mut term_years = use_signal(|| "30".to_string());
    let mut frequency = use_signal(|| "monthly".to_string());
    let mut extra_payment = use_signal(|| "0".to_string());

    let loan = use_memo(move || {
        let principal_d = Decimal::from_str(&principal.read()).ok()?;
        let rate_d = Decimal::from_str(&rate_percent.read()).ok()?;
        let term_d = Decimal::from_str(&term_years.read()).ok()?;
        let freq = parse_frequency(&frequency.read());
        Loan::builder()
            .principal(principal_d)
            .annual_rate(rate_d / Decimal::from(100))
            .term_years(term_d)
            .frequency(freq)
            .build()
            .ok()
    });

    let extra_decimal =
        use_memo(move || Decimal::from_str(&extra_payment.read()).unwrap_or(Decimal::ZERO).max(Decimal::ZERO));

    let schedule = use_memo(move || {
        let loan = (*loan.read())?;
        mortgage_calc::amortization::schedule(&loan, extra_decimal()).ok()
    });

    let impact = use_memo(move || {
        let loan = (*loan.read())?;
        if extra_decimal() <= Decimal::ZERO {
            return None;
        }
        mortgage_calc::amortization::extra_payment_impact(&loan, extra_decimal()).ok()
    });

    let yearly = use_memo(move || {
        schedule
            .read()
            .as_ref()
            .map(|rows| summarize_by_year(rows, periods_per_year(&frequency.read())))
    });

    let current_inputs = use_memo(move || {
        serde_json::to_string(&Inputs {
            principal: principal(),
            rate_percent: rate_percent(),
            term_years: term_years(),
            frequency: frequency(),
            extra_payment: extra_payment(),
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
                div { class: "field",
                    label { "Extra payment per period" }
                    input { value: "{extra_payment}", oninput: move |e| extra_payment.set(e.value()) }
                }
            }

            if let Some(impact) = impact.read().as_ref() {
                div { class: "stat-grid",
                    div { class: "stat stat-primary",
                        span { class: "stat-label", "Payoff time saved" }
                        span { class: "stat-value", "{impact.periods_saved} payments" }
                    }
                    div { class: "stat",
                        span { class: "stat-label", "Interest saved" }
                        span { class: "stat-value", "${impact.interest_saved}" }
                    }
                    div { class: "stat",
                        span { class: "stat-label", "New payoff" }
                        span { class: "stat-value", "{impact.payoff_periods} payments" }
                    }
                }
            }

            match yearly.read().as_ref() {
                Some(years) => rsx! {
                    div { class: "schedule-table-wrap",
                        table { class: "schedule-table",
                            thead {
                                tr {
                                    th { "Year" }
                                    th { "Paid" }
                                    th { "Principal" }
                                    th { "Interest" }
                                    th { "Balance" }
                                }
                            }
                            tbody {
                                for row in years.iter() {
                                    tr { key: "{row.year}",
                                        td { "{row.year}" }
                                        td { "${row.paid}" }
                                        td { "${row.principal}" }
                                        td { "${row.interest}" }
                                        td { "${row.remaining_balance}" }
                                    }
                                }
                            }
                        }
                    }
                },
                None => rsx! {
                    p { class: "error", "Enter valid loan details." }
                },
            }

            SavedScenarios {
                calculator: CalculatorKind::Amortization,
                current_inputs: current_inputs(),
                on_load: move |json: String| {
                    if let Ok(inputs) = serde_json::from_str::<Inputs>(&json) {
                        principal.set(inputs.principal);
                        rate_percent.set(inputs.rate_percent);
                        term_years.set(inputs.term_years);
                        frequency.set(inputs.frequency);
                        extra_payment.set(inputs.extra_payment);
                    }
                },
            }
        }
    }
}
