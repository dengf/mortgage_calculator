//! Stable message codes for text the UI has to show a user.
//!
//! `MortgageError`'s `Display` output is English prose. The Slint app
//! forwards it to the screen verbatim and the web app used to as well, which
//! meant a translated UI would still report every failure in English —
//! precisely when a reader most needs to understand what happened.
//!
//! So errors cross the boundary as a code plus its parameters, and the UI
//! composes the sentence in whatever language it is running. The English
//! text travels alongside as `text`, both as a fallback for a UI with no
//! catalog entry and so existing consumers keep working unchanged.

use std::collections::BTreeMap;

use mortgage_core::MortgageError;
use serde::Serialize;

/// A user-facing message: what went wrong, the values involved, and a
/// ready-made English rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Message {
    /// Stable identifier, e.g. `"err.invalidPrincipal"`. Matches the key in
    /// the frontend catalogs (www/src/i18n).
    pub code: String,
    /// Values to interpolate, keyed by placeholder name. BTreeMap rather
    /// than HashMap so serialization is deterministic and tests can compare
    /// whole structures.
    pub params: BTreeMap<String, String>,
    /// English rendering, for callers that don't translate.
    pub text: String,
}

impl Message {
    fn new(code: &str, params: BTreeMap<String, String>, text: String) -> Self {
        Self {
            code: code.to_string(),
            params,
            text,
        }
    }

    /// A message with no interpolated values.
    pub fn bare(code: &str, text: impl Into<String>) -> Self {
        Self::new(code, BTreeMap::new(), text.into())
    }

    /// A message with a single `{value}` placeholder — the shape most of
    /// the validation errors take.
    pub fn with_value(code: &str, value: impl Into<String>, text: String) -> Self {
        let mut params = BTreeMap::new();
        params.insert("value".to_string(), value.into());
        Self::new(code, params, text)
    }

    /// The values a caller sent could not be read into the expected shape —
    /// a blank field, a non-numeric string, a missing key.
    ///
    /// Deliberately carries nothing from the underlying
    /// `serde_wasm_bindgen::Error`. That error wraps a JS `Error` object, so
    /// its `Debug` rendering includes a live JavaScript stack trace, and
    /// every caller writes this message straight into the DOM. Formatting it
    /// with `{e:?}` put bundle paths and raw `wasm-function[N]:0x…` offsets
    /// on the page the moment anyone cleared a numeric field — which is the
    /// first thing a visitor does, since they have to clear the default
    /// before typing their own. A reader needs one sentence about their
    /// input, not a backtrace.
    pub fn bad_request() -> Self {
        Message::bare(
            "err.badRequest",
            "Some values are missing or aren't valid numbers. Check the fields above.",
        )
    }
}

impl From<&MortgageError> for Message {
    fn from(error: &MortgageError) -> Self {
        let text = error.to_string();
        match error {
            MortgageError::InvalidPrincipal(v) => {
                Message::with_value("err.invalidPrincipal", v.clone(), text)
            }
            MortgageError::InvalidRate(v) => {
                Message::with_value("err.invalidRate", v.clone(), text)
            }
            MortgageError::InvalidTerm(v) => {
                Message::with_value("err.invalidTerm", v.to_string(), text)
            }
            MortgageError::TermTooLong(v) => {
                Message::with_value("err.termTooLong", v.to_string(), text)
            }
            MortgageError::DownPaymentExceedsPrice {
                down_payment,
                home_price,
            } => {
                let mut params = BTreeMap::new();
                params.insert("downPayment".to_string(), down_payment.clone());
                params.insert("homePrice".to_string(), home_price.clone());
                Message::new("err.downPaymentTooLarge", params, text)
            }
            MortgageError::InvalidIncome(v) => {
                Message::with_value("err.invalidIncome", v.clone(), text)
            }
            MortgageError::InvalidDtiRatio(v) => {
                Message::with_value("err.invalidDti", v.clone(), text)
            }
            MortgageError::InvalidExtraPayment(v) => {
                Message::with_value("err.invalidExtraPayment", v.clone(), text)
            }
            MortgageError::ParseError(v) => Message::with_value("err.parse", v.clone(), text),
        }
    }
}

impl From<MortgageError> for Message {
    fn from(error: MortgageError) -> Self {
        Message::from(&error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carries_the_offending_value_so_the_ui_can_rebuild_the_sentence() {
        let msg = Message::from(&MortgageError::InvalidPrincipal("-100".into()));
        assert_eq!(msg.code, "err.invalidPrincipal");
        assert_eq!(msg.params.get("value").unwrap(), "-100");
    }

    #[test]
    fn keeps_the_english_text_as_a_fallback() {
        let msg = Message::from(&MortgageError::TermTooLong(52_000_000));
        assert_eq!(
            msg.text,
            "loan term is unreasonably long, got 52000000 payment periods"
        );
    }

    #[test]
    fn names_both_values_for_a_two_part_error() {
        let msg = Message::from(&MortgageError::DownPaymentExceedsPrice {
            down_payment: "600000".into(),
            home_price: "500000".into(),
        });
        assert_eq!(msg.code, "err.downPaymentTooLarge");
        assert_eq!(msg.params.get("downPayment").unwrap(), "600000");
        assert_eq!(msg.params.get("homePrice").unwrap(), "500000");
    }

    #[test]
    fn every_variant_maps_to_a_distinct_code() {
        let all = [
            MortgageError::InvalidPrincipal("1".into()),
            MortgageError::InvalidRate("1".into()),
            MortgageError::InvalidTerm(1),
            MortgageError::TermTooLong(1),
            MortgageError::DownPaymentExceedsPrice {
                down_payment: "1".into(),
                home_price: "1".into(),
            },
            MortgageError::InvalidIncome("1".into()),
            MortgageError::InvalidDtiRatio("1".into()),
            MortgageError::InvalidExtraPayment("1".into()),
            MortgageError::ParseError("1".into()),
        ];

        let codes: std::collections::BTreeSet<_> =
            all.iter().map(|e| Message::from(e).code).collect();

        // A duplicated code would make two different failures render as the
        // same sentence.
        assert_eq!(codes.len(), all.len());
        assert!(codes.iter().all(|c| c.starts_with("err.")));
    }
}

/// Guards the boundary against leaking a foreign error's `Debug` output to
/// the page.
///
/// `serde_wasm_bindgen::Error` wraps a JS `Error`, so `{e:?}` renders a live
/// JavaScript stack trace — bundle paths and raw `wasm-function[N]:0x…`
/// offsets. Every binding writes its `error` field straight into the DOM,
/// so formatting a parse failure that way put a wasm backtrace on screen the
/// moment anyone cleared a numeric field.
///
/// This is a source-level check rather than a behavioural one because a
/// `JsValue` can't be constructed off the wasm32 target, so the failing path
/// can't be exercised from a normal `cargo test`.
#[cfg(test)]
mod no_debug_formatted_errors {
    /// Every module that converts a `JsValue` into a result DTO.
    const BINDINGS: &[(&str, &str)] = &[
        ("payment.rs", include_str!("payment.rs")),
        ("amortization.rs", include_str!("amortization.rs")),
        ("affordability.rs", include_str!("affordability.rs")),
        ("refinance.rs", include_str!("refinance.rs")),
        ("comparison.rs", include_str!("comparison.rs")),
        ("united_states.rs", include_str!("united_states.rs")),
        ("singapore.rs", include_str!("singapore.rs")),
        ("storage.rs", include_str!("storage.rs")),
        ("sg_affordability.rs", include_str!("sg_affordability.rs")),
        ("region.rs", include_str!("region.rs")),
        ("scenario.rs", include_str!("scenario.rs")),
        ("duration.rs", include_str!("duration.rs")),
    ];

    #[test]
    fn every_binding_serializes_through_the_json_compatible_helper() {
        // `serde_wasm_bindgen::to_value` turns a Rust map into a JS `Map`,
        // not a plain object. Every message carrying values -- every
        // validation error on the site -- reached the UI as a `Map`, and the
        // interpolator reads its placeholders with `hasOwnProperty`, so
        // readers were shown "(got {value})" with the braces intact. It
        // shipped, in three languages.
        //
        // `convert::to_js` uses the JSON-compatible serializer. This fails if
        // a new binding goes back to the raw call.
        let mut offenders = Vec::new();
        for (name, source) in BINDINGS {
            for (i, line) in source.lines().enumerate() {
                let code = line.split("//").next().unwrap_or(line);
                if code.contains("serde_wasm_bindgen::to_value") {
                    offenders.push(format!("{name}:{}", i + 1));
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "these lines serialize with serde_wasm_bindgen::to_value: {offenders:?}. \
             That renders a map as a JS Map rather than a plain object, and every \
             message parameter silently stops interpolating. Use convert::to_js."
        );
    }

    #[test]
    fn no_binding_debug_formats_an_error_into_a_user_facing_field() {
        let mut offenders = Vec::new();
        for (name, source) in BINDINGS {
            for (i, line) in source.lines().enumerate() {
                let code = line.split("//").next().unwrap_or(line);
                if code.contains("{e:?}") || code.contains("{err:?}") {
                    offenders.push(format!("{name}:{}", i + 1));
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "these lines Debug-format an error that reaches the DOM: {offenders:?}. \
             A serde_wasm_bindgen::Error carries a JS stack trace, so this puts a wasm \
             backtrace on the page. Use Message::bad_request() instead."
        );
    }

    #[test]
    fn the_bad_request_message_names_no_internals() {
        let text = super::Message::bad_request().text;
        for leak in ["wasm", "Error(", "JsValue", "f64", ".js:", "0x"] {
            assert!(
                !text.contains(leak),
                "bad_request text leaks {leak:?} to the reader: {text}"
            );
        }
    }
}
