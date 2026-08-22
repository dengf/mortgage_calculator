//! `describe_duration`: a count of payment periods in the units people think in.
//!
//! Nobody plans around "127 payments". This converts a period count into
//! years and whole months, and reports the cadence it was measured against
//! so a caller does not have to keep its own table of periods per year.
//!
//! The wording is composed by the front end from its own catalogs -- this
//! returns numbers, not a sentence.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::convert::{decimal_to_f64, parse_frequency};

#[derive(Debug, Default, Deserialize)]
pub struct DurationParams {
    #[serde(default)]
    pub periods: u32,
    #[serde(default)]
    pub frequency: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct DurationResult {
    pub years: u32,
    /// Months left over after the whole years, 0-11.
    pub months: u32,
    pub total_months: u32,
    /// The same span as a fraction of a year, for an axis label that has to
    /// read "12.4 yr" rather than snap to a whole one.
    pub years_exact: f64,
    /// The cadence this was measured against, so callers do not keep their
    /// own monthly/biweekly/weekly table.
    pub periods_per_year: u32,
}

#[wasm_bindgen]
pub fn describe_duration(params: JsValue) -> JsValue {
    let result = describe_duration_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn describe_duration_impl(params: JsValue) -> DurationResult {
    // A duration is read on every render, including before a schedule
    // exists. Unparseable input describes no time at all, which is what an
    // absent schedule is.
    let params: DurationParams = serde_wasm_bindgen::from_value(params).unwrap_or_default();
    duration_of(&params)
}

/// The JsValue-free core, so this is testable on the native host.
fn duration_of(params: &DurationParams) -> DurationResult {
    let frequency = parse_frequency(params.frequency.as_deref());
    let total_months = frequency.periods_to_months(params.periods);
    DurationResult {
        years: total_months / 12,
        months: total_months % 12,
        total_months,
        years_exact: decimal_to_f64(frequency.periods_to_years(params.periods)),
        periods_per_year: frequency.periods_per_year(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn describe(periods: u32, frequency: &str) -> DurationResult {
        duration_of(&DurationParams {
            periods,
            frequency: Some(frequency.to_string()),
        })
    }

    #[test]
    fn splits_a_monthly_term_into_years_and_months() {
        let d = describe(127, "monthly");
        assert_eq!((d.years, d.months), (10, 7));
        assert_eq!(d.total_months, 127);
        assert_eq!(d.periods_per_year, 12);
    }

    #[test]
    fn measures_a_fortnightly_term_against_its_own_cadence() {
        // 260 fortnights is ten years, not 260 months.
        let d = describe(260, "biweekly");
        assert_eq!((d.years, d.months), (10, 0));
        assert_eq!(d.periods_per_year, 26);
    }

    #[test]
    fn reports_a_fractional_year_for_an_axis_label() {
        let d = describe(150, "monthly");
        assert_eq!(d.years_exact, 12.5);
    }

    #[test]
    fn an_unknown_cadence_falls_back_to_monthly() {
        assert_eq!(describe(12, "fortnightly-ish").periods_per_year, 12);
    }
}
