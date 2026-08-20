use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::str::FromStr;

use mortgage_calc::affordability::AffordabilityInput;
use mortgage_calc::comparison::{ComparisonEntry, ComparisonInput};
use mortgage_calc::refinance::RefinanceInput;
use mortgage_calc::{singapore, united_states, Loan, PaymentFrequency, RateType};
use mortgage_core::{round_currency, Region};
use mortgage_ext_redb::RedbScenarioStore;
use mortgage_ports::{CalculatorKind, Scenario, ScenarioStore};
use rust_decimal::{Decimal, RoundingStrategy};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use slint::{ModelRc, VecModel, Weak};

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

mod storage;

slint::include_modules!();

type StoreCell = Rc<RefCell<Option<Rc<RedbScenarioStore>>>>;

fn default_true() -> bool {
    true
}

// `#[serde(default)]` on every region/SG/US field keeps this
// backward-compatible with scenarios saved before those panels existed —
// loading an old save just leaves them at their zero-ish defaults instead
// of failing to deserialize.
#[derive(Serialize, Deserialize)]
struct PaymentInputs {
    principal: String,
    rate_percent: String,
    term_years: String,
    #[serde(default)]
    region: String,
    #[serde(default)]
    sg_gross_income: String,
    #[serde(default)]
    sg_other_debts: String,
    #[serde(default = "default_true")]
    sg_is_hdb: bool,
    #[serde(default)]
    sg_loan_type: String,
    #[serde(default)]
    sg_use_cpf: bool,
    #[serde(default)]
    sg_cpf_oa_available: String,
    #[serde(default)]
    sg_home_price: String,
    #[serde(default)]
    sg_residency: String,
    #[serde(default)]
    sg_property_count: String,
    #[serde(default)]
    us_home_price: String,
    #[serde(default)]
    us_zip: String,
    #[serde(default)]
    us_pmi_rate_percent: String,
    #[serde(default)]
    us_use_tax_deduction: bool,
    #[serde(default)]
    us_marginal_tax_rate_percent: String,
}

#[derive(Serialize, Deserialize)]
struct AmortInputs {
    principal: String,
    rate_percent: String,
    term_years: String,
    extra_payment: String,
}

#[derive(Serialize, Deserialize)]
struct AffInputs {
    income: String,
    debts: String,
    down_payment: String,
    rate_percent: String,
    term_years: String,
    max_dti_percent: String,
    tax_rate_percent: String,
    insurance: String,
    hoa: String,
}

#[derive(Serialize, Deserialize)]
struct RefiInputs {
    current_balance: String,
    current_rate_percent: String,
    remaining_periods: String,
    new_rate_percent: String,
    new_term_years: String,
    closing_costs: String,
}

#[derive(Serialize, Deserialize)]
struct CompareInputs {
    principal: String,
    a_label: String,
    a_floating: bool,
    a_rate_percent: String,
    a_base_rate_percent: String,
    a_spread_percent: String,
    a_term_years: String,
    b_label: String,
    b_floating: bool,
    b_rate_percent: String,
    b_base_rate_percent: String,
    b_spread_percent: String,
    b_term_years: String,
}

fn rows_from_scenarios(list: Vec<Scenario>) -> ModelRc<ScenarioRow> {
    let rows: Vec<ScenarioRow> = list
        .into_iter()
        .map(|s| ScenarioRow {
            id: s.id.into(),
            name: s.name.into(),
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}

async fn refresh_scenarios(ui: &AppWindow, store: &RedbScenarioStore, kind: CalculatorKind) {
    let list = store.list(Some(kind)).await.unwrap_or_default();
    let rows = rows_from_scenarios(list);
    match kind {
        CalculatorKind::Payment => ui.set_payment_scenarios(rows),
        CalculatorKind::Amortization => ui.set_amort_scenarios(rows),
        CalculatorKind::Affordability => ui.set_aff_scenarios(rows),
        CalculatorKind::Refinance => ui.set_refi_scenarios(rows),
        CalculatorKind::Comparison => ui.set_cmp_scenarios(rows),
    }
}

async fn refresh_all_scenarios(ui: &AppWindow, store: &RedbScenarioStore) {
    refresh_scenarios(ui, store, CalculatorKind::Payment).await;
    refresh_scenarios(ui, store, CalculatorKind::Amortization).await;
    refresh_scenarios(ui, store, CalculatorKind::Affordability).await;
    refresh_scenarios(ui, store, CalculatorKind::Refinance).await;
    refresh_scenarios(ui, store, CalculatorKind::Comparison).await;
}

fn do_save(
    ui: Weak<AppWindow>,
    store_cell: StoreCell,
    kind: CalculatorKind,
    name: String,
    inputs_json: String,
    set_error: fn(&AppWindow, slint::SharedString),
) {
    let store = store_cell.borrow().clone();
    let Some(store) = store else {
        if let Some(ui) = ui.upgrade() {
            set_error(&ui, "Storage not ready yet.".into());
        }
        return;
    };
    slint::spawn_local(async move {
        let scenario = Scenario {
            id: storage::new_scenario_id(),
            calculator: kind,
            name,
            created_at: storage::now_millis(),
            inputs_json,
        };
        let result = store.save(scenario).await;
        if let Some(ui) = ui.upgrade() {
            match result {
                Ok(()) => {
                    set_error(&ui, "".into());
                    refresh_scenarios(&ui, &store, kind).await;
                }
                Err(e) => set_error(&ui, e.to_string().into()),
            }
        }
    })
    .ok();
}

fn do_load(
    ui: Weak<AppWindow>,
    store_cell: StoreCell,
    id: String,
    apply: impl FnOnce(&AppWindow, &str) + 'static,
) {
    let store = store_cell.borrow().clone();
    let Some(store) = store else { return };
    slint::spawn_local(async move {
        if let Ok(scenario) = store.load(&id).await {
            if let Some(ui) = ui.upgrade() {
                apply(&ui, &scenario.inputs_json);
            }
        }
    })
    .ok();
}

fn do_delete(ui: Weak<AppWindow>, store_cell: StoreCell, kind: CalculatorKind, id: String) {
    let store = store_cell.borrow().clone();
    let Some(store) = store else { return };
    slint::spawn_local(async move {
        let _ = store.delete(&id).await;
        if let Some(ui) = ui.upgrade() {
            refresh_scenarios(&ui, &store, kind).await;
        }
    })
    .ok();
}

fn build_loan(principal: &str, rate_percent: &str, term_years: &str) -> Option<Loan> {
    let principal_d = Decimal::from_str(principal).ok()?;
    let rate_d = Decimal::from_str(rate_percent).ok()?;
    let term_d = Decimal::from_str(term_years).ok()?;

    Loan::builder()
        .principal(principal_d)
        .annual_rate(rate_d / Decimal::from(100))
        .term_years(term_d)
        .frequency(PaymentFrequency::Monthly)
        .build()
        .ok()
}

fn parse_rate_value(s: &str) -> f32 {
    s.parse::<f32>().unwrap_or(0.0)
}

fn format_rate_value(v: f32) -> String {
    let s = format!("{:.2}", v);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() { "0".to_string() } else { s.to_string() }
}

/// Formats `ratio` (e.g. `0.0909` for 9.09%) as a percentage string with
/// `dp` decimal places, rounded rather than truncated — `Decimal`'s own
/// `{:.N}` formatting truncates, which silently understates borderline
/// figures (9.0909...% would print as "9.0%" instead of "9.1%").
fn format_percent(ratio: Decimal, dp: u32) -> String {
    (ratio * dec!(100))
        .round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero)
        .to_string()
}

fn boom_text(periods_saved: u32, interest_saved: Decimal) -> String {
    if periods_saved == 0 {
        return format!("Boom! You just chopped ${} in interest off your mortgage.", interest_saved);
    }
    let years = periods_saved / 12;
    let months = periods_saved % 12;
    let time_phrase = match (years, months) {
        (0, m) => format!("{} month{}", m, if m == 1 { "" } else { "s" }),
        (y, 0) => format!("{} year{}", y, if y == 1 { "" } else { "s" }),
        (y, m) => format!(
            "{} year{} {} month{}",
            y,
            if y == 1 { "" } else { "s" },
            m,
            if m == 1 { "" } else { "s" }
        ),
    };
    format!(
        "Boom! You just chopped {} and ${} in interest off your mortgage.",
        time_phrase, interest_saved
    )
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

fn blank_sg_limits(ui: &AppWindow) {
    ui.set_sg_tdsr_label("".into());
    ui.set_sg_tdsr_exceeded(false);
    ui.set_sg_tdsr_near(false);
    ui.set_sg_msr_label("".into());
    ui.set_sg_msr_exceeded(false);
    ui.set_sg_msr_near(false);
    ui.set_sg_limit_warning("".into());
}

/// Recomputes the Payment tab's Singapore panel (TDSR/MSR limits, CPF/cash
/// split, and BSD/ABSD upfront costs). `monthly_payment` is the Payment
/// tab's own result, if the loan inputs were valid — the limits and CPF
/// split depend on it, but upfront costs are priced off a separate home
/// price input and don't. `principal` feeds the down payment (home price
/// minus loan amount) that upfront costs get added to for the total cash
/// figure.
fn recompute_payment_sg(ui: &AppWindow, principal: Decimal, monthly_payment: Option<Decimal>) {
    let wants_hdb_loan = ui.get_sg_loan_type() == "HDB Loan";
    if wants_hdb_loan && !singapore::hdb_loan_eligible(ui.get_sg_is_hdb()) {
        ui.set_sg_loan_type_warning("HDB loans are only available for HDB flats/ECs bought from HDB.".into());
    } else {
        ui.set_sg_loan_type_warning("".into());
    }

    match monthly_payment {
        Some(payment) => {
            let income = Decimal::from_str(ui.get_sg_gross_income().as_str()).unwrap_or_default();
            let other_debts = Decimal::from_str(ui.get_sg_other_debts().as_str()).unwrap_or_default();

            match singapore::check_tdsr_msr(payment, other_debts, income, ui.get_sg_is_hdb()) {
                Ok(check) => {
                    ui.set_sg_tdsr_label(format!("{}%", format_percent(check.tdsr.ratio, 1)).into());
                    ui.set_sg_tdsr_exceeded(check.tdsr.exceeded);
                    ui.set_sg_tdsr_near(check.tdsr.near_limit);

                    let mut warnings = Vec::new();
                    if check.tdsr.exceeded {
                        warnings.push("Exceeds MAS TDSR limit (55%).");
                    }
                    match check.msr {
                        Some(msr) => {
                            ui.set_sg_msr_label(format!("{}%", format_percent(msr.ratio, 1)).into());
                            ui.set_sg_msr_exceeded(msr.exceeded);
                            ui.set_sg_msr_near(msr.near_limit);
                            if msr.exceeded {
                                warnings.push("Exceeds MAS MSR limit (30%) for HDB/EC.");
                            }
                        }
                        None => {
                            ui.set_sg_msr_label("".into());
                            ui.set_sg_msr_exceeded(false);
                            ui.set_sg_msr_near(false);
                        }
                    }
                    ui.set_sg_limit_warning(warnings.join(" ").into());
                }
                Err(_) => blank_sg_limits(ui),
            }

            let cpf_oa = Decimal::from_str(ui.get_sg_cpf_oa_available().as_str()).unwrap_or_default();
            let split = singapore::cpf_cash_split(payment, cpf_oa);
            ui.set_sg_cpf_used_label(format!("${}", split.cpf_used).into());
            ui.set_sg_cash_required_label(format!("${}", split.cash_required).into());
        }
        None => {
            blank_sg_limits(ui);
            ui.set_sg_cpf_used_label("".into());
            ui.set_sg_cash_required_label("".into());
        }
    }

    let price = Decimal::from_str(ui.get_sg_home_price().as_str()).unwrap_or_default().max(Decimal::ZERO);
    let residency = parse_residency(ui.get_sg_residency().as_str());
    let count = parse_property_count(ui.get_sg_property_count().as_str());
    let costs = singapore::upfront_costs(price, residency, count);
    ui.set_sg_bsd_label(format!("${}", costs.bsd).into());
    ui.set_sg_absd_label(format!("${}", costs.absd).into());
    ui.set_sg_upfront_total_label(format!("${}", costs.total).into());

    let down_payment = round_currency((price - principal).max(Decimal::ZERO));
    ui.set_sg_down_payment_label(format!("${}", down_payment).into());
    ui.set_sg_total_cash_required_label(format!("${}", round_currency(down_payment + costs.total)).into());
}

/// Recomputes the Payment tab's United States panel (ZIP-based property
/// tax, PMI trigger, and the tax-deductibility "net cost" simulator).
/// `principal`/`monthly_pi`/`first_period_interest` come from the Payment
/// tab's own loan, if the inputs were valid — home price (independent
/// input) and PMI/property tax still compute without a valid loan, but
/// PITI and the tax-savings figure need it.
fn recompute_payment_us(
    ui: &AppWindow,
    principal: Decimal,
    monthly_pi: Option<Decimal>,
    first_period_interest: Option<Decimal>,
) {
    let home_price = Decimal::from_str(ui.get_us_home_price().as_str()).unwrap_or_default().max(Decimal::ZERO);

    let loan_type_label = match united_states::classify_loan(principal) {
        united_states::LoanConformance::Conforming => "Conforming",
        united_states::LoanConformance::Jumbo => "Jumbo",
    };
    ui.set_us_loan_type_label(loan_type_label.into());

    let tax_rate = united_states::estimate_property_tax_rate(ui.get_us_zip().as_str());
    let monthly_property_tax = tax_rate
        .map(|rate| round_currency(home_price * rate / dec!(12)))
        .unwrap_or(Decimal::ZERO);

    match tax_rate {
        Some(rate) => ui.set_us_property_tax_rate_label(format!("{}%", format_percent(rate, 2)).into()),
        None => ui.set_us_property_tax_rate_label("Unrecognized ZIP".into()),
    }
    ui.set_us_monthly_property_tax_label(format!("${}", monthly_property_tax).into());

    let down_payment = (home_price - principal).max(Decimal::ZERO);
    let down_payment_percent = if home_price > Decimal::ZERO {
        down_payment / home_price
    } else {
        Decimal::ZERO
    };
    ui.set_us_down_payment_percent_label(format!("{}%", format_percent(down_payment_percent, 1)).into());
    // Keeps the slider in sync no matter what triggered this recompute —
    // dragging it, or editing home price / loan amount directly.
    ui.set_us_down_payment_percent_value(decimal_to_f64(down_payment_percent * dec!(100)) as f32);

    let pmi_required = home_price > Decimal::ZERO && united_states::requires_pmi(down_payment_percent);
    ui.set_us_pmi_required(pmi_required);

    let pmi_rate_percent = Decimal::from_str(ui.get_us_pmi_rate_percent().as_str())
        .unwrap_or(united_states::DEFAULT_PMI_ANNUAL_RATE * dec!(100));
    let monthly_pmi = if pmi_required {
        united_states::monthly_pmi(principal, pmi_rate_percent / dec!(100))
    } else {
        Decimal::ZERO
    };
    ui.set_us_monthly_pmi_label(format!("${}", monthly_pmi).into());

    let monthly_piti = round_currency(monthly_pi.unwrap_or(Decimal::ZERO) + monthly_property_tax + monthly_pmi);
    ui.set_us_monthly_piti_label(format!("${}", monthly_piti).into());

    if ui.get_us_use_tax_deduction() {
        let marginal_rate_percent = Decimal::from_str(ui.get_us_marginal_tax_rate_percent().as_str()).unwrap_or_default();
        let savings = united_states::monthly_tax_savings(
            first_period_interest.unwrap_or(Decimal::ZERO),
            marginal_rate_percent / dec!(100),
        );
        ui.set_us_monthly_tax_savings_label(format!("${}", savings).into());
        ui.set_us_net_monthly_cost_label(format!("${}", round_currency(monthly_piti - savings)).into());
    } else {
        ui.set_us_monthly_tax_savings_label("".into());
        ui.set_us_net_monthly_cost_label("".into());
    }
}

fn recompute_payment(ui: &AppWindow) {
    ui.set_rate_percent_value(parse_rate_value(ui.get_rate_percent().as_str()));
    let loan = build_loan(
        ui.get_principal().as_str(),
        ui.get_rate_percent().as_str(),
        ui.get_term_years().as_str(),
    );

    match loan {
        Some(loan) => {
            let summary = mortgage_calc::payment::summarize(&loan);
            ui.set_has_error(false);
            ui.set_payment_result(format!("${}", summary.payment).into());
            ui.set_total_paid_result(format!("${}", summary.total_paid).into());
            ui.set_total_interest_result(format!("${}", summary.total_interest).into());
            recompute_payment_sg(ui, loan.principal(), Some(summary.payment));
            let first_period_interest = loan.principal() * loan.periodic_rate();
            recompute_payment_us(ui, loan.principal(), Some(summary.payment), Some(first_period_interest));
        }
        None => {
            ui.set_has_error(true);
            recompute_payment_sg(ui, Decimal::ZERO, None);
            recompute_payment_us(ui, Decimal::ZERO, None, None);
        }
    }
}

fn recompute_amortization(ui: &AppWindow) {
    ui.set_amort_rate_percent_value(parse_rate_value(ui.get_amort_rate_percent().as_str()));
    let loan = build_loan(
        ui.get_amort_principal().as_str(),
        ui.get_amort_rate_percent().as_str(),
        ui.get_amort_term_years().as_str(),
    );

    let Some(loan) = loan else {
        ui.set_amort_has_error(true);
        return;
    };

    let summary = mortgage_calc::payment::summarize(&loan);
    ui.set_amort_has_error(false);
    ui.set_amort_payment_result(format!("${}", summary.payment).into());
    ui.set_amort_total_paid_result(format!("${}", summary.total_paid).into());
    ui.set_amort_total_interest_result(format!("${}", summary.total_interest).into());

    let extra = Decimal::from_str(ui.get_amort_extra_payment().as_str())
        .unwrap_or(Decimal::ZERO)
        .max(Decimal::ZERO);

    if extra > Decimal::ZERO {
        if let Ok(impact) = mortgage_calc::amortization::extra_payment_impact(&loan, extra) {
            ui.set_amort_show_impact(true);
            ui.set_amort_periods_saved(impact.periods_saved.to_string().into());
            ui.set_amort_interest_saved(format!("${}", impact.interest_saved).into());
            ui.set_amort_new_payoff(impact.payoff_periods.to_string().into());
            ui.set_amort_boom_text(boom_text(impact.periods_saved, impact.interest_saved).into());
        } else {
            ui.set_amort_show_impact(false);
            ui.set_amort_boom_text("".into());
        }
    } else {
        ui.set_amort_show_impact(false);
        ui.set_amort_boom_text("".into());
    }

    let principal_d = Decimal::from_str(ui.get_amort_principal().as_str()).unwrap_or(Decimal::ZERO);

    match mortgage_calc::amortization::schedule(&loan, extra) {
        Ok(rows) => {
            ui.set_amort_schedule_rows(build_schedule_rows(&rows, ui.get_amort_show_full()));
            let (balance_path, interest_path) = build_timeline_paths(&rows, principal_d);
            ui.set_amort_timeline_balance_path(balance_path.into());
            ui.set_amort_timeline_interest_path(interest_path.into());
            apply_scrub(ui, &rows, principal_d, ui.get_amort_scrub_fraction());
        }
        Err(_) => {
            ui.set_amort_schedule_rows(ModelRc::new(VecModel::from(Vec::<AmortScheduleRow>::new())));
            ui.set_amort_timeline_balance_path("".into());
            ui.set_amort_timeline_interest_path("".into());
            apply_scrub(ui, &[], Decimal::ZERO, 0.0);
        }
    }
}

fn build_schedule_rows(
    rows: &[mortgage_calc::amortization::AmortizationRow],
    show_full: bool,
) -> ModelRc<AmortScheduleRow> {
    if show_full {
        let out: Vec<AmortScheduleRow> = rows
            .iter()
            .map(|r| AmortScheduleRow {
                label: r.period.to_string().into(),
                paid: format!("${}", r.payment).into(),
                principal: format!("${}", r.principal_portion).into(),
                interest: format!("${}", r.interest_portion).into(),
                balance: format!("${}", r.remaining_balance).into(),
            })
            .collect();
        return ModelRc::new(VecModel::from(out));
    }

    let mut out = Vec::new();
    for (i, chunk) in rows.chunks(12).enumerate() {
        let paid: Decimal = chunk.iter().map(|r| r.payment).sum();
        let principal: Decimal = chunk.iter().map(|r| r.principal_portion).sum();
        let interest: Decimal = chunk.iter().map(|r| r.interest_portion).sum();
        let balance = chunk.last().expect("chunks(12) never yields an empty chunk").remaining_balance;
        out.push(AmortScheduleRow {
            label: (i + 1).to_string().into(),
            paid: format!("${}", paid).into(),
            principal: format!("${}", principal).into(),
            interest: format!("${}", interest).into(),
            balance: format!("${}", balance).into(),
        });
    }
    ModelRc::new(VecModel::from(out))
}

fn recompute_affordability(ui: &AppWindow) {
    ui.set_aff_rate_percent_value(parse_rate_value(ui.get_aff_rate_percent().as_str()));
    let result = (|| {
        let input = AffordabilityInput {
            gross_monthly_income: Decimal::from_str(ui.get_aff_income().as_str()).ok()?,
            monthly_debts: Decimal::from_str(ui.get_aff_debts().as_str()).ok()?,
            down_payment: Decimal::from_str(ui.get_aff_down_payment().as_str()).ok()?,
            annual_rate: Decimal::from_str(ui.get_aff_rate_percent().as_str()).ok()? / Decimal::from(100),
            term_years: Decimal::from_str(ui.get_aff_term_years().as_str()).ok()?,
            max_dti: Decimal::from_str(ui.get_aff_max_dti_percent().as_str()).ok()? / Decimal::from(100),
            annual_property_tax_rate: Decimal::from_str(ui.get_aff_tax_rate_percent().as_str()).ok()?
                / Decimal::from(100),
            annual_insurance: Decimal::from_str(ui.get_aff_insurance().as_str()).ok()?,
            monthly_hoa: Decimal::from_str(ui.get_aff_hoa().as_str()).ok()?,
        };
        mortgage_calc::affordability::max_affordable(&input).ok()
    })();

    match result {
        Some(result) => {
            ui.set_aff_has_error(false);
            ui.set_aff_max_home_price(format!("${}", result.max_home_price).into());
            ui.set_aff_max_loan_amount(format!("${}", result.max_loan_amount).into());
            ui.set_aff_max_pi(format!("${}", result.max_principal_and_interest).into());
            ui.set_aff_front_dti(format_percent(result.front_end_dti, 1).into());
            ui.set_aff_back_dti(format_percent(result.back_end_dti, 1).into());
        }
        None => ui.set_aff_has_error(true),
    }
}

fn recompute_refinance(ui: &AppWindow) {
    ui.set_refi_current_rate_percent_value(parse_rate_value(ui.get_refi_current_rate_percent().as_str()));
    ui.set_refi_new_rate_percent_value(parse_rate_value(ui.get_refi_new_rate_percent().as_str()));
    let result = (|| {
        let input = RefinanceInput {
            current_balance: Decimal::from_str(ui.get_refi_current_balance().as_str()).ok()?,
            current_annual_rate: Decimal::from_str(ui.get_refi_current_rate_percent().as_str()).ok()?
                / Decimal::from(100),
            remaining_periods: ui.get_refi_remaining_periods().as_str().parse::<u32>().ok()?,
            new_annual_rate: Decimal::from_str(ui.get_refi_new_rate_percent().as_str()).ok()?
                / Decimal::from(100),
            new_term_years: Decimal::from_str(ui.get_refi_new_term_years().as_str()).ok()?,
            closing_costs: Decimal::from_str(ui.get_refi_closing_costs().as_str()).ok()?,
            frequency: PaymentFrequency::Monthly,
        };
        mortgage_calc::refinance::analyze_refinance(&input).ok()
    })();

    match result {
        Some(result) => {
            ui.set_refi_has_error(false);
            ui.set_refi_current_payment(format!("${}", result.current_payment).into());
            ui.set_refi_new_payment(format!("${}", result.new_payment).into());
            ui.set_refi_payment_savings(format!("${}", result.payment_savings).into());
            ui.set_refi_break_even(
                match result.break_even_periods {
                    Some(periods) => format!("{periods} months"),
                    None => "Never".to_string(),
                }
                .into(),
            );
            ui.set_refi_lifetime_savings(format!("${}", result.lifetime_savings).into());
        }
        None => ui.set_refi_has_error(true),
    }
}

fn build_rate_type(
    floating: bool,
    rate_percent: &str,
    base_rate_percent: &str,
    spread_percent: &str,
) -> Option<RateType> {
    if floating {
        Some(RateType::Floating {
            base_rate: Decimal::from_str(base_rate_percent).ok()? / Decimal::from(100),
            spread: Decimal::from_str(spread_percent).ok()? / Decimal::from(100),
        })
    } else {
        Some(RateType::Fixed {
            rate: Decimal::from_str(rate_percent).ok()? / Decimal::from(100),
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn build_compare_loan(
    principal: Decimal,
    floating: bool,
    rate_percent: &str,
    base_rate_percent: &str,
    spread_percent: &str,
    term_years: &str,
) -> Option<Loan> {
    let rate_type = build_rate_type(floating, rate_percent, base_rate_percent, spread_percent)?;
    Loan::builder()
        .principal(principal)
        .rate_type(rate_type)
        .term_years(Decimal::from_str(term_years).ok()?)
        .frequency(PaymentFrequency::Monthly)
        .build()
        .ok()
}

/// Balance-over-time as an SVG path string (viewBox 300x140, high balance at
/// top-left sloping to zero at bottom-right) so the Compare tab can overlay
/// both scenarios on one chart. Scenarios that pay off before
/// `max_periods` get a flat tail at zero for the remaining width.
const CHART_W: f64 = 300.0;
const CHART_H: f64 = 140.0;

/// SVG-style path (viewBox 300x140) plotting `values` against `max_value`,
/// high value at the top (y=0) sloping toward the bottom (y=H) as it drops
/// toward zero. Shared by the Compare-tab payoff chart and the
/// Amortization-tab equity timeline so both stay pixel-compatible.
fn path_from_values(values: &[f64], max_value: f64, max_periods: usize) -> String {
    if values.is_empty() || max_periods == 0 {
        return String::new();
    }
    let max_value = max_value.max(1.0);
    let mut s = String::new();
    for (i, &v) in values.iter().enumerate() {
        let x = (i as f64 / max_periods.max(1) as f64) * CHART_W;
        let y = CHART_H - (v / max_value).clamp(0.0, 1.0) * CHART_H;
        if i == 0 {
            s.push_str(&format!("M {:.2} {:.2} ", x, y));
        } else {
            s.push_str(&format!("L {:.2} {:.2} ", x, y));
        }
    }
    s
}

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse::<f64>().unwrap_or(0.0)
}

fn build_chart_path(rows: &[mortgage_calc::amortization::AmortizationRow], principal: Decimal, max_periods: usize) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let values: Vec<f64> = rows.iter().map(|r| decimal_to_f64(r.remaining_balance)).collect();
    let mut s = path_from_values(&values, decimal_to_f64(principal), max_periods);
    if rows.len() < max_periods {
        s.push_str(&format!("L {:.2} {:.2} ", CHART_W, CHART_H));
    }
    s
}

/// Remaining-principal and cumulative-interest-paid curves for the
/// Amortization tab's equity timeline (single schedule, so no padding tail
/// like the Compare-tab chart needs).
fn build_timeline_paths(rows: &[mortgage_calc::amortization::AmortizationRow], principal: Decimal) -> (String, String) {
    let n = rows.len();
    let balance_values: Vec<f64> = rows.iter().map(|r| decimal_to_f64(r.remaining_balance)).collect();

    let mut cumulative_interest = Decimal::ZERO;
    let interest_values: Vec<f64> = rows
        .iter()
        .map(|r| {
            cumulative_interest += r.interest_portion;
            decimal_to_f64(cumulative_interest)
        })
        .collect();
    let total_interest_f = interest_values.last().copied().unwrap_or(1.0);

    let balance_path = path_from_values(&balance_values, decimal_to_f64(principal), n);
    let interest_path = path_from_values(&interest_values, total_interest_f, n);
    (balance_path, interest_path)
}

/// Recomputes the scrub summary (month/balance/interest/equity) for the
/// given `fraction` (0..1) along `rows`, or clears it when there's no
/// schedule. Called both when the user drags the scrub line and whenever
/// the underlying loan changes, so the summary stays in sync either way.
fn apply_scrub(ui: &AppWindow, rows: &[mortgage_calc::amortization::AmortizationRow], principal: Decimal, fraction: f32) {
    if rows.is_empty() {
        ui.set_amort_scrub_month_label("".into());
        ui.set_amort_scrub_balance_label("".into());
        ui.set_amort_scrub_interest_label("".into());
        ui.set_amort_scrub_equity_label("".into());
        ui.set_amort_scrub_equity_percent(0.0);
        return;
    }

    let fraction = fraction.clamp(0.0, 1.0);
    let idx = ((fraction as f64) * (rows.len() - 1) as f64).round() as usize;
    let idx = idx.min(rows.len() - 1);
    let row = rows[idx];

    let cumulative_interest: Decimal = rows[..=idx].iter().map(|r| r.interest_portion).sum();
    let equity = (principal - row.remaining_balance).max(Decimal::ZERO);
    let equity_percent = if principal > Decimal::ZERO {
        decimal_to_f64(equity / principal * Decimal::from(100)) as f32
    } else {
        0.0
    };

    ui.set_amort_scrub_fraction(fraction);
    ui.set_amort_scrub_month_label(format!("Month {}", row.period).into());
    ui.set_amort_scrub_balance_label(format!("${}", row.remaining_balance).into());
    ui.set_amort_scrub_interest_label(format!("${}", cumulative_interest).into());
    ui.set_amort_scrub_equity_label(format!("${}", equity).into());
    ui.set_amort_scrub_equity_percent(equity_percent);
}

#[allow(clippy::too_many_arguments)]
fn compare_slot(
    principal: Decimal,
    floating: bool,
    rate_percent: &str,
    base_rate_percent: &str,
    spread_percent: &str,
    term_years: &str,
) -> Option<String> {
    let rate_type = build_rate_type(floating, rate_percent, base_rate_percent, spread_percent)?;

    let input = ComparisonInput {
        principal,
        frequency: PaymentFrequency::Monthly,
    };
    let entry = ComparisonEntry {
        label: String::new(),
        rate_type,
        term_years: Decimal::from_str(term_years).ok()?,
    };

    let rows = mortgage_calc::comparison::compare(&input, std::slice::from_ref(&entry)).ok()?;
    let row = rows.into_iter().next()?;

    Some(format!(
        "{:.3}% \u{2022} {} yr \u{2022} ${}/mo \u{2022} ${} total \u{2022} ${} interest",
        row.effective_rate * Decimal::from(100),
        row.term_years,
        row.payment,
        row.total_paid,
        row.total_interest
    ))
}

fn recompute_compare(ui: &AppWindow) {
    ui.set_cmp_a_rate_percent_value(parse_rate_value(ui.get_cmp_a_rate_percent().as_str()));
    ui.set_cmp_b_rate_percent_value(parse_rate_value(ui.get_cmp_b_rate_percent().as_str()));
    let Some(principal) = Decimal::from_str(ui.get_cmp_principal().as_str()).ok() else {
        ui.set_cmp_has_error(true);
        return;
    };

    let a = compare_slot(
        principal,
        ui.get_cmp_a_floating(),
        ui.get_cmp_a_rate_percent().as_str(),
        ui.get_cmp_a_base_rate_percent().as_str(),
        ui.get_cmp_a_spread_percent().as_str(),
        ui.get_cmp_a_term_years().as_str(),
    );
    let b = compare_slot(
        principal,
        ui.get_cmp_b_floating(),
        ui.get_cmp_b_rate_percent().as_str(),
        ui.get_cmp_b_base_rate_percent().as_str(),
        ui.get_cmp_b_spread_percent().as_str(),
        ui.get_cmp_b_term_years().as_str(),
    );

    match (a, b) {
        (Some(a), Some(b)) => {
            ui.set_cmp_has_error(false);
            ui.set_cmp_a_result(a.into());
            ui.set_cmp_b_result(b.into());
        }
        _ => {
            ui.set_cmp_has_error(true);
            ui.set_cmp_chart_path_a("".into());
            ui.set_cmp_chart_path_b("".into());
            return;
        }
    }

    let loan_a = build_compare_loan(
        principal,
        ui.get_cmp_a_floating(),
        ui.get_cmp_a_rate_percent().as_str(),
        ui.get_cmp_a_base_rate_percent().as_str(),
        ui.get_cmp_a_spread_percent().as_str(),
        ui.get_cmp_a_term_years().as_str(),
    );
    let loan_b = build_compare_loan(
        principal,
        ui.get_cmp_b_floating(),
        ui.get_cmp_b_rate_percent().as_str(),
        ui.get_cmp_b_base_rate_percent().as_str(),
        ui.get_cmp_b_spread_percent().as_str(),
        ui.get_cmp_b_term_years().as_str(),
    );

    match (loan_a, loan_b) {
        (Some(loan_a), Some(loan_b)) => {
            let rows_a = mortgage_calc::amortization::schedule(&loan_a, Decimal::ZERO).unwrap_or_default();
            let rows_b = mortgage_calc::amortization::schedule(&loan_b, Decimal::ZERO).unwrap_or_default();
            let max_periods = rows_a.len().max(rows_b.len());
            ui.set_cmp_chart_path_a(build_chart_path(&rows_a, principal, max_periods).into());
            ui.set_cmp_chart_path_b(build_chart_path(&rows_b, principal, max_periods).into());
        }
        _ => {
            ui.set_cmp_chart_path_a("".into());
            ui.set_cmp_chart_path_b("".into());
        }
    }
}

/// Best-effort region guess from the device/browser locale (e.g. `en-SG`),
/// used to preset the region toggle on startup. Falls back to
/// [`Region::US`] when the locale can't be read or isn't recognized.
#[cfg(not(target_family = "wasm"))]
fn detect_region() -> Region {
    sys_locale::get_locale()
        .map(|locale| Region::parse(&locale))
        .unwrap_or_default()
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen(inline_js = "export function navigator_language() { \
    return (typeof navigator !== 'undefined' && navigator.language) || null; \
}")]
extern "C" {
    fn navigator_language() -> Option<String>;
}

#[cfg(target_family = "wasm")]
fn detect_region() -> Region {
    navigator_language()
        .map(|locale| Region::parse(&locale))
        .unwrap_or_default()
}

/// Shared entry point: builds the window, wires callbacks, and runs the
/// event loop. Called from `main.rs` on native targets and from
/// [`run_wasm`] on wasm32.
pub fn run_app() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    ui.set_region(detect_region().as_str().into());

    // The `region` property is already updated by the toggle's two-way
    // binding by the time this fires; region-specific calculators re-run
    // here so their region-gated panels populate immediately on switch.
    let ui_handle = ui.as_weak();
    ui.on_region_changed(move |region| {
        let ui = ui_handle.unwrap();
        ui.set_region(region);
        recompute_payment(&ui);
    });

    recompute_payment(&ui);
    recompute_amortization(&ui);
    recompute_affordability(&ui);
    recompute_refinance(&ui);
    recompute_compare(&ui);

    let ui_handle = ui.as_weak();
    ui.on_recompute(move || {
        let ui = ui_handle.unwrap();
        recompute_payment(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_rate_slider_changed(move |v| {
        let ui = ui_handle.unwrap();
        ui.set_rate_percent(format_rate_value(v).into());
        recompute_payment(&ui);
    });

    // Dragging down payment % re-derives the loan amount from home price,
    // exactly like editing "Home loan amount" directly would.
    let ui_handle = ui.as_weak();
    ui.on_us_down_payment_changed(move |percent| {
        let ui = ui_handle.unwrap();
        let home_price = Decimal::from_str(ui.get_us_home_price().as_str()).unwrap_or_default().max(Decimal::ZERO);
        let percent_dec = Decimal::from_str(&format_rate_value(percent.clamp(0.0, 100.0)))
            .unwrap_or_default()
            / dec!(100);
        let new_principal = round_currency(home_price * (Decimal::ONE - percent_dec));
        ui.set_principal(new_principal.to_string().into());
        recompute_payment(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_amort_recompute(move || {
        let ui = ui_handle.unwrap();
        recompute_amortization(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_amort_rate_slider_changed(move |v| {
        let ui = ui_handle.unwrap();
        ui.set_amort_rate_percent(format_rate_value(v).into());
        recompute_amortization(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_amort_whatif_toggled(move || {
        let ui = ui_handle.unwrap();
        let mut extra = Decimal::ZERO;
        if ui.get_amort_lattes_on() {
            extra += Decimal::from(50);
        }
        if ui.get_amort_subscription_on() {
            extra += Decimal::from(15);
        }
        if ui.get_amort_bonus_on() {
            extra += Decimal::from(83);
        }
        ui.set_amort_extra_payment(extra.to_string().into());
        recompute_amortization(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_amort_toggle_full(move || {
        let ui = ui_handle.unwrap();
        ui.set_amort_show_full(!ui.get_amort_show_full());
        recompute_amortization(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_amort_scrub(move |fraction| {
        let ui = ui_handle.unwrap();
        let Some(loan) = build_loan(
            ui.get_amort_principal().as_str(),
            ui.get_amort_rate_percent().as_str(),
            ui.get_amort_term_years().as_str(),
        ) else {
            return;
        };
        let extra = Decimal::from_str(ui.get_amort_extra_payment().as_str())
            .unwrap_or(Decimal::ZERO)
            .max(Decimal::ZERO);
        let Ok(rows) = mortgage_calc::amortization::schedule(&loan, extra) else {
            return;
        };
        let principal_d = Decimal::from_str(ui.get_amort_principal().as_str()).unwrap_or(Decimal::ZERO);
        apply_scrub(&ui, &rows, principal_d, fraction);
    });

    let ui_handle = ui.as_weak();
    ui.on_aff_recompute(move || {
        let ui = ui_handle.unwrap();
        recompute_affordability(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_aff_rate_slider_changed(move |v| {
        let ui = ui_handle.unwrap();
        ui.set_aff_rate_percent(format_rate_value(v).into());
        recompute_affordability(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_refi_recompute(move || {
        let ui = ui_handle.unwrap();
        recompute_refinance(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_refi_current_rate_slider_changed(move |v| {
        let ui = ui_handle.unwrap();
        ui.set_refi_current_rate_percent(format_rate_value(v).into());
        recompute_refinance(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_refi_new_rate_slider_changed(move |v| {
        let ui = ui_handle.unwrap();
        ui.set_refi_new_rate_percent(format_rate_value(v).into());
        recompute_refinance(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_cmp_recompute(move || {
        let ui = ui_handle.unwrap();
        recompute_compare(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_cmp_a_rate_slider_changed(move |v| {
        let ui = ui_handle.unwrap();
        ui.set_cmp_a_rate_percent(format_rate_value(v).into());
        recompute_compare(&ui);
    });

    let ui_handle = ui.as_weak();
    ui.on_cmp_b_rate_slider_changed(move |v| {
        let ui = ui_handle.unwrap();
        ui.set_cmp_b_rate_percent(format_rate_value(v).into());
        recompute_compare(&ui);
    });

    let store_cell: StoreCell = Rc::new(RefCell::new(None));

    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        slint::spawn_local(async move {
            let store = storage::open_store().await;
            if let Some(ui) = ui_handle.upgrade() {
                refresh_all_scenarios(&ui, &store).await;
            }
            *store_cell.borrow_mut() = Some(store);
        })
        .expect("failed to spawn scenario store init");
    }

    // --- Payment scenarios ---
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_payment_save_scenario(move |name| {
            let Some(ui) = ui_handle.upgrade() else { return };
            let inputs = PaymentInputs {
                principal: ui.get_principal().to_string(),
                rate_percent: ui.get_rate_percent().to_string(),
                term_years: ui.get_term_years().to_string(),
                region: ui.get_region().to_string(),
                sg_gross_income: ui.get_sg_gross_income().to_string(),
                sg_other_debts: ui.get_sg_other_debts().to_string(),
                sg_is_hdb: ui.get_sg_is_hdb(),
                sg_loan_type: ui.get_sg_loan_type().to_string(),
                sg_use_cpf: ui.get_sg_use_cpf(),
                sg_cpf_oa_available: ui.get_sg_cpf_oa_available().to_string(),
                sg_home_price: ui.get_sg_home_price().to_string(),
                sg_residency: ui.get_sg_residency().to_string(),
                sg_property_count: ui.get_sg_property_count().to_string(),
                us_home_price: ui.get_us_home_price().to_string(),
                us_zip: ui.get_us_zip().to_string(),
                us_pmi_rate_percent: ui.get_us_pmi_rate_percent().to_string(),
                us_use_tax_deduction: ui.get_us_use_tax_deduction(),
                us_marginal_tax_rate_percent: ui.get_us_marginal_tax_rate_percent().to_string(),
            };
            let Ok(inputs_json) = serde_json::to_string(&inputs) else { return };
            do_save(
                ui_handle.clone(),
                store_cell.clone(),
                CalculatorKind::Payment,
                name.to_string(),
                inputs_json,
                |ui, err| ui.set_payment_scenario_error(err),
            );
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_payment_load_scenario(move |id| {
            do_load(ui_handle.clone(), store_cell.clone(), id.to_string(), |ui, json| {
                if let Ok(inputs) = serde_json::from_str::<PaymentInputs>(json) {
                    ui.set_principal(inputs.principal.into());
                    ui.set_rate_percent(inputs.rate_percent.into());
                    ui.set_term_years(inputs.term_years.into());
                    // Scenarios saved before the region/SG/US panels existed
                    // deserialize these as empty strings (see `#[serde(default)]`
                    // on PaymentInputs) — leave the panel's own defaults alone
                    // in that case rather than blanking them out.
                    let apply = |value: String, set: &mut dyn FnMut(slint::SharedString)| {
                        if !value.is_empty() {
                            set(value.into());
                        }
                    };
                    apply(inputs.region, &mut |v| ui.set_region(v));
                    apply(inputs.sg_gross_income, &mut |v| ui.set_sg_gross_income(v));
                    apply(inputs.sg_other_debts, &mut |v| ui.set_sg_other_debts(v));
                    apply(inputs.sg_loan_type, &mut |v| ui.set_sg_loan_type(v));
                    apply(inputs.sg_cpf_oa_available, &mut |v| ui.set_sg_cpf_oa_available(v));
                    apply(inputs.sg_home_price, &mut |v| ui.set_sg_home_price(v));
                    apply(inputs.sg_residency, &mut |v| ui.set_sg_residency(v));
                    apply(inputs.sg_property_count, &mut |v| ui.set_sg_property_count(v));
                    apply(inputs.us_home_price, &mut |v| ui.set_us_home_price(v));
                    apply(inputs.us_zip, &mut |v| ui.set_us_zip(v));
                    apply(inputs.us_pmi_rate_percent, &mut |v| ui.set_us_pmi_rate_percent(v));
                    apply(inputs.us_marginal_tax_rate_percent, &mut |v| ui.set_us_marginal_tax_rate_percent(v));
                    // Bools default to exactly what a fresh panel already
                    // shows (sg_is_hdb: true, the rest: false), so applying
                    // them unconditionally is safe even for old scenarios.
                    ui.set_sg_is_hdb(inputs.sg_is_hdb);
                    ui.set_sg_use_cpf(inputs.sg_use_cpf);
                    ui.set_us_use_tax_deduction(inputs.us_use_tax_deduction);
                    recompute_payment(ui);
                }
            });
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_payment_delete_scenario(move |id| {
            do_delete(ui_handle.clone(), store_cell.clone(), CalculatorKind::Payment, id.to_string());
        });
    }

    // --- Amortization scenarios ---
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_amort_save_scenario(move |name| {
            let Some(ui) = ui_handle.upgrade() else { return };
            let inputs = AmortInputs {
                principal: ui.get_amort_principal().to_string(),
                rate_percent: ui.get_amort_rate_percent().to_string(),
                term_years: ui.get_amort_term_years().to_string(),
                extra_payment: ui.get_amort_extra_payment().to_string(),
            };
            let Ok(inputs_json) = serde_json::to_string(&inputs) else { return };
            do_save(
                ui_handle.clone(),
                store_cell.clone(),
                CalculatorKind::Amortization,
                name.to_string(),
                inputs_json,
                |ui, err| ui.set_amort_scenario_error(err),
            );
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_amort_load_scenario(move |id| {
            do_load(ui_handle.clone(), store_cell.clone(), id.to_string(), |ui, json| {
                if let Ok(inputs) = serde_json::from_str::<AmortInputs>(json) {
                    ui.set_amort_principal(inputs.principal.into());
                    ui.set_amort_rate_percent(inputs.rate_percent.into());
                    ui.set_amort_term_years(inputs.term_years.into());
                    ui.set_amort_extra_payment(inputs.extra_payment.into());
                    recompute_amortization(ui);
                }
            });
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_amort_delete_scenario(move |id| {
            do_delete(ui_handle.clone(), store_cell.clone(), CalculatorKind::Amortization, id.to_string());
        });
    }

    // --- Affordability scenarios ---
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_aff_save_scenario(move |name| {
            let Some(ui) = ui_handle.upgrade() else { return };
            let inputs = AffInputs {
                income: ui.get_aff_income().to_string(),
                debts: ui.get_aff_debts().to_string(),
                down_payment: ui.get_aff_down_payment().to_string(),
                rate_percent: ui.get_aff_rate_percent().to_string(),
                term_years: ui.get_aff_term_years().to_string(),
                max_dti_percent: ui.get_aff_max_dti_percent().to_string(),
                tax_rate_percent: ui.get_aff_tax_rate_percent().to_string(),
                insurance: ui.get_aff_insurance().to_string(),
                hoa: ui.get_aff_hoa().to_string(),
            };
            let Ok(inputs_json) = serde_json::to_string(&inputs) else { return };
            do_save(
                ui_handle.clone(),
                store_cell.clone(),
                CalculatorKind::Affordability,
                name.to_string(),
                inputs_json,
                |ui, err| ui.set_aff_scenario_error(err),
            );
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_aff_load_scenario(move |id| {
            do_load(ui_handle.clone(), store_cell.clone(), id.to_string(), |ui, json| {
                if let Ok(inputs) = serde_json::from_str::<AffInputs>(json) {
                    ui.set_aff_income(inputs.income.into());
                    ui.set_aff_debts(inputs.debts.into());
                    ui.set_aff_down_payment(inputs.down_payment.into());
                    ui.set_aff_rate_percent(inputs.rate_percent.into());
                    ui.set_aff_term_years(inputs.term_years.into());
                    ui.set_aff_max_dti_percent(inputs.max_dti_percent.into());
                    ui.set_aff_tax_rate_percent(inputs.tax_rate_percent.into());
                    ui.set_aff_insurance(inputs.insurance.into());
                    ui.set_aff_hoa(inputs.hoa.into());
                    recompute_affordability(ui);
                }
            });
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_aff_delete_scenario(move |id| {
            do_delete(ui_handle.clone(), store_cell.clone(), CalculatorKind::Affordability, id.to_string());
        });
    }

    // --- Refinance scenarios ---
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_refi_save_scenario(move |name| {
            let Some(ui) = ui_handle.upgrade() else { return };
            let inputs = RefiInputs {
                current_balance: ui.get_refi_current_balance().to_string(),
                current_rate_percent: ui.get_refi_current_rate_percent().to_string(),
                remaining_periods: ui.get_refi_remaining_periods().to_string(),
                new_rate_percent: ui.get_refi_new_rate_percent().to_string(),
                new_term_years: ui.get_refi_new_term_years().to_string(),
                closing_costs: ui.get_refi_closing_costs().to_string(),
            };
            let Ok(inputs_json) = serde_json::to_string(&inputs) else { return };
            do_save(
                ui_handle.clone(),
                store_cell.clone(),
                CalculatorKind::Refinance,
                name.to_string(),
                inputs_json,
                |ui, err| ui.set_refi_scenario_error(err),
            );
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_refi_load_scenario(move |id| {
            do_load(ui_handle.clone(), store_cell.clone(), id.to_string(), |ui, json| {
                if let Ok(inputs) = serde_json::from_str::<RefiInputs>(json) {
                    ui.set_refi_current_balance(inputs.current_balance.into());
                    ui.set_refi_current_rate_percent(inputs.current_rate_percent.into());
                    ui.set_refi_remaining_periods(inputs.remaining_periods.into());
                    ui.set_refi_new_rate_percent(inputs.new_rate_percent.into());
                    ui.set_refi_new_term_years(inputs.new_term_years.into());
                    ui.set_refi_closing_costs(inputs.closing_costs.into());
                    recompute_refinance(ui);
                }
            });
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_refi_delete_scenario(move |id| {
            do_delete(ui_handle.clone(), store_cell.clone(), CalculatorKind::Refinance, id.to_string());
        });
    }

    // --- Compare scenarios ---
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_cmp_save_scenario(move |name| {
            let Some(ui) = ui_handle.upgrade() else { return };
            let inputs = CompareInputs {
                principal: ui.get_cmp_principal().to_string(),
                a_label: ui.get_cmp_a_label().to_string(),
                a_floating: ui.get_cmp_a_floating(),
                a_rate_percent: ui.get_cmp_a_rate_percent().to_string(),
                a_base_rate_percent: ui.get_cmp_a_base_rate_percent().to_string(),
                a_spread_percent: ui.get_cmp_a_spread_percent().to_string(),
                a_term_years: ui.get_cmp_a_term_years().to_string(),
                b_label: ui.get_cmp_b_label().to_string(),
                b_floating: ui.get_cmp_b_floating(),
                b_rate_percent: ui.get_cmp_b_rate_percent().to_string(),
                b_base_rate_percent: ui.get_cmp_b_base_rate_percent().to_string(),
                b_spread_percent: ui.get_cmp_b_spread_percent().to_string(),
                b_term_years: ui.get_cmp_b_term_years().to_string(),
            };
            let Ok(inputs_json) = serde_json::to_string(&inputs) else { return };
            do_save(
                ui_handle.clone(),
                store_cell.clone(),
                CalculatorKind::Comparison,
                name.to_string(),
                inputs_json,
                |ui, err| ui.set_cmp_scenario_error(err),
            );
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_cmp_load_scenario(move |id| {
            do_load(ui_handle.clone(), store_cell.clone(), id.to_string(), |ui, json| {
                if let Ok(inputs) = serde_json::from_str::<CompareInputs>(json) {
                    ui.set_cmp_principal(inputs.principal.into());
                    ui.set_cmp_a_label(inputs.a_label.into());
                    ui.set_cmp_a_floating(inputs.a_floating);
                    ui.set_cmp_a_rate_percent(inputs.a_rate_percent.into());
                    ui.set_cmp_a_base_rate_percent(inputs.a_base_rate_percent.into());
                    ui.set_cmp_a_spread_percent(inputs.a_spread_percent.into());
                    ui.set_cmp_a_term_years(inputs.a_term_years.into());
                    ui.set_cmp_b_label(inputs.b_label.into());
                    ui.set_cmp_b_floating(inputs.b_floating);
                    ui.set_cmp_b_rate_percent(inputs.b_rate_percent.into());
                    ui.set_cmp_b_base_rate_percent(inputs.b_base_rate_percent.into());
                    ui.set_cmp_b_spread_percent(inputs.b_spread_percent.into());
                    ui.set_cmp_b_term_years(inputs.b_term_years.into());
                    recompute_compare(ui);
                }
            });
        });
    }
    {
        let store_cell = store_cell.clone();
        let ui_handle = ui.as_weak();
        ui.on_cmp_delete_scenario(move |id| {
            do_delete(ui_handle.clone(), store_cell.clone(), CalculatorKind::Comparison, id.to_string());
        });
    }

    ui.run()?;

    Ok(())
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen(start)]
pub fn run_wasm() {
    run_app().expect("failed to start mortgage-ui-slint");
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app).unwrap();
    run_app().expect("failed to start mortgage-ui-slint");
}
