//! Serde-derived input/output types crossing the JS/Rust boundary.
//!
//! Every result DTO carries an `error: Option<String>` field instead of
//! using `Result` — `wasm-bindgen` return values need a single concrete
//! shape, so failures are reported in-band rather than as a thrown
//! exception, and the frontend just checks `result.error`.

use serde::{Deserialize, Serialize};

use crate::message::Message;

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

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonVerdictDto {
    pub cheapest_payment: usize,
    pub cheapest_interest: usize,
    pub cheapest_total_paid: usize,
    /// `"outright"` when one row wins on both measures, `"split"` when the
    /// cheaper loan costs more each period. A stable code, not a sentence:
    /// the UI composes the wording in the reader's language.
    pub kind: String,
    pub cheaper: usize,
    pub lighter: usize,
    pub payment_delta: f64,
    pub interest_delta: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PaymentSummaryResult {
    pub payment: Option<f64>,
    pub total_periods: Option<u32>,
    pub total_paid: Option<f64>,
    pub total_interest: Option<f64>,
    /// Interest as a percentage of everything paid. `null` when nothing is
    /// paid -- a share of zero is undefined, not zero percent.
    pub interest_share_percent: Option<f64>,
    pub error: Option<String>,
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
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

#[derive(Debug, Clone, Serialize)]
pub struct AmortizationYearDto {
    pub year: u32,
    pub paid: f64,
    pub principal: f64,
    pub interest: f64,
    pub remaining_balance: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AmortizationResult {
    pub rows: Vec<AmortizationRowDto>,
    /// The same schedule grouped into years. Returned alongside the rows
    /// rather than from a second binding: the yearly figures are shown
    /// beside the periods they sum, so they must come from one calculation.
    pub yearly: Vec<AmortizationYearDto>,
    pub error: Option<String>,
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
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
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
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
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
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
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
}

/// A rate, expressed either as a flat fixed percentage or as a floating
/// base index + spread (e.g. "SOFR + 2.5%"). Used both as an input (a
/// comparison entry's rate) and an output (a preset's rate) — same shape
/// either direction, so the frontend can pass a preset straight into a
/// comparison entry without reshaping it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RateTypeDto {
    Fixed {
        rate_percent: f64,
    },
    Floating {
        base_rate_percent: f64,
        spread_percent: f64,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct RatePresetDto {
    pub label: String,
    pub rate_type: RateTypeDto,
    pub term_years: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComparisonEntryParams {
    pub label: String,
    pub rate_type: RateTypeDto,
    pub term_years: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComparisonParams {
    pub principal: f64,
    pub frequency: Option<String>,
    pub entries: Vec<ComparisonEntryParams>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComparisonRowDto {
    pub label: String,
    pub effective_rate_percent: f64,
    pub term_years: f64,
    pub payment: f64,
    pub total_periods: u32,
    pub total_paid: f64,
    pub total_interest: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ComparisonResult {
    pub rows: Vec<ComparisonRowDto>,
    /// Which row wins on each measure, and what the choice costs. `null` for
    /// fewer than two rows -- there is no verdict to give on a single line.
    pub verdict: Option<ComparisonVerdictDto>,
    pub error: Option<String>,
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
}

/// A saved scenario's inputs are stored as an opaque JSON string (see
/// [`mortgage_ports::Scenario`]) — this crate doesn't need to know each
/// calculator's exact input shape, only pass it through.
#[derive(Debug, Clone, Deserialize)]
pub struct SaveScenarioParams {
    /// `"payment" | "amortization" | "affordability" | "refinance" | "comparison"`
    pub calculator: String,
    pub name: String,
    pub inputs_json: String,
    /// Omit to create a new scenario; pass an existing id to overwrite it.
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioDto {
    pub id: String,
    pub calculator: String,
    pub name: String,
    pub created_at: i64,
    pub inputs_json: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SaveScenarioResult {
    pub id: Option<String>,
    pub error: Option<String>,
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ScenarioListResult {
    pub scenarios: Vec<ScenarioDto>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ScenarioResult {
    pub scenario: Option<ScenarioDto>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DeleteScenarioResult {
    pub success: bool,
    pub error: Option<String>,
}

/// Inputs for the United States panel.
///
/// `monthly_pi` is the principal-and-interest payment from whichever
/// calculator the panel is attached to, and is `None` when those inputs
/// don't form a valid loan. Property tax and PMI are priced off the home
/// price and loan amount alone, so they still compute without it.
#[derive(Debug, Clone, Deserialize)]
pub struct UnitedStatesParams {
    pub monthly_pi: Option<f64>,
    pub principal: f64,
    pub home_price: f64,
    /// Used to derive the first period's interest for the deduction
    /// estimate, so the formula stays on this side of the boundary.
    pub annual_rate_percent: f64,
    /// Five-digit ZIP; only the first three digits are used to resolve a
    /// state.
    pub zip: String,
    /// Annual PMI rate as a percentage. `None` uses the crate default.
    pub pmi_rate_percent: Option<f64>,
    pub use_tax_deduction: bool,
    pub marginal_tax_rate_percent: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UnitedStatesResult {
    /// `"Conforming"` or `"Jumbo"`.
    pub loan_type: String,
    /// `None` when the ZIP doesn't resolve to a known state.
    pub property_tax_rate_percent: Option<f64>,
    pub monthly_property_tax: f64,
    pub down_payment: f64,
    pub down_payment_percent: f64,
    pub pmi_required: bool,
    pub monthly_pmi: f64,
    /// Principal, interest, taxes and insurance. `None` without a payment.
    pub monthly_piti: Option<f64>,
    /// Both `None` unless the deduction estimate is switched on.
    pub monthly_tax_savings: Option<f64>,
    pub net_monthly_cost: Option<f64>,
    pub error: Option<String>,
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
}

/// Inputs for the Singapore regulatory panel.
///
/// `monthly_payment` comes from whichever calculator the panel is attached
/// to. It's `None` when those inputs don't currently form a valid loan —
/// the TDSR/MSR and CPF figures then have nothing to work from, but the
/// BSD/ABSD stamp duties are priced off `home_price` alone and still
/// compute.
#[derive(Debug, Clone, Deserialize)]
pub struct SingaporeParams {
    pub monthly_payment: Option<f64>,
    /// The loan amount, used with `home_price` to derive the down payment.
    pub principal: f64,
    /// The borrower's own rate, e.g. `4.5` for 4.5%. TDSR/MSR are assessed
    /// at the higher of this and MAS's 4% floor, so the terms have to cross
    /// the boundary — a pre-computed `monthly_payment` alone can't be
    /// repriced.
    pub annual_rate_percent: f64,
    pub term_years: f64,
    pub home_price: f64,
    /// Fixed salary, counted in full.
    pub fixed_monthly_income: f64,
    /// Commission, bonus or allowance, counted at 70% per Notice 645 para 17.
    pub variable_monthly_income: f64,
    pub other_monthly_debts: f64,
    pub cpf_oa_available: f64,
    /// `"Citizen" | "PR" | "Foreigner"`, defaults to Citizen.
    pub residency: String,
    /// `"1st" | "2nd" | "3rd+"`, defaults to 1st.
    pub property_count: String,
    /// HDB flats and Executive Condominiums are subject to MSR on top of
    /// TDSR, and are the only properties eligible for an HDB loan.
    pub is_hdb_or_ec: bool,
    /// `"HDB Loan" | "Bank Loan"`.
    pub loan_type: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SingaporeResult {
    /// TDSR as a percentage, e.g. `48.2`. `None` when no valid payment or
    /// income was supplied.
    pub tdsr_ratio_percent: Option<f64>,
    pub tdsr_exceeded: bool,
    pub tdsr_near_limit: bool,
    /// `None` when the property isn't an HDB flat/EC, since MSR doesn't apply.
    pub msr_ratio_percent: Option<f64>,
    pub msr_exceeded: bool,
    pub msr_near_limit: bool,
    /// The rate the ratios were assessed at, as a percentage — MAS's 4%
    /// floor unless the borrower's own rate is higher.
    pub assessment_rate_percent: Option<f64>,
    /// The monthly instalment the ratios were computed from. Above the
    /// borrower's actual payment whenever they're quoted under the floor;
    /// the UI shows both so the difference is explained rather than
    /// mysterious.
    pub assessed_monthly_instalment: Option<f64>,
    /// Income after the variable-income haircut — the ratios' denominator.
    pub assessed_monthly_income: Option<f64>,
    pub cpf_used: Option<f64>,
    pub cash_required: Option<f64>,
    pub bsd: f64,
    pub absd: f64,
    pub upfront_total: f64,
    pub down_payment: f64,
    pub total_cash_required: f64,
    /// Set when an HDB loan is selected for a property that can't have one.
    pub loan_type_warning: Option<String>,
    /// The same warning as a translation code.
    pub loan_type_warning_code: Option<String>,
    /// Human-readable breaches of the MAS ceilings, in display order.
    pub warnings: Vec<String>,
    /// The same breaches as translation codes, in the same order.
    pub warning_codes: Vec<String>,
    pub error: Option<String>,
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
}

/// Inputs for the Singapore affordability model.
///
/// Distinct from [`AffordabilityParams`], which is the US DTI model — the two
/// regimes share no rules worth abstracting over, so they stay apart rather
/// than growing a union type with half its fields unused on either side.
#[derive(Debug, Clone, Deserialize)]
pub struct SgAffordabilityParams {
    /// Fixed salary, counted in full.
    pub fixed_monthly_income: f64,
    /// Commission, bonus or allowance, counted at 70% per Notice 645 para 17.
    pub variable_monthly_income: f64,
    pub other_monthly_debts: f64,
    /// Cash on hand. Separate from CPF because the cash floor and both stamp
    /// duties cannot be met from CPF.
    pub cash_available: f64,
    /// CPF OA balance usable for the deposit above the cash floor.
    pub cpf_oa_available: f64,
    /// The borrower's own rate, e.g. `4.5`. Servicing is assessed at the
    /// higher of this and MAS's 4% floor.
    pub annual_rate_percent: f64,
    pub term_years: f64,
    /// Optional: without it only the tenure half of the extended-tenure test
    /// can be applied.
    pub borrower_age: Option<f64>,
    pub is_hdb_or_ec: bool,
    /// `"Citizen" | "PR" | "Foreigner"`.
    pub residency: String,
    /// `"1st" | "2nd" | "3rd+"` — properties held after this purchase.
    pub property_count: String,
    /// Housing loans already outstanding, which sets the LTV row.
    pub outstanding_housing_loans: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SgAffordabilityResultDto {
    pub max_price: Option<f64>,
    pub max_loan: Option<f64>,
    /// `"tdsr" | "msr" | "ltv"` — which side of the purchase is tight.
    pub binding_constraint: Option<String>,
    pub max_monthly_instalment: Option<f64>,
    pub assessment_rate_percent: Option<f64>,
    pub ltv_percent: Option<f64>,
    pub extended_tenure: bool,
    pub deposit: Option<f64>,
    pub min_cash_required: Option<f64>,
    pub bsd: Option<f64>,
    pub absd: Option<f64>,
    /// Cash the buyer must actually produce at completion.
    pub cash_required: Option<f64>,
    /// CPF OA applied to the deposit.
    pub cpf_used: Option<f64>,
    pub cash_and_cpf_at_completion: Option<f64>,
    /// Income after the variable-income haircut.
    pub assessed_monthly_income: Option<f64>,
    pub error: Option<String>,
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
}
