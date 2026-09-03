use thiserror::Error;

/// Errors raised while validating or calculating mortgage figures.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MortgageError {
    #[error("principal must be positive, got {0}")]
    InvalidPrincipal(String),

    #[error("annual interest rate must be zero or positive, got {0}")]
    InvalidRate(String),

    #[error("annual interest rate is unreasonably high, got {0}")]
    RateTooHigh(String),

    #[error("loan term must be at least one payment period, got {0} periods")]
    InvalidTerm(u32),

    #[error("loan term is unreasonably long, got {0} payment periods")]
    TermTooLong(u32),

    #[error("down payment {down_payment} cannot exceed home price {home_price}")]
    DownPaymentExceedsPrice {
        down_payment: String,
        home_price: String,
    },

    #[error("monthly income must be positive, got {0}")]
    InvalidIncome(String),

    #[error("max debt-to-income ratio must be between 0 and 1, got {0}")]
    InvalidDtiRatio(String),

    #[error("extra payment must be zero or positive, got {0}")]
    InvalidExtraPayment(String),

    #[error("failed to parse input: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // mortgage-ui-slint now forwards these Display messages straight to
    // the user (instead of a generic "enter valid details" string), so a
    // wording change here is a UI-visible change, not just an internal
    // refactor — this pins the exact text so that's a deliberate edit.
    #[test]
    fn display_messages_are_specific_about_which_value_and_constraint_failed() {
        assert_eq!(
            MortgageError::InvalidPrincipal("-100".into()).to_string(),
            "principal must be positive, got -100"
        );
        assert_eq!(
            MortgageError::TermTooLong(52_000_000).to_string(),
            "loan term is unreasonably long, got 52000000 payment periods"
        );
        assert_eq!(
            MortgageError::DownPaymentExceedsPrice {
                down_payment: "600000".into(),
                home_price: "500000".into(),
            }
            .to_string(),
            "down payment 600000 cannot exceed home price 500000"
        );
    }
}
