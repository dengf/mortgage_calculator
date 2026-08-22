//! Shared helper for turning a [`LoanParams`] DTO into a validated
//! [`mortgage_calc::Loan`], used by every wasm entrypoint that takes loan
//! terms.

use mortgage_calc::Loan;

use crate::message::Message;

use crate::convert::{f64_to_decimal, parse_frequency, percent_to_rate};
use crate::dto::LoanParams;

pub fn build_loan(params: &LoanParams) -> Result<Loan, Message> {
    let annual_rate = percent_to_rate(params.annual_rate_percent).ok_or_else(|| {
        Message::bare(
            "err.invalidRate",
            "annual_rate_percent must be a finite number",
        )
    })?;

    Loan::builder()
        .principal(f64_to_decimal(params.principal))
        .annual_rate(annual_rate)
        .term_years(f64_to_decimal(params.term_years))
        .frequency(parse_frequency(params.frequency.as_deref()))
        .build()
        .map_err(|e| Message::from(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_params() -> LoanParams {
        LoanParams {
            principal: 400_000.0,
            annual_rate_percent: 6.5,
            term_years: 30.0,
            frequency: None,
        }
    }

    #[test]
    fn builds_a_valid_loan() {
        assert!(build_loan(&valid_params()).is_ok());
    }

    #[test]
    fn rejects_a_nan_rate_instead_of_silently_building_a_zero_percent_loan() {
        let params = LoanParams {
            annual_rate_percent: f64::NAN,
            ..valid_params()
        };
        assert!(build_loan(&params).is_err());
    }
}
