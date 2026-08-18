use std::rc::Rc;

use dioxus::prelude::*;
use mortgage_ext_redb::RedbScenarioStore;
use mortgage_ports::{CalculatorKind, Scenario, ScenarioStore};

use crate::storage;

/// Provided by `App` via `use_context_provider`. A `Signal` (cheap `Copy`)
/// rather than the raw `Rc<RedbScenarioStore>`, since `RedbScenarioStore`
/// has no `PartialEq` — every read here re-derives an owned `Rc` fresh via
/// `.expect(...)` rather than holding one shared clone across closures,
/// which avoids move/borrow conflicts between the several event handlers
/// below that each need their own copy.
pub type StoreSignal = Signal<Option<Rc<RedbScenarioStore>>>;

/// Save/list/load/delete panel for one calculator's scenarios.
#[component]
pub fn SavedScenarios(
    calculator: CalculatorKind,
    current_inputs: String,
    on_load: EventHandler<String>,
) -> Element {
    let store_signal = use_context::<StoreSignal>();

    let mut refresh_key = use_signal(|| 0u32);
    let mut is_saving = use_signal(|| false);
    let mut name_input = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);

    let scenarios = use_resource(move || {
        let store = store_signal().expect("SavedScenarios only mounts once the store is loaded");
        let _dep = refresh_key();
        async move { store.list(Some(calculator)).await.unwrap_or_default() }
    });

    rsx! {
        div { class: "saved-scenarios",
            div { class: "saved-scenarios-header",
                h3 { "Saved scenarios" }
                if is_saving() {
                    div { class: "save-form",
                        input {
                            value: "{name_input}",
                            placeholder: "Scenario name",
                            oninput: move |e| name_input.set(e.value()),
                        }
                        button {
                            class: "link-button",
                            onclick: move |_| {
                                let name = name_input().trim().to_string();
                                if name.is_empty() {
                                    return;
                                }
                                let store = store_signal().expect("store loaded");
                                let inputs_json = current_inputs.clone();
                                spawn(async move {
                                    let scenario = Scenario {
                                        id: storage::new_scenario_id(),
                                        calculator,
                                        name,
                                        created_at: storage::now_millis(),
                                        inputs_json,
                                    };
                                    if let Err(e) = store.save(scenario).await {
                                        error.set(Some(e.to_string()));
                                    }
                                    refresh_key += 1;
                                });
                                name_input.set(String::new());
                                is_saving.set(false);
                            },
                            "Save",
                        }
                        button { class: "link-button", onclick: move |_| is_saving.set(false), "Cancel" }
                    }
                } else {
                    button { class: "link-button", onclick: move |_| is_saving.set(true), "+ Save current as..." }
                }
            }

            if let Some(err) = error() {
                p { class: "error", "{err}" }
            }

            match &*scenarios.read() {
                Some(list) if !list.is_empty() => rsx! {
                    ul { class: "saved-scenarios-list",
                        for s in list.iter().cloned() {
                            ScenarioRow {
                                key: "{s.id}",
                                scenario: s,
                                on_load,
                                on_delete: move |id: String| {
                                    let store = store_signal().expect("store loaded");
                                    spawn(async move {
                                        let _ = store.delete(&id).await;
                                    });
                                    refresh_key += 1;
                                },
                            }
                        }
                    }
                },
                _ => rsx! { p { class: "saved-scenarios-empty", "No saved scenarios yet." } },
            }
        }
    }
}

#[component]
fn ScenarioRow(scenario: Scenario, on_load: EventHandler<String>, on_delete: EventHandler<String>) -> Element {
    let inputs = scenario.inputs_json.clone();
    let id_for_delete = scenario.id.clone();

    rsx! {
        li {
            span { class: "scenario-name", "{scenario.name}" }
            button { class: "link-button", onclick: move |_| on_load.call(inputs.clone()), "Load" }
            button {
                class: "link-button danger",
                onclick: move |_| on_delete.call(id_for_delete.clone()),
                "Delete",
            }
        }
    }
}
