use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;
use mortgage_calc::comparison::{ComparisonEntry, ComparisonInput};
use mortgage_calc::RateType;
use mortgage_ports::CalculatorKind;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::components::SavedScenarios;
use crate::screens::parse_frequency;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct CompareRow {
    id: String,
    label: String,
    kind: String,
    rate_percent: String,
    base_rate_percent: String,
    spread_percent: String,
    term_years: String,
}

fn new_row_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("row-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn preset_to_row(preset: &mortgage_calc::RatePreset) -> CompareRow {
    let (kind, rate_percent, base_rate_percent, spread_percent) = match preset.rate_type {
        RateType::Fixed { rate } => (
            "fixed",
            (rate * Decimal::from(100)).to_string(),
            "4.3".to_string(),
            "2".to_string(),
        ),
        RateType::Floating { base_rate, spread } => (
            "floating",
            "6.5".to_string(),
            (base_rate * Decimal::from(100)).to_string(),
            (spread * Decimal::from(100)).to_string(),
        ),
    };
    CompareRow {
        id: new_row_id(),
        label: preset.label.to_string(),
        kind: kind.to_string(),
        rate_percent,
        base_rate_percent,
        spread_percent,
        term_years: preset.term_years.to_string(),
    }
}

fn blank_row() -> CompareRow {
    CompareRow {
        id: new_row_id(),
        label: "Custom scenario".to_string(),
        kind: "fixed".to_string(),
        rate_percent: "6.5".to_string(),
        base_rate_percent: "4.3".to_string(),
        spread_percent: "2".to_string(),
        term_years: "30".to_string(),
    }
}

fn to_comparison_entry(row: &CompareRow) -> Option<ComparisonEntry> {
    let rate_type = if row.kind == "floating" {
        RateType::Floating {
            base_rate: Decimal::from_str(&row.base_rate_percent).ok()? / Decimal::from(100),
            spread: Decimal::from_str(&row.spread_percent).ok()? / Decimal::from(100),
        }
    } else {
        RateType::Fixed {
            rate: Decimal::from_str(&row.rate_percent).ok()? / Decimal::from(100),
        }
    };
    Some(ComparisonEntry {
        label: row.label.clone(),
        rate_type,
        term_years: Decimal::from_str(&row.term_years).ok()?,
    })
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct Inputs {
    principal: String,
    frequency: String,
    rows: Vec<CompareRow>,
}

#[component]
pub fn CompareScreen() -> Element {
    let mut principal = use_signal(|| "400000".to_string());
    let mut frequency = use_signal(|| "monthly".to_string());
    let presets = use_hook(mortgage_calc::common_presets);
    let mut rows = use_signal(|| {
        vec![preset_to_row(&presets[0]), preset_to_row(&presets[2])]
    });

    let results = use_memo(move || {
        let principal_d = Decimal::from_str(&principal.read()).ok()?;
        let freq = parse_frequency(&frequency.read());
        let entries: Vec<ComparisonEntry> = rows.read().iter().filter_map(to_comparison_entry).collect();
        if entries.is_empty() {
            return None;
        }
        let input = ComparisonInput { principal: principal_d, frequency: freq };
        mortgage_calc::comparison::compare(&input, &entries).ok()
    });

    let current_inputs = use_memo(move || {
        serde_json::to_string(&Inputs {
            principal: principal(),
            frequency: frequency(),
            rows: rows(),
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

            div { class: "comparison-presets",
                span { class: "field-label", "Quick add: " }
                for preset in presets.iter() {
                    button {
                        class: "link-button",
                        onclick: {
                            let preset = preset.clone();
                            move |_| rows.write().push(preset_to_row(&preset))
                        },
                        "+ {preset.label}"
                    }
                }
                button { class: "link-button", onclick: move |_| rows.write().push(blank_row()), "+ Custom" }
            }

            div { class: "comparison-entries",
                for row in rows.read().iter().cloned() {
                    CompareRowEditor {
                        key: "{row.id}",
                        row: row.clone(),
                        on_change: move |updated: CompareRow| {
                            if let Some(existing) = rows.write().iter_mut().find(|r| r.id == updated.id) {
                                *existing = updated;
                            }
                        },
                        on_remove: {
                            let id = row.id.clone();
                            move |_| rows.write().retain(|r| r.id != id)
                        },
                    }
                }
                if rows.read().is_empty() {
                    p { class: "saved-scenarios-empty", "Add a scenario to compare." }
                }
            }

            if let Some(results) = results.read().as_ref() {
                div { class: "schedule-table-wrap",
                    table { class: "schedule-table",
                        thead {
                            tr {
                                th { "Scenario" }
                                th { "Rate" }
                                th { "Term" }
                                th { "Payment" }
                                th { "Total paid" }
                                th { "Total interest" }
                            }
                        }
                        tbody {
                            for row in results.iter() {
                                tr { key: "{row.label}",
                                    td { "{row.label}" }
                                    td { "{row.effective_rate * Decimal::from(100)}%" }
                                    td { "{row.term_years}yr" }
                                    td { "${row.payment}" }
                                    td { "${row.total_paid}" }
                                    td { "${row.total_interest}" }
                                }
                            }
                        }
                    }
                }
            }

            SavedScenarios {
                calculator: CalculatorKind::Comparison,
                current_inputs: current_inputs(),
                on_load: move |json: String| {
                    if let Ok(inputs) = serde_json::from_str::<Inputs>(&json) {
                        principal.set(inputs.principal);
                        frequency.set(inputs.frequency);
                        rows.set(inputs.rows);
                    }
                },
            }
        }
    }
}

#[component]
fn CompareRowEditor(row: CompareRow, on_change: EventHandler<CompareRow>, on_remove: EventHandler<()>) -> Element {
    rsx! {
        div { class: "comparison-entry",
            input {
                class: "comparison-entry-label",
                value: "{row.label}",
                oninput: {
                    let row = row.clone();
                    move |e| on_change.call(CompareRow { label: e.value(), ..row.clone() })
                },
            }
            div { class: "comparison-entry-kind",
                button {
                    class: if row.kind == "fixed" { "kind-toggle active" } else { "kind-toggle" },
                    onclick: {
                        let row = row.clone();
                        move |_| on_change.call(CompareRow { kind: "fixed".to_string(), ..row.clone() })
                    },
                    "Fixed",
                }
                button {
                    class: if row.kind == "floating" { "kind-toggle active" } else { "kind-toggle" },
                    onclick: {
                        let row = row.clone();
                        move |_| on_change.call(CompareRow { kind: "floating".to_string(), ..row.clone() })
                    },
                    "Floating",
                }
            }
            if row.kind == "fixed" {
                label { class: "comparison-entry-field",
                    span { "Rate" }
                    input {
                        value: "{row.rate_percent}",
                        oninput: {
                            let row = row.clone();
                            move |e| on_change.call(CompareRow { rate_percent: e.value(), ..row.clone() })
                        },
                    }
                    span { class: "unit", "%" }
                }
            } else {
                label { class: "comparison-entry-field",
                    span { "Base" }
                    input {
                        value: "{row.base_rate_percent}",
                        oninput: {
                            let row = row.clone();
                            move |e| on_change.call(CompareRow { base_rate_percent: e.value(), ..row.clone() })
                        },
                    }
                    span { class: "unit", "%" }
                }
                label { class: "comparison-entry-field",
                    span { "Spread" }
                    input {
                        value: "{row.spread_percent}",
                        oninput: {
                            let row = row.clone();
                            move |e| on_change.call(CompareRow { spread_percent: e.value(), ..row.clone() })
                        },
                    }
                    span { class: "unit", "%" }
                }
            }
            label { class: "comparison-entry-field",
                span { "Term" }
                input {
                    value: "{row.term_years}",
                    oninput: {
                        let row = row.clone();
                        move |e| on_change.call(CompareRow { term_years: e.value(), ..row.clone() })
                    },
                }
                span { class: "unit", "yrs" }
            }
            button { class: "link-button danger", onclick: move |_| on_remove.call(()), "Remove" }
        }
    }
}
