//! Serde-derived input/output types crossing the JS/Rust boundary.
//!
//! Every result DTO carries an `error: Option<String>` field instead of
//! using `Result` — `wasm-bindgen` return values need a single concrete
//! shape, so failures are reported in-band rather than as a thrown
//! exception, and the frontend just checks `result.error`.

use serde::{Deserialize, Serialize};

/// Shared loan terms: principal, rate, term, and payment cadence.
#[derive(Debug, Clone, Deserialize)]
pub struct LoanParams {
    pub principal: f64,
    /// Annual interest rate as a percentage, e.g. `6.5` for 6.5%.
    pub annual_rate_percent: f64,
    pub term_years: f64,
    /// `"monthly" | "biweekly" | "weekly"`, defaults to monthly.
    pub frequency: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PaymentSummaryResult {
    pub payment: Option<f64>,
    pub total_periods: Option<u32>,
    pub total_paid: Option<f64>,
    pub total_interest: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AmortizationParams {
    pub loan: LoanParams,
    /// Extra amount applied to principal every period, in addition to the
    /// regular payment. `0` for a plain schedule.
    pub extra_payment: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AmortizationRowDto {
    pub period: u32,
    pub payment: f64,
    pub extra_payment: f64,
    pub principal_portion: f64,
    pub interest_portion: f64,
    pub remaining_balance: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AmortizationResult {
    pub rows: Vec<AmortizationRowDto>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ExtraPaymentImpactResult {
    pub baseline_periods: Option<u32>,
    pub payoff_periods: Option<u32>,
    pub periods_saved: Option<u32>,
    pub baseline_total_interest: Option<f64>,
    pub total_interest_with_extra: Option<f64>,
    pub interest_saved: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AffordabilityParams {
    pub gross_monthly_income: f64,
    pub monthly_debts: f64,
    pub down_payment: f64,
    pub annual_rate_percent: f64,
    pub term_years: f64,
    /// Back-end debt-to-income ceiling as a percentage, e.g. `36` for 36%.
    pub max_dti_percent: f64,
    /// Property tax as an annual percentage of home price, e.g. `1.2`.
    pub annual_property_tax_rate_percent: f64,
    pub annual_insurance: f64,
    pub monthly_hoa: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AffordabilityResultDto {
    pub max_monthly_housing_payment: Option<f64>,
    pub max_principal_and_interest: Option<f64>,
    pub max_loan_amount: Option<f64>,
    pub max_home_price: Option<f64>,
    pub front_end_dti_percent: Option<f64>,
    pub back_end_dti_percent: Option<f64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefinanceParams {
    pub current_balance: f64,
    pub current_annual_rate_percent: f64,
    pub remaining_periods: u32,
    pub new_annual_rate_percent: f64,
    pub new_term_years: f64,
    pub closing_costs: f64,
    pub frequency: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RefinanceResultDto {
    pub current_payment: Option<f64>,
    pub new_payment: Option<f64>,
    pub payment_savings: Option<f64>,
    pub break_even_periods: Option<u32>,
    pub remaining_interest_on_current_loan: Option<f64>,
    pub total_interest_on_new_loan: Option<f64>,
    pub lifetime_savings: Option<f64>,
    pub error: Option<String>,
}
