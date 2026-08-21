//! `init_storage`, `save_scenario`, `list_scenarios`, `load_scenario`,
//! `delete_scenario`.
//!
//! The open [`RedbScenarioStore`] lives in a thread-local `Rc` (wasm32 is
//! single-threaded, so this is just "one instance per page load," not
//! actually per-thread). Every entrypoint below clones the `Rc` out of the
//! `RefCell` *before* doing any `await`ed work — holding a `Ref`/`RefMut`
//! across an `.await` would panic the moment two calls interleave (e.g. the
//! user double-clicks "Save"), since JS can resume a second call's future
//! while the first is still suspended.

use std::cell::RefCell;
use std::rc::Rc;

use mortgage_ports::{CalculatorKind, Scenario, ScenarioStore};
use wasm_bindgen::prelude::*;

use crate::convert::{calculator_kind_to_str, parse_calculator_kind};
use crate::dto::{
    DeleteScenarioResult, SaveScenarioParams, SaveScenarioResult, ScenarioDto, ScenarioListResult,
    ScenarioResult,
};

thread_local! {
    static STORE: RefCell<Option<Rc<mortgage_ext_redb::RedbScenarioStore>>> = const { RefCell::new(None) };
}

fn get_store() -> Result<Rc<mortgage_ext_redb::RedbScenarioStore>, String> {
    STORE.with(|cell| {
        cell.borrow()
            .clone()
            .ok_or_else(|| "storage not initialized; call init_storage() first".to_string())
    })
}

fn new_scenario_id() -> String {
    let now = js_sys::Date::now();
    let random = (js_sys::Math::random() * 1e9) as u64;
    format!("{now:.0}-{random:x}")
}

fn to_dto(scenario: Scenario) -> ScenarioDto {
    ScenarioDto {
        id: scenario.id,
        calculator: calculator_kind_to_str(scenario.calculator).to_string(),
        name: scenario.name,
        created_at: scenario.created_at,
        inputs_json: scenario.inputs_json,
    }
}

/// Must be called (and awaited) once before any other function in this
/// module — it loads any previously-persisted data from IndexedDB and
/// opens the in-memory-backed redb database against it.
#[wasm_bindgen]
pub async fn init_storage() -> Result<(), JsValue> {
    let store = mortgage_ext_redb::RedbScenarioStore::open_wasm()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    STORE.with(|cell| *cell.borrow_mut() = Some(Rc::new(store)));
    Ok(())
}

#[wasm_bindgen]
pub async fn save_scenario(params: JsValue) -> JsValue {
    let result = save_scenario_impl(params).await;
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

async fn save_scenario_impl(params: JsValue) -> SaveScenarioResult {
    let params: SaveScenarioParams = match serde_wasm_bindgen::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            return SaveScenarioResult {
                error: Some(format!("Failed to parse scenario parameters: {e:?}")),
                ..Default::default()
            }
        }
    };

    let calculator = match parse_calculator_kind(&params.calculator) {
        Ok(c) => c,
        Err(e) => {
            return SaveScenarioResult {
                error: Some(e),
                ..Default::default()
            }
        }
    };

    let store = match get_store() {
        Ok(s) => s,
        Err(e) => {
            return SaveScenarioResult {
                error: Some(e),
                ..Default::default()
            }
        }
    };

    let id = params.id.unwrap_or_else(new_scenario_id);
    let scenario = Scenario {
        id: id.clone(),
        calculator,
        name: params.name,
        created_at: js_sys::Date::now() as i64,
        inputs_json: params.inputs_json,
    };

    match store.save(scenario).await {
        Ok(()) => SaveScenarioResult {
            id: Some(id),
            error: None,
        },
        Err(e) => SaveScenarioResult {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

#[wasm_bindgen]
pub async fn list_scenarios(calculator: Option<String>) -> JsValue {
    let result = list_scenarios_impl(calculator).await;
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

async fn list_scenarios_impl(calculator: Option<String>) -> ScenarioListResult {
    let filter: Option<CalculatorKind> = match calculator.as_deref().map(parse_calculator_kind) {
        Some(Ok(kind)) => Some(kind),
        Some(Err(e)) => {
            return ScenarioListResult {
                error: Some(e),
                ..Default::default()
            }
        }
        None => None,
    };

    let store = match get_store() {
        Ok(s) => s,
        Err(e) => {
            return ScenarioListResult {
                error: Some(e),
                ..Default::default()
            }
        }
    };

    match store.list(filter).await {
        Ok(scenarios) => ScenarioListResult {
            scenarios: scenarios.into_iter().map(to_dto).collect(),
            error: None,
        },
        Err(e) => ScenarioListResult {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

#[wasm_bindgen]
pub async fn load_scenario(id: String) -> JsValue {
    let result = load_scenario_impl(id).await;
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

async fn load_scenario_impl(id: String) -> ScenarioResult {
    let store = match get_store() {
        Ok(s) => s,
        Err(e) => {
            return ScenarioResult {
                error: Some(e),
                ..Default::default()
            }
        }
    };

    match store.load(&id).await {
        Ok(scenario) => ScenarioResult {
            scenario: Some(to_dto(scenario)),
            error: None,
        },
        Err(e) => ScenarioResult {
            error: Some(e.to_string()),
            ..Default::default()
        },
    }
}

#[wasm_bindgen]
pub async fn delete_scenario(id: String) -> JsValue {
    let result = delete_scenario_impl(id).await;
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

async fn delete_scenario_impl(id: String) -> DeleteScenarioResult {
    let store = match get_store() {
        Ok(s) => s,
        Err(e) => {
            return DeleteScenarioResult {
                error: Some(e),
                success: false,
            }
        }
    };

    match store.delete(&id).await {
        Ok(()) => DeleteScenarioResult {
            success: true,
            error: None,
        },
        Err(e) => DeleteScenarioResult {
            success: false,
            error: Some(e.to_string()),
        },
    }
}
