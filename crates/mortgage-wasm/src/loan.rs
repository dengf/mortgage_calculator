//! Shared helper for turning a [`LoanParams`] DTO into a validated
//! [`mortgage_calc::Loan`], used by every wasm entrypoint that takes loan
//! terms.

use mortgage_calc::Loan;

use crate::convert::{f64_to_decimal, parse_frequency, percent_to_rate};
use crate::dto::LoanParams;

pub fn build_loan(params: &LoanParams) -> Result<Loan, String> {
    Loan::builder()
        .principal(f64_to_decimal(params.principal))
        .annual_rate(percent_to_rate(params.annual_rate_percent))
        .term_years(f64_to_decimal(params.term_years))
        .frequency(parse_frequency(params.frequency.as_deref()))
        .build()
        .map_err(|e| e.to_string())
}
