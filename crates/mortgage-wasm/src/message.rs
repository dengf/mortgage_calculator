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
