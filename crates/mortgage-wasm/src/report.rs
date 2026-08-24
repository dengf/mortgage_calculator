//! `build_report`: the figures a client-facing loan illustration states.
//!
//! Bridge only. What the document says is decided in
//! [`mortgage_calc::report`]; how it looks is the front end's.

use wasm_bindgen::prelude::*;

use crate::convert::{decimal_to_f64, frequency_name, rate_to_percent, to_js};
use crate::dto::{
    PaymentBandDto, RateRiseRowDto, ReferenceDto, ReportParams, ReportResult, ReportScheduleRowDto,
};
use crate::loan::build_loan;
use crate::message::Message;
use crate::rate::floating_base_note;

#[wasm_bindgen]
pub fn build_report(params: JsValue) -> JsValue {
    let result = build_report_impl(params);
    to_js(&result)
}

fn build_report_impl(params: JsValue) -> ReportResult {
    match serde_wasm_bindgen::from_value(params) {
        Ok(params) => report_from_params(params),
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
fn report_from_params(params: ReportParams) -> ReportResult {
    let loan = match build_loan(&params.loan) {
        Ok(loan) => loan,
        Err(e) => return failed(e),
    };

    // An unrecognized or absent region falls back to the default rather
    // than failing: a document with the wrong citations is a bug, but a
    // document that refuses to exist is worse.
    let region = params
        .region
        .as_deref()
        .and_then(mortgage_core::Region::try_parse)
        .unwrap_or_default();

    let report = match mortgage_calc::report::report(&loan, region) {
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
        frequency: Some(frequency_name(report.frequency).to_string()),
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
        floating_base_percent: report.floating_base.map(rate_to_percent),
        rate_note: floating_base_note(report.floating_base),
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
        schedule: report
            .schedule
            .into_iter()
            .map(|row| ReportScheduleRowDto {
                period: row.period,
                paid: decimal_to_f64(row.payment),
                principal: decimal_to_f64(row.principal_portion),
                interest: decimal_to_f64(row.interest_portion),
                remaining_balance: decimal_to_f64(row.remaining_balance),
            })
            .collect(),
        references: report
            .references
            .into_iter()
            .map(|authority| ReferenceDto {
                code: format!("ref.{authority:?}"),
                url: authority.url().to_string(),
            })
            .collect(),
        error: None,
        error_message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{LoanParams, RateTypeDto};

    fn package() -> ReportParams {
        ReportParams {
            region: Some("SG".to_string()),
            loan: LoanParams {
                principal: 400_000.0,
                rate: RateTypeDto::Reverting {
                    base_rate_percent: 1.12,
                    base_floats: true,
                    initial_spread_percent: 0.3,
                    initial_years: 2.0,
                    thereafter_spread_percent: 0.6,
                },
                term_years: 25.0,
                frequency: None,
            },
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
        let r = report_from_params(ReportParams {
            loan: LoanParams {
                term_years: 0.0,
                ..package().loan
            },
            ..package()
        });

        assert!(r.error.is_some());
        assert!(r.bands.is_empty());
        assert!(r.principal.is_none());
    }

    #[test]
    fn cites_the_market_the_document_will_be_read_in() {
        let sg = report_from_params(package());
        let us = report_from_params(ReportParams {
            region: Some("US".to_string()),
            ..package()
        });

        let codes = |r: &ReportResult| {
            r.references
                .iter()
                .map(|x| x.code.clone())
                .collect::<Vec<_>>()
        };

        assert!(codes(&sg).contains(&"ref.MasNotice645".to_string()));
        assert!(codes(&us).contains(&"ref.Fhfa".to_string()));
        assert!(!codes(&us).contains(&"ref.MasNotice645".to_string()));
        assert!(sg.references.iter().all(|r| r.url.starts_with("https://")));
    }
}
