#![allow(non_snake_case)]

mod components;
mod screens;
mod storage;

use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use mortgage_ext_redb::RedbScenarioStore;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Payment,
    Amortization,
    Affordability,
    Refinance,
    Compare,
}

/// The store, once loaded, is provided through context as this signal
/// (see [`crate::components::StoreSignal`]) rather than as a component
/// prop — `RedbScenarioStore` has no `PartialEq`, which `#[component]`
/// props require, whereas `Signal<T>` is cheaply `Copy` regardless of what
/// it wraps.
#[component]
fn App() -> Element {
    let mut store = use_signal(|| None::<Rc<RedbScenarioStore>>);
    use_context_provider(|| store);

    use_effect(move || {
        spawn(async move {
            store.set(Some(storage::open_store().await));
        });
    });

    rsx! {
        style { {include_str!("../assets/main.css")} }
        if store.read().is_some() {
            MainApp {}
        } else {
            div { class: "app loading", "Loading\u{2026}" }
        }
    }
}

#[component]
fn MainApp() -> Element {
    let mut tab = use_signal(|| Tab::Payment);

    rsx! {
        div { class: "app",
            h1 { "Mortgage Calculator" }
            nav { class: "tabs",
                TabButton { label: "Payment", active: tab() == Tab::Payment, onclick: move |_| tab.set(Tab::Payment) }
                TabButton {
                    label: "Amortization",
                    active: tab() == Tab::Amortization,
                    onclick: move |_| tab.set(Tab::Amortization),
                }
                TabButton {
                    label: "Affordability",
                    active: tab() == Tab::Affordability,
                    onclick: move |_| tab.set(Tab::Affordability),
                }
                TabButton {
                    label: "Refinance",
                    active: tab() == Tab::Refinance,
                    onclick: move |_| tab.set(Tab::Refinance),
                }
                TabButton { label: "Compare", active: tab() == Tab::Compare, onclick: move |_| tab.set(Tab::Compare) }
            }

            match tab() {
                Tab::Payment => rsx! {
                    screens::PaymentScreen {}
                },
                Tab::Amortization => rsx! {
                    screens::AmortizationScreen {}
                },
                Tab::Affordability => rsx! {
                    screens::AffordabilityScreen {}
                },
                Tab::Refinance => rsx! {
                    screens::RefinanceScreen {}
                },
                Tab::Compare => rsx! {
                    screens::CompareScreen {}
                },
            }
        }
    }
}

#[component]
fn TabButton(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: if active { "tab active" } else { "tab" },
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}
