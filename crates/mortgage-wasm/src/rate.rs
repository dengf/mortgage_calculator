//! What a quoted rate assumes, as a sentence the UI can translate.
//!
//! Every tab that shows a figure derived from a rate has to be able to ask
//! "was anything held still to get this?", and get the same answer worded
//! the same way. That is one binding rather than a field on five result
//! DTOs, and the wording is composed from a code and its values here rather
//! than written into five components — the convention errors and preset
//! names already use.
//!
//! The rule itself is not here. Whether a quote rests on something that
//! moves is decided by `mortgage_calc::RateType::floating_base`; this turns
//! its answer into text.

use wasm_bindgen::prelude::*;

use rust_decimal::Decimal;

use crate::convert::{rate_to_percent, rate_type_from_dto, to_js};
use crate::dto::RateTypeDto;
use crate::message::Message;

/// What the reader has to be told about a rate before the figures beside it
/// mean anything. `null` when there is nothing to tell them.
#[wasm_bindgen]
pub fn rate_note(rate: JsValue) -> JsValue {
    let note = serde_wasm_bindgen::from_value::<RateTypeDto>(rate)
        .ok()
        .and_then(|dto| rate_type_from_dto(&dto).ok())
        .and_then(|rate| floating_base_note(rate.floating_base()));
    to_js(&note)
}

/// The disclosure a quote resting on a moving benchmark requires.
///
/// Says three things, and needs all three: the figure that was held still,
/// that it is published rather than agreed, and that this calculator does
/// not read it. A reader who is told only the first assumes the second was
/// checked.
///
/// It does not predict. MAS publishes SORA daily and this app ships no
/// market feed; the honest position is that today's print was used and the
/// figures move with it, which is also what a bank must admit on a Notice
/// 632A fact sheet.
/// Takes the base itself rather than the rate it came from, so the document
/// -- which is handed a built report and never sees a `RateType` -- prints
/// the same sentence as the tabs rather than a second one that drifts.
pub fn floating_base_note(base: Option<Decimal>) -> Option<Message> {
    let base = format!("{:.2}", rate_to_percent(base?));
    Some(Message::with_params(
        "note.floatingBase",
        [("base".to_string(), base.clone())],
        format!(
            "These figures assume the base rate stays at {base}%. It is a published \
             benchmark that moves, this calculator does not track it, and every amount \
             shown moves with it."
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mortgage_calc::RateType;
    use rust_decimal_macros::dec;

    fn package(base_floats: bool) -> RateType {
        RateType::Reverting {
            base_rate: dec!(0.0112),
            base_floats,
            initial_spread: dec!(0.003),
            initial_years: dec!(2),
            thereafter_spread: dec!(0.006),
        }
    }

    #[test]
    fn a_package_on_sora_names_the_figure_that_was_held_still() {
        let note =
            floating_base_note(package(true).floating_base()).expect("a SORA package discloses");
        assert_eq!(note.code, "note.floatingBase");
        // The base, not the rate charged: 1.12%, not 1.42%. Quoting the
        // rate back would tell the reader nothing they cannot already see.
        assert_eq!(note.params.get("base").unwrap(), "1.12");
    }

    #[test]
    fn a_package_on_agreed_rates_says_nothing() {
        assert_eq!(floating_base_note(package(false).floating_base()), None);
        assert_eq!(
            floating_base_note(RateType::Fixed { rate: dec!(0.065) }.floating_base()),
            None
        );
    }

    #[test]
    fn a_floating_quote_always_discloses() {
        let note = floating_base_note(
            RateType::Floating {
                base_rate: dec!(0.0363),
                spread: dec!(0.02),
            }
            .floating_base(),
        )
        .expect("a floating quote rests on a benchmark by construction");
        assert_eq!(note.params.get("base").unwrap(), "3.63");
    }

    #[test]
    fn the_english_fallback_admits_the_calculator_has_no_feed() {
        // The fallback is what a UI with no catalog entry shows, and it is
        // the wording the translations are checked against. A sentence that
        // says only "rates may vary" leaves the reader thinking the base was
        // looked up.
        let text = floating_base_note(package(true).floating_base())
            .unwrap()
            .text;
        assert!(text.contains("1.12%"), "{text}");
        assert!(text.contains("does not track it"), "{text}");
    }

    #[test]
    fn the_default_for_a_step_up_with_no_stated_basis_is_that_it_floats() {
        // A scenario saved before the field existed. Every one was seeded
        // from a SGD preset quoted over SORA.
        let dto: RateTypeDto = serde_json::from_str(
            r#"{"kind":"reverting","base_rate_percent":1.12,"initial_spread_percent":0.3,
                "initial_years":2,"thereafter_spread_percent":0.6}"#,
        )
        .expect("an old record still loads");
        let rate = rate_type_from_dto(&dto).unwrap();
        assert_eq!(rate.floating_base(), Some(dec!(0.0112)));
    }
}
