//! Shared helper for turning a [`LoanParams`] DTO into a validated
//! [`mortgage_calc::Loan`], used by every wasm entrypoint that takes loan
//! terms.

use mortgage_calc::Loan;

use crate::message::Message;

use crate::convert::{f64_to_decimal, parse_frequency, rate_type_from_dto};
use crate::dto::LoanParams;

pub fn build_loan(params: &LoanParams) -> Result<Loan, Message> {
    // `rate_type` rather than `annual_rate`: a reverting package sets its
    // own step-up on the way in, so there is no path through this function
    // that builds a Singapore loan as though its promotional rate ran for
    // twenty-five years.
    let rate_type =
        rate_type_from_dto(&params.rate).map_err(|text| Message::bare("err.invalidRate", &text))?;

    Loan::builder()
        .principal(f64_to_decimal(params.principal))
        .rate_type(rate_type)
        .term_years(f64_to_decimal(params.term_years))
        .frequency(parse_frequency(params.frequency.as_deref()))
        .build()
        .map_err(|e| Message::from(&e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::RateTypeDto;

    fn valid_params() -> LoanParams {
        LoanParams {
            principal: 400_000.0,
            rate: RateTypeDto::Fixed { rate_percent: 6.5 },
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
            rate: RateTypeDto::Fixed {
                rate_percent: f64::NAN,
            },
            ..valid_params()
        };
        assert!(build_loan(&params).is_err());
    }

    #[test]
    fn a_reverting_package_keeps_its_step_up() {
        // The whole point of taking a shape rather than a figure: every
        // caller of this helper -- payment, schedule, extra-payment impact
        // -- inherits the reversion instead of having to remember it.
        let loan = build_loan(&LoanParams {
            rate: RateTypeDto::Reverting {
                base_rate_percent: 1.12,
                initial_spread_percent: 0.3,
                initial_years: 2.0,
                thereafter_spread_percent: 0.6,
            },
            term_years: 25.0,
            ..valid_params()
        })
        .unwrap();

        assert!(loan.reversion().is_some());
        assert!(loan.final_annual_rate() > loan.annual_rate());
    }
}
