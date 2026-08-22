//! `calculate_singapore`: the MAS/CPF/IRAS regulatory panel — TDSR and MSR
//! borrowing limits, the CPF-versus-cash split of a monthly payment, and
//! BSD/ABSD stamp duty with the resulting cash needed at completion.
//!
//! This mirrors what the Slint app shows on its Payment tab, so both UIs
//! present the same figures from the same `mortgage_calc::singapore` logic.

use wasm_bindgen::prelude::*;

use mortgage_calc::singapore;
use mortgage_core::round_currency;
use rust_decimal::Decimal;

use crate::convert::{decimal_to_f64, f64_to_decimal};
use crate::dto::{SingaporeParams, SingaporeResult};

#[wasm_bindgen]
pub fn calculate_singapore(params: JsValue) -> JsValue {
    let result = calculate_singapore_impl(params);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn calculate_singapore_impl(params: JsValue) -> SingaporeResult {
    match serde_wasm_bindgen::from_value(params) {
        Ok(p) => singapore_from_params(p),
        Err(e) => SingaporeResult {
            error: Some(format!("Failed to parse Singapore parameters: {e:?}")),
            ..Default::default()
        },
    }
}

fn parse_residency(s: &str) -> singapore::Residency {
    match s {
        "PR" => singapore::Residency::PermanentResident,
        "Foreigner" => singapore::Residency::Foreigner,
        _ => singapore::Residency::Citizen,
    }
}

fn parse_property_count(s: &str) -> singapore::PropertyCount {
    match s {
        "2nd" => singapore::PropertyCount::Second,
        "3rd+" => singapore::PropertyCount::ThirdOrMore,
        _ => singapore::PropertyCount::First,
    }
}

/// Ratios come back from `mortgage_calc` as fractions (0.482); the UI shows
/// percentages.
fn to_percent(ratio: Decimal) -> f64 {
    decimal_to_f64(ratio) * 100.0
}

/// The JsValue-free core, so it can be unit-tested with plain
/// `SingaporeParams` values without needing a wasm32 target.
fn singapore_from_params(p: SingaporeParams) -> SingaporeResult {
    let mut result = SingaporeResult::default();

    if p.loan_type == "HDB Loan" && !singapore::hdb_loan_eligible(p.is_hdb_or_ec) {
        result.loan_type_warning =
            Some("HDB loans are only available for HDB flats/ECs bought from HDB.".to_string());
        result.loan_type_warning_code = Some("warn.hdbLoanIneligible".to_string());
    }

    // TDSR/MSR and the CPF split both need a payment to work from. Stamp
    // duty below does not, so an invalid loan still gets useful output.
    if let Some(payment) = p.monthly_payment.filter(|v| v.is_finite() && *v > 0.0) {
        let payment = f64_to_decimal(payment);
        let income = f64_to_decimal(p.gross_monthly_income);
        let other_debts = f64_to_decimal(p.other_monthly_debts);

        // The ratios are assessed on this loan repriced at MAS's medium-term
        // rate floor, so `check_tdsr_msr` needs the terms, not the payment.
        // The CPF split below stays on the real payment — CPF services what
        // the borrower actually owes, not the stressed figure.
        let assessed_loan = mortgage_calc::Loan::builder()
            .principal(f64_to_decimal(p.principal))
            .annual_rate(f64_to_decimal(p.annual_rate_percent) / Decimal::from(100))
            .term_years(f64_to_decimal(p.term_years))
            .frequency(mortgage_core::PaymentFrequency::Monthly)
            .build();

        if let Ok(check) = assessed_loan
            .and_then(|loan| singapore::check_tdsr_msr(&loan, other_debts, income, p.is_hdb_or_ec))
        {
            result.tdsr_ratio_percent = Some(to_percent(check.tdsr.ratio));
            result.tdsr_exceeded = check.tdsr.exceeded;
            result.tdsr_near_limit = check.tdsr.near_limit;
            result.assessment_rate_percent = Some(to_percent(check.assessment_rate));
            result.assessed_monthly_instalment =
                Some(decimal_to_f64(check.assessed_monthly_instalment));
            if check.tdsr.exceeded {
                result.warnings.push("Exceeds MAS TDSR limit (55%).".into());
                result.warning_codes.push("warn.tdsrExceeded".into());
            }

            if let Some(msr) = check.msr {
                result.msr_ratio_percent = Some(to_percent(msr.ratio));
                result.msr_exceeded = msr.exceeded;
                result.msr_near_limit = msr.near_limit;
                if msr.exceeded {
                    result
                        .warnings
                        .push("Exceeds MAS MSR limit (30%) for HDB/EC.".into());
                    result.warning_codes.push("warn.msrExceeded".into());
                }
            }
        }

        let split = singapore::cpf_cash_split(payment, f64_to_decimal(p.cpf_oa_available));
        result.cpf_used = Some(decimal_to_f64(split.cpf_used));
        result.cash_required = Some(decimal_to_f64(split.cash_required));
    }

    let price = f64_to_decimal(p.home_price).max(Decimal::ZERO);
    let costs = singapore::upfront_costs(
        price,
        parse_residency(&p.residency),
        parse_property_count(&p.property_count),
    );
    result.bsd = decimal_to_f64(costs.bsd);
    result.absd = decimal_to_f64(costs.absd);
    result.upfront_total = decimal_to_f64(costs.total);

    let down_payment = round_currency((price - f64_to_decimal(p.principal)).max(Decimal::ZERO));
    result.down_payment = decimal_to_f64(down_payment);
    result.total_cash_required = decimal_to_f64(round_currency(down_payment + costs.total));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> SingaporeParams {
        SingaporeParams {
            monthly_payment: Some(3_000.0),
            principal: 800_000.0,
            annual_rate_percent: 4.0,
            term_years: 30.0,
            home_price: 1_000_000.0,
            gross_monthly_income: 10_000.0,
            other_monthly_debts: 0.0,
            cpf_oa_available: 1_200.0,
            residency: "Citizen".into(),
            property_count: "1st".into(),
            is_hdb_or_ec: false,
            loan_type: "Bank Loan".into(),
        }
    }

    #[test]
    fn reports_tdsr_and_leaves_msr_unset_for_private_property() {
        let r = singapore_from_params(params());
        // Derived from the loan repriced at the assessment rate, not from
        // the `monthly_payment` the caller passed in: $800k over 30y at 4%
        // is ~$3,819/mo against $10,000 income.
        let ratio = r.tdsr_ratio_percent.unwrap();
        assert!((38.0..38.5).contains(&ratio), "got {ratio}");
        assert!(!r.tdsr_exceeded);
        assert!(r.msr_ratio_percent.is_none());
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn ratios_ignore_the_quoted_payment_and_use_the_mas_floor() {
        // A borrower quoted 1.5% still gets assessed at 4%. Passing a
        // matching low `monthly_payment` must not soften the ratio — that
        // was the bug: the panel reported a payment-derived TDSR a bank
        // would never have accepted.
        let r = singapore_from_params(SingaporeParams {
            annual_rate_percent: 1.5,
            monthly_payment: Some(2_761.0),
            ..params()
        });

        assert_eq!(r.assessment_rate_percent, Some(4.0));
        let assessed = r.assessed_monthly_instalment.unwrap();
        assert!(
            assessed > 2_761.0,
            "assessed {assessed} should exceed the quoted payment"
        );

        // CPF still services the real instalment, not the stressed one.
        assert_eq!(r.cpf_used, Some(1_200.0));
        assert_eq!(r.cash_required, Some(2_761.0 - 1_200.0));
    }

    #[test]
    fn msr_applies_only_to_hdb_and_is_the_tighter_ceiling() {
        let r = singapore_from_params(SingaporeParams {
            monthly_payment: Some(3_200.0),
            is_hdb_or_ec: true,
            ..params()
        });
        // 32% clears TDSR's 55% but breaches MSR's 30%.
        assert!(!r.tdsr_exceeded);
        assert!(r.msr_exceeded);
        assert!(r.warnings.iter().any(|w| w.contains("MSR")));
    }

    #[test]
    fn splits_the_payment_between_cpf_and_cash() {
        let r = singapore_from_params(params());
        assert_eq!(r.cpf_used, Some(1_200.0));
        assert_eq!(r.cash_required, Some(1_800.0));
    }

    #[test]
    fn stamp_duty_still_computes_without_a_valid_payment() {
        let r = singapore_from_params(SingaporeParams {
            monthly_payment: None,
            ..params()
        });
        assert!(r.tdsr_ratio_percent.is_none());
        assert!(r.cpf_used.is_none());
        // BSD on $1m is the published 24,600 example; citizen's first
        // property attracts no ABSD.
        assert_eq!(r.bsd, 24_600.0);
        assert_eq!(r.absd, 0.0);
    }

    #[test]
    fn foreigners_pay_absd_on_a_first_property() {
        let r = singapore_from_params(SingaporeParams {
            residency: "Foreigner".into(),
            ..params()
        });
        assert_eq!(r.absd, 600_000.0);
        assert_eq!(r.upfront_total, 624_600.0);
    }

    #[test]
    fn total_cash_required_adds_the_down_payment_to_stamp_duty() {
        let r = singapore_from_params(params());
        // 1,000,000 price - 800,000 loan = 200,000 down, plus 24,600 BSD.
        assert_eq!(r.down_payment, 200_000.0);
        assert_eq!(r.total_cash_required, 224_600.0);
    }

    #[test]
    fn warns_when_an_hdb_loan_is_chosen_for_private_property() {
        let r = singapore_from_params(SingaporeParams {
            loan_type: "HDB Loan".into(),
            is_hdb_or_ec: false,
            ..params()
        });
        assert!(r.loan_type_warning.is_some());
    }

    #[test]
    fn no_loan_type_warning_for_an_hdb_flat() {
        let r = singapore_from_params(SingaporeParams {
            loan_type: "HDB Loan".into(),
            is_hdb_or_ec: true,
            ..params()
        });
        assert!(r.loan_type_warning.is_none());
    }

    #[test]
    fn a_non_finite_payment_is_treated_as_no_payment() {
        let r = singapore_from_params(SingaporeParams {
            monthly_payment: Some(f64::NAN),
            ..params()
        });
        assert!(r.tdsr_ratio_percent.is_none());
        assert!(r.cpf_used.is_none());
    }
}
