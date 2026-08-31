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
    /// The rate as a shape, not a figure. A flat quote is
    /// `{ kind: "fixed", rate_percent: 6.5 }`; a Singapore package is
    /// `reverting`, and carries its own step-up so no caller can quote the
    /// promotional rate as if it lasted the whole term.
    pub rate: RateTypeDto,
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
    /// What the instalment becomes after the lock-in, for a package that
    /// steps up. `null` for a rate that holds for the whole term.
    pub payment_after_reversion: Option<f64>,
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
    /// The rate being refinanced *into*, as a shape — in Singapore that is
    /// a package that steps up, not a single figure.
    pub new_rate: RateTypeDto,
    pub new_term_years: f64,
    pub closing_costs: f64,
    pub frequency: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RefinanceResultDto {
    pub current_payment: Option<f64>,
    pub new_payment: Option<f64>,
    /// What the new instalment becomes after the lock-in. `null` when the
    /// new rate holds for its whole term.
    pub new_payment_after_reversion: Option<f64>,
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
    /// A package whose spread steps up after a lock-in: how every Singapore
    /// bank quotes a home loan.
    Reverting {
        base_rate_percent: f64,
        /// Whether that base is a published benchmark that moves -- 3M SORA
        /// for a SGD package -- rather than a figure agreed for the term.
        ///
        /// Defaults to `true` when absent, which covers a scenario saved
        /// before the field existed. Those were all built from the SGD
        /// presets, every one of which is quoted over SORA, so the default
        /// restores the truth about them rather than guessing. It is also
        /// the safe direction: over-disclosing an assumption costs the
        /// reader a sentence, and under-disclosing one costs them the
        /// difference between a projection and a quotation.
        #[serde(default = "base_floats_by_default")]
        base_floats: bool,
        initial_spread_percent: f64,
        initial_years: f64,
        thereafter_spread_percent: f64,
    },
}

fn base_floats_by_default() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct DescribeRateParams {
    pub rate_type: RateTypeDto,
    #[serde(default)]
    pub term_years: f64,
    /// The published benchmark the row was seeded with. A row built from
    /// scratch has none, and keeps whatever name the user gave it.
    #[serde(default)]
    pub index: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RatePresetDto {
    /// English rendering, kept as the entry's identity and as a fallback for
    /// a UI with no catalog entry.
    pub label: String,
    /// The same name as a code plus its values, so the UI can compose it in
    /// the reader's language -- the convention already used for errors.
    pub label_message: Message,
    /// The benchmark this preset names, so a row seeded from it can be
    /// renamed from its own figures as they are edited.
    pub index: Option<String>,
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
    /// The rate charged at the start. For a reverting package this is the
    /// promotional one -- see `thereafter_rate_percent`.
    pub effective_rate_percent: f64,
    /// The rate after the lock-in, and the instalment it produces. Both
    /// `null` for a rate that holds for the term.
    pub thereafter_rate_percent: Option<f64>,
    pub payment_after_reversion: Option<f64>,
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
    /// Omit to stamp the current time; pass a value to preserve one read
    /// back from an import, so a restored scenario doesn't claim to have
    /// been saved at the moment it was merely re-uploaded.
    pub created_at: Option<i64>,
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
    /// estimate, so the formula stays on this side of the boundary. Taken
    /// as a shape rather than a figure so that resolving a floating quote
    /// to the rate it actually charges stays here too.
    pub rate: RateTypeDto,
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
    /// The borrower's own package. TDSR/MSR are assessed at the higher of
    /// MAS's 4% floor and the rate the loan *ends* on, so the terms have to
    /// cross the boundary — a pre-computed `monthly_payment` alone can't be
    /// repriced, and a single rate would hide the step-up that Notice 645
    /// para 6(b) is asking about.
    pub rate: RateTypeDto,
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
    /// The rate the loan opens at, e.g. `1.42`. What the borrower pays first.
    pub annual_rate_percent: f64,
    /// The rate after the lock-in, for a package that steps up. Omit for a
    /// rate that holds for the term.
    ///
    /// Servicing is assessed at the higher of this and MAS's 4% floor --
    /// "thereafter" is the Notice's word, and assessing on the promotional
    /// rate would pass borrowers a bank would decline.
    #[serde(default)]
    pub thereafter_annual_rate_percent: Option<f64>,
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

/// A stretch of the loan over which the instalment does not change.
///
/// The year-band idea the CFPB's Loan Estimate uses for an adjustable loan
/// ("Years 1-5 | Years 6-8 | ..."), because a package that steps up cannot
/// be honestly described by one headline figure.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct PaymentBandDto {
    pub from_year: f64,
    pub to_year: f64,
    pub annual_rate_percent: f64,
    pub payment: f64,
}

/// One line of the rate-change illustration MAS requires a fact sheet to
/// carry: what a rise in the benchmark does to the instalment.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct RateRiseRowDto {
    /// Percentage points added to the rate that lasts, e.g. `1.0` for +1%.
    pub increase_percent: f64,
    pub annual_rate_percent: f64,
    pub payment: f64,
    pub payment_increase: f64,
}

/// One authority a report cites, as a stable code plus somewhere to check
/// it. The code is the catalog key; the URL is the authority's own.
#[derive(Debug, Clone, Serialize)]
pub struct ReferenceDto {
    pub code: String,
    pub url: String,
}

/// A loan illustration, plus the loan terms it was built from.
#[derive(Debug, Clone, Deserialize)]
pub struct ReportParams {
    pub loan: LoanParams,
    /// `"US" | "SG"`. Decides which authorities the document cites.
    pub region: Option<String>,
}

/// One scheduled payment, as the document prints it.
///
/// Leaner than `AmortizationRowDto`: a report is built with no extra
/// payment, so that type's `extra_payment` would be a column of zeros on
/// every row of a 300-row table.
#[derive(Debug, Clone, Serialize)]
pub struct ReportScheduleRowDto {
    pub period: u32,
    /// Which year of the loan the payment falls in. Derived in
    /// `mortgage_calc::report` from the cadence, not by dividing here.
    pub year: u32,
    pub paid: f64,
    pub principal: f64,
    pub interest: f64,
    pub remaining_balance: f64,
}

/// The figures a client-facing loan illustration states.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReportResult {
    pub principal: Option<f64>,
    pub term_years: Option<f64>,
    pub initial_rate_percent: Option<f64>,
    pub initial_payment: Option<f64>,
    /// The rate the loan spends most of its life at. Equal to
    /// `initial_rate_percent` unless it steps up.
    pub final_rate_percent: Option<f64>,
    pub payment_after_reversion: Option<f64>,
    pub lock_in_years: Option<f64>,
    pub total_paid: Option<f64>,
    pub total_interest: Option<f64>,
    pub interest_share_percent: Option<f64>,
    /// `"monthly" | "biweekly" | "weekly"`. The document has to say which
    /// cadence its instalments are on -- it used to print "Monthly
    /// instalment" over a fortnightly figure, because the cadence never
    /// crossed the boundary at all.
    pub frequency: Option<String>,
    pub bands: Vec<PaymentBandDto>,
    /// The benchmark every figure here was computed at, when the quote
    /// rests on one that moves. `null` when the rates are contractual.
    pub floating_base_percent: Option<f64>,
    /// The same fact as a sentence the document prints, in the reader's
    /// language. `null` alongside a `null` base.
    pub rate_note: Option<Message>,
    pub rate_rise: Vec<RateRiseRowDto>,
    /// One row per scheduled payment.
    pub schedule: Vec<ReportScheduleRowDto>,
    /// The same payments rolled up by year. Both are sent; which one the
    /// document prints is the reader's choice, and a toggle that had to
    /// cross the boundary to answer would stall on every flip.
    pub yearly: Vec<AmortizationYearDto>,
    /// Who set the rules the figures follow. The document cites these
    /// rather than asserting the numbers on its own authority.
    pub references: Vec<ReferenceDto>,
    pub error: Option<String>,
    /// The same failure as `error`, but as a code plus its values so a
    /// translated UI can compose the sentence itself.
    pub error_message: Option<Message>,
}
