//! `build_report`: the figures a client-facing loan illustration states.
//!
//! Bridge only. What the document says is decided in
//! [`mortgage_calc::report`]; how it looks is the front end's.

use wasm_bindgen::prelude::*;

use crate::convert::{decimal_to_f64, rate_to_percent, to_js};
use crate::dto::{LoanParams, PaymentBandDto, RateRiseRowDto, ReportResult};
use crate::loan::build_loan;
use crate::message::Message;

#[wasm_bindgen]
pub fn build_report(params: JsValue) -> JsValue {
    let result = build_report_impl(params);
    to_js(&result)
}

fn build_report_impl(params: JsValue) -> ReportResult {
    match serde_wasm_bindgen::from_value(params) {
        Ok(loan_params) => report_from_params(loan_params),
        Err(_) => failed(Message::bad_request()),
    }
}

fn failed(message: Message) -> ReportResult {
    ReportResult {
        error: Some(message.text.clone()),
        error_message: Some(message),
        ..Default::default()
    }
}

/// JsValue-free core — see the matching comment on
/// `payment::payment_from_params`.
fn report_from_params(loan_params: LoanParams) -> ReportResult {
    let loan = match build_loan(&loan_params) {
        Ok(loan) => loan,
        Err(e) => return failed(e),
    };

    let report = match mortgage_calc::report::report(&loan) {
        Ok(report) => report,
        Err(e) => return failed(Message::from(&e)),
    };

    ReportResult {
        principal: Some(decimal_to_f64(report.principal)),
        term_years: Some(decimal_to_f64(report.term_years)),
        initial_rate_percent: Some(rate_to_percent(report.initial_annual_rate)),
        initial_payment: Some(decimal_to_f64(report.initial_payment)),
        final_rate_percent: Some(rate_to_percent(report.final_annual_rate)),
        payment_after_reversion: report.payment_after_reversion.map(decimal_to_f64),
        lock_in_years: report.lock_in_years.map(decimal_to_f64),
        total_paid: Some(decimal_to_f64(report.total_paid)),
        total_interest: Some(decimal_to_f64(report.total_interest)),
        interest_share_percent: report.interest_share.map(decimal_to_f64),
        bands: report
            .bands
            .into_iter()
            .map(|band| PaymentBandDto {
                from_year: decimal_to_f64(band.from_year),
                to_year: decimal_to_f64(band.to_year),
                annual_rate_percent: rate_to_percent(band.annual_rate),
                payment: decimal_to_f64(band.payment),
            })
            .collect(),
        rate_rise: report
            .rate_rise
            .into_iter()
            .map(|row| RateRiseRowDto {
                increase_percent: rate_to_percent(row.increase),
                annual_rate_percent: rate_to_percent(row.annual_rate),
                payment: decimal_to_f64(row.payment),
                payment_increase: decimal_to_f64(row.payment_increase),
            })
            .collect(),
        yearly: report
            .yearly
            .into_iter()
            .map(crate::amortization::to_year_dto)
            .collect(),
        error: None,
        error_message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::RateTypeDto;

    fn package() -> LoanParams {
        LoanParams {
            principal: 400_000.0,
            rate: RateTypeDto::Reverting {
                base_rate_percent: 1.12,
                initial_spread_percent: 0.3,
                initial_years: 2.0,
                thereafter_spread_percent: 0.6,
            },
            term_years: 25.0,
            frequency: None,
        }
    }

    #[test]
    fn describes_a_package_over_time_rather_than_as_one_figure() {
        let r = report_from_params(package());

        assert!(r.error.is_none());
        assert_eq!(r.bands.len(), 2);
        assert_eq!((r.bands[0].from_year, r.bands[0].to_year), (1.0, 2.0));
        assert_eq!((r.bands[1].from_year, r.bands[1].to_year), (3.0, 25.0));
        assert!(r.bands[1].payment > r.bands[0].payment);
    }

    #[test]
    fn carries_the_rate_change_illustration_a_fact_sheet_needs() {
        // MoneySense lists it among the fact sheet's required contents:
        // how possible increases in interest rates will affect the monthly
        // instalment.
        let r = report_from_params(package());

        assert_eq!(r.rate_rise.len(), 3);
        assert!(r.rate_rise[0].payment_increase > 0.0);
        assert!(r.rate_rise[2].payment > r.rate_rise[0].payment);
    }

    #[test]
    fn reports_a_bad_loan_in_band_rather_than_returning_an_empty_document() {
        let r = report_from_params(LoanParams {
            term_years: 0.0,
            ..package()
        });

        assert!(r.error.is_some());
        assert!(r.bands.is_empty());
        assert!(r.principal.is_none());
    }
}
