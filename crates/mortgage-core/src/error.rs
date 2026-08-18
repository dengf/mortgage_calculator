use thiserror::Error;

/// Errors raised while validating or calculating mortgage figures.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MortgageError {
    #[error("principal must be positive, got {0}")]
    InvalidPrincipal(String),

    #[error("annual interest rate must be zero or positive, got {0}")]
    InvalidRate(String),

    #[error("loan term must be at least one payment period, got {0} periods")]
    InvalidTerm(u32),

    #[error("down payment {down_payment} cannot exceed home price {home_price}")]
    DownPaymentExceedsPrice { down_payment: String, home_price: String },

    #[error("monthly income must be positive, got {0}")]
    InvalidIncome(String),

    #[error("max debt-to-income ratio must be between 0 and 1, got {0}")]
    InvalidDtiRatio(String),

    #[error("extra payment must be zero or positive, got {0}")]
    InvalidExtraPayment(String),

    #[error("failed to parse input: {0}")]
    ParseError(String),
}
