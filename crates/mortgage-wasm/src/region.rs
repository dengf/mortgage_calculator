//! `detect_region`: which market's rules to start the app in.
//!
//! The host gathers the raw observations it can make — a stored choice, the
//! device time zone, the preferred language tags — and this hands back a
//! region. The ranking between those signals is `mortgage_core::Region`'s,
//! not JavaScript's, so the web and desktop apps cannot drift apart on the
//! question of where the user is.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::convert::to_js;
use mortgage_core::{Region, RegionSignals};

#[derive(Debug, Default, Deserialize)]
pub struct RegionSignalsParams {
    #[serde(default)]
    pub chosen: Option<String>,
    #[serde(default)]
    pub time_zone: Option<String>,
    #[serde(default)]
    pub locales: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RegionResult {
    pub region: String,
}

#[wasm_bindgen]
pub fn detect_region(params: JsValue) -> JsValue {
    let result = detect_region_impl(params);
    to_js(&result)
}

fn detect_region_impl(params: JsValue) -> RegionResult {
    // Unparseable input is answered with the default rather than an error.
    // This runs once to pick a first render's ruleset, and every signal it
    // reads is optional and best-effort; there is no user mistake here to
    // report, and the header toggle is right there either way.
    let params: RegionSignalsParams = serde_wasm_bindgen::from_value(params).unwrap_or_default();
    RegionResult {
        region: region_from_params(&params).as_str().to_string(),
    }
}

/// The JsValue-free core, so the wiring is testable on the native host.
fn region_from_params(params: &RegionSignalsParams) -> Region {
    let locales: Vec<&str> = params.locales.iter().map(String::as_str).collect();
    Region::detect(RegionSignals {
        chosen: params.chosen.as_deref(),
        time_zone: params.time_zone.as_deref(),
        locales: &locales,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(
        chosen: Option<&str>,
        time_zone: Option<&str>,
        locales: &[&str],
    ) -> RegionSignalsParams {
        RegionSignalsParams {
            chosen: chosen.map(str::to_string),
            time_zone: time_zone.map(str::to_string),
            locales: locales.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn passes_every_signal_through_to_the_core_ranking() {
        assert_eq!(
            region_from_params(&params(None, Some("Asia/Singapore"), &["en-GB"])),
            Region::SG
        );
        assert_eq!(
            region_from_params(&params(Some("US"), Some("Asia/Singapore"), &["en-SG"])),
            Region::US
        );
    }

    #[test]
    fn an_empty_signal_set_is_the_default_region() {
        assert_eq!(
            region_from_params(&RegionSignalsParams::default()),
            Region::US
        );
    }
}
