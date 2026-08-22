//! Singapore-specific regulatory calculations: CPF Ordinary Account
//! integration, MAS TDSR/MSR borrowing limits, and BSD/ABSD stamp duty.
//!
//! Rates reflect the rulesets in effect since 15 February 2023 (BSD) and
//! 27 April 2023 (ABSD); MAS has not changed the 55%/30% TDSR/MSR ceilings
//! since their introduction.
//!
//! CPF usage here is simplified to "how much of this payment can CPF OA
//! cover, dollar for dollar" — it doesn't model the Valuation Limit/
//! Withdrawal Limit caps on total CPF usage over a property's life, since
//! those depend on the property's valuation history rather than anything
//! this calculator otherwise tracks.

use mortgage_core::{round_currency, MortgageError, MortgageResult, PaymentFrequency};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::loan::Loan;

/// How much of a monthly payment CPF Ordinary Account funds can cover,
/// versus how much cash is still required.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CpfCashSplit {
    pub cpf_used: Decimal,
    pub cash_required: Decimal,
}

/// Splits `monthly_payment` between CPF OA and cash, using CPF first up to
/// whatever is available (and needed).
pub fn cpf_cash_split(monthly_payment: Decimal, cpf_oa_available: Decimal) -> CpfCashSplit {
    let payment = monthly_payment.max(Decimal::ZERO);
    let cpf_used = cpf_oa_available.max(Decimal::ZERO).min(payment);
    CpfCashSplit {
        cpf_used: round_currency(cpf_used),
        cash_required: round_currency(payment - cpf_used),
    }
}

/// MAS's Total Debt Servicing Ratio ceiling: total monthly debt repayments
/// cannot exceed this share of gross monthly income. Applies to all
/// property loans.
pub const TDSR_LIMIT: Decimal = dec!(0.55);

/// MAS's Mortgage Servicing Ratio ceiling: the housing loan payment alone
/// cannot exceed this share of gross monthly income. Applies only to HDB
/// flats and Executive Condominiums.
pub const MSR_LIMIT: Decimal = dec!(0.30);

/// MAS's medium-term interest rate floor for residential property loans.
///
/// TDSR and MSR are *not* computed on the instalment a borrower is quoted.
/// MAS Notice 645 (last revised 21 August 2025) para 6(b) requires a bank to
/// "base its calculation of the monthly interest payable under the credit
/// facility on a medium-term interest rate", which for the purchase of
/// residential property is the "[h]igher of 4% or the thereafter interest
/// rate" — in force for options to purchase granted on or after
/// 30 September 2022.
///
/// Testing the contract instalment instead understates both ratios for
/// anyone quoted under 4%, which is precisely the borrower closest to the
/// ceiling. So the ratios here are always computed on a repriced loan.
pub const MEDIUM_TERM_RATE_FLOOR: Decimal = dec!(0.04);

/// The rate a bank must assess a residential property loan at: the floor, or
/// the rate the loan actually runs at, whichever is higher.
///
/// "Thereafter" is the operative word in the Notice and it is not a detail.
/// Every Singapore bank package opens on a promotional spread and steps up
/// after two or three years, so the rate a borrower is quoted is not the one
/// they spend the loan at. Assessing on the promotional rate would pass
/// borrowers a bank would decline -- the same one-directional error as
/// ignoring the floor, and invisible for as long as both rates sit under 4%.
pub fn assessment_rate(thereafter_annual_rate: Decimal) -> Decimal {
    thereafter_annual_rate.max(MEDIUM_TERM_RATE_FLOOR)
}

/// The share of variable income — commission, bonus, allowance — that counts
/// towards the servicing ratios.
///
/// MAS Notice 645 para 17(b): a bank may count "not more than 70% of ... the
/// average of the monthly variable income earned in the preceding 12 months".
/// Para 17(c)(i) applies the same haircut to the variable half of a mixed
/// income, leaving fixed salary whole.
pub const VARIABLE_INCOME_HAIRCUT: Decimal = dec!(0.70);

/// A borrower's monthly income, split the way Notice 645 para 17 splits it.
///
/// Kept as a pair rather than one figure because the two halves are not
/// interchangeable: $10,000 of salary supports a materially larger loan than
/// $10,000 of commission. A calculator that asks only for "gross monthly
/// income" silently assumes the borrower-friendly reading of every case.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MonthlyIncome {
    /// Fixed salary, counted in full — para 17(a).
    pub fixed: Decimal,
    /// Commission, bonus or allowance, counted at 70% — para 17(b).
    pub variable: Decimal,
}

impl MonthlyIncome {
    /// All fixed, no variable component — the common case.
    pub fn fixed(amount: Decimal) -> Self {
        Self {
            fixed: amount,
            variable: Decimal::ZERO,
        }
    }

    /// The income figure the servicing ratios are actually computed on.
    pub fn assessed(&self) -> Decimal {
        self.fixed.max(Decimal::ZERO) + self.variable.max(Decimal::ZERO) * VARIABLE_INCOME_HAIRCUT
    }
}

/// A ratio's standing against its regulatory ceiling: exceeded outright,
/// within a warning band below it, or comfortably clear.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LimitStatus {
    pub ratio: Decimal,
    pub exceeded: bool,
    /// Within 90% of the limit but not yet over it.
    pub near_limit: bool,
}

fn limit_status(ratio: Decimal, limit: Decimal) -> LimitStatus {
    LimitStatus {
        ratio,
        exceeded: ratio > limit,
        near_limit: ratio <= limit && ratio >= limit * dec!(0.9),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TdsrMsrCheck {
    pub tdsr: LimitStatus,
    /// `None` when the property isn't an HDB flat/EC, since MSR doesn't apply.
    pub msr: Option<LimitStatus>,
    /// The rate the ratios were assessed at — [`MEDIUM_TERM_RATE_FLOOR`]
    /// unless the borrower's own rate is higher.
    pub assessment_rate: Decimal,
    /// The instalment the ratios were computed from: this loan repriced at
    /// `assessment_rate`. Differs from what the borrower actually pays
    /// whenever they're quoted under the floor, so the UI can show both and
    /// explain the gap.
    pub assessed_monthly_instalment: Decimal,
    /// Income after the variable-income haircut — the denominator of both
    /// ratios. Below the borrower's headline pay whenever any of it is
    /// commission or bonus.
    pub assessed_monthly_income: Decimal,
}

/// Checks a loan against the MAS TDSR (always) and MSR (HDB/EC only)
/// ceilings.
///
/// Takes [`MonthlyIncome`] rather than one gross figure so the Notice 645
/// para 17 haircut on variable income is applied here, not left to callers to
/// remember. Takes the loan rather than a payment for the same reason. The ratios must be
/// computed on the instalment at [`MEDIUM_TERM_RATE_FLOOR`], not the one the
/// borrower is quoted, and a caller handed a payment has no way to tell
/// which it holds — so the repricing happens here, where it can't be
/// skipped.
pub fn check_tdsr_msr(
    loan: &Loan,
    other_monthly_debts: Decimal,
    income: MonthlyIncome,
    is_hdb_or_ec: bool,
) -> MortgageResult<TdsrMsrCheck> {
    let gross_monthly_income = income.assessed();
    if gross_monthly_income <= Decimal::ZERO {
        return Err(MortgageError::InvalidIncome(
            gross_monthly_income.to_string(),
        ));
    }

    // The rate that lasts, not the one the loan opens on.
    let assessment_rate = assessment_rate(loan.final_annual_rate());
    let assessed = Loan::builder()
        .principal(loan.principal())
        .annual_rate(assessment_rate)
        .term_years(loan.term_years())
        // TDSR and MSR are monthly ceilings, so the assessed instalment is a
        // monthly one regardless of how the borrower actually repays.
        .frequency(PaymentFrequency::Monthly)
        .build()?;
    let instalment = crate::payment::payment_amount(&assessed);

    let tdsr_ratio = (instalment + other_monthly_debts) / gross_monthly_income;
    let msr = is_hdb_or_ec.then(|| limit_status(instalment / gross_monthly_income, MSR_LIMIT));

    Ok(TdsrMsrCheck {
        tdsr: limit_status(tdsr_ratio, TDSR_LIMIT),
        msr,
        assessment_rate,
        assessed_monthly_instalment: round_currency(instalment),
        assessed_monthly_income: round_currency(gross_monthly_income),
    })
}

/// Buyer's residency status, which sets the ABSD rate alongside property count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Residency {
    Citizen,
    PermanentResident,
    Foreigner,
    /// A foreigner entitled to National Treatment for stamp duty under one of
    /// Singapore's free trade agreements, and so charged ABSD at citizen
    /// rates rather than the flat 60%.
    ///
    /// Covers nationals of the **United States** (under the USSFTA — citizens
    /// only, not green-card holders), and nationals *and* permanent residents
    /// of **Iceland, Liechtenstein, Norway and Switzerland** (under the
    /// EFTA-Singapore FTA).
    ///
    /// The gap this closes is not marginal: a US citizen buying their first
    /// home here owes 0%, not 60% of the purchase price. Note the remission
    /// is claimed from IRAS rather than applied automatically, which the UI
    /// says out loud — the duty is assessed at the foreigner rate first.
    FtaNational,
}

/// How many residential properties the buyer will own after this purchase,
/// counting this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyCount {
    First,
    Second,
    ThirdOrMore,
}

/// The financing framework used: HDB's own concessionary loan, or a
/// private bank loan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanType {
    HdbLoan,
    BankLoan,
}

/// HDB concessionary loans are only available for HDB flats/ECs bought
/// directly from HDB — never for private property, and not offered by
/// banks.
pub fn hdb_loan_eligible(is_hdb_or_ec: bool) -> bool {
    is_hdb_or_ec
}

/// Progressive Buyer's Stamp Duty on a residential property's purchase
/// price (or market value, if higher).
pub fn buyers_stamp_duty(price: Decimal) -> Decimal {
    const TIERS: [(Decimal, Decimal); 5] = [
        (dec!(180_000), dec!(0.01)),
        (dec!(180_000), dec!(0.02)),
        (dec!(640_000), dec!(0.03)),
        (dec!(500_000), dec!(0.04)),
        (dec!(1_500_000), dec!(0.05)),
    ];
    const TOP_RATE: Decimal = dec!(0.06);

    let mut remaining = price.max(Decimal::ZERO);
    let mut duty = Decimal::ZERO;
    for (band, rate) in TIERS {
        if remaining <= Decimal::ZERO {
            break;
        }
        let taxed = remaining.min(band);
        duty += taxed * rate;
        remaining -= taxed;
    }
    duty += remaining * TOP_RATE;
    round_currency(duty)
}

/// Flat-rate Additional Buyer's Stamp Duty on the purchase price, layered
/// on top of BSD.
pub fn additional_buyers_stamp_duty(
    price: Decimal,
    residency: Residency,
    count: PropertyCount,
) -> Decimal {
    let rate = match (residency, count) {
        (Residency::Citizen, PropertyCount::First) => Decimal::ZERO,
        (Residency::Citizen, PropertyCount::Second) => dec!(0.20),
        (Residency::Citizen, PropertyCount::ThirdOrMore) => dec!(0.30),
        (Residency::PermanentResident, PropertyCount::First) => dec!(0.05),
        (Residency::PermanentResident, PropertyCount::Second) => dec!(0.30),
        (Residency::PermanentResident, PropertyCount::ThirdOrMore) => dec!(0.35),
        (Residency::Foreigner, _) => dec!(0.60),
        // National Treatment: charged exactly as a citizen would be.
        (Residency::FtaNational, PropertyCount::First) => Decimal::ZERO,
        (Residency::FtaNational, PropertyCount::Second) => dec!(0.20),
        (Residency::FtaNational, PropertyCount::ThirdOrMore) => dec!(0.30),
    };
    round_currency(price.max(Decimal::ZERO) * rate)
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UpfrontCosts {
    pub bsd: Decimal,
    pub absd: Decimal,
    pub total: Decimal,
}

/// BSD + ABSD due on a purchase, given the buyer's residency and how many
/// properties (including this one) they'll hold.
pub fn upfront_costs(price: Decimal, residency: Residency, count: PropertyCount) -> UpfrontCosts {
    let bsd = buyers_stamp_duty(price);
    let absd = additional_buyers_stamp_duty(price, residency, count);
    UpfrontCosts {
        bsd,
        absd,
        total: round_currency(bsd + absd),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpf_covers_up_to_whats_available() {
        let split = cpf_cash_split(dec!(2500), dec!(1500));
        assert_eq!(split.cpf_used, dec!(1500));
        assert_eq!(split.cash_required, dec!(1000));
    }

    #[test]
    fn cpf_cannot_exceed_the_payment_itself() {
        let split = cpf_cash_split(dec!(2500), dec!(4000));
        assert_eq!(split.cpf_used, dec!(2500));
        assert_eq!(split.cash_required, dec!(0));
    }

    /// A loan whose instalment lands wherever the test needs it. `rate` is a
    /// fraction, matching `Loan::annual_rate`.
    fn loan_of(principal: Decimal, rate: Decimal, years: Decimal) -> Loan {
        Loan::builder()
            .principal(principal)
            .annual_rate(rate)
            .term_years(years)
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap()
    }

    #[test]
    fn tdsr_flags_when_total_debt_exceeds_55_percent() {
        // ~$5,057/mo assessed, plus $1,000 other debt, against $10,000 income.
        let check = check_tdsr_msr(
            &loan_of(dec!(1_000_000), dec!(0.045), dec!(30)),
            dec!(1000),
            MonthlyIncome::fixed(dec!(10_000)),
            false,
        )
        .unwrap();
        assert!(check.tdsr.exceeded);
        assert!(check.msr.is_none());
    }

    #[test]
    fn msr_only_applies_to_hdb_and_is_tighter_than_tdsr() {
        // ~$3,341/mo: clear of TDSR's 55%, over MSR's 30%.
        let check = check_tdsr_msr(
            &loan_of(dec!(700_000), dec!(0.04), dec!(30)),
            dec!(0),
            MonthlyIncome::fixed(dec!(10_000)),
            true,
        )
        .unwrap();
        assert!(!check.tdsr.exceeded);
        assert!(check.msr.unwrap().exceeded);
    }

    #[test]
    fn rejects_non_positive_income() {
        assert!(check_tdsr_msr(
            &loan_of(dec!(500_000), dec!(0.04), dec!(25)),
            dec!(0),
            MonthlyIncome::fixed(dec!(0)),
            false
        )
        .is_err());
    }

    #[test]
    fn assesses_a_cheap_loan_at_the_mas_floor_rather_than_its_own_rate() {
        // MAS Notice 645 para 6(b): residential loans are assessed at the
        // higher of 4% and the contract rate. At 1.5% this borrower looks far
        // more affordable than a bank is permitted to find them.
        // $1.2M at 1.5% is ~$4,141/mo — 41% of income, comfortably inside
        // TDSR. Repriced at the 4% floor it is ~$5,728/mo, or 57%, which is
        // not. A bank would decline this borrower; so must the calculator.
        let loan = loan_of(dec!(1_200_000), dec!(0.015), dec!(30));
        let check =
            check_tdsr_msr(&loan, dec!(0), MonthlyIncome::fixed(dec!(10_000)), false).unwrap();

        assert_eq!(check.assessment_rate, MEDIUM_TERM_RATE_FLOOR);

        let contract_instalment = crate::payment::payment_amount(&loan);
        assert!(
            check.assessed_monthly_instalment > contract_instalment,
            "assessed {} should exceed the contract instalment {}",
            check.assessed_monthly_instalment,
            contract_instalment
        );

        // The whole point: the stress is what pushes this over the ceiling.
        assert!(check.tdsr.exceeded);
        let unstressed_ratio = contract_instalment / dec!(10_000);
        assert!(unstressed_ratio < TDSR_LIMIT);
    }

    #[test]
    fn leaves_a_loan_above_the_floor_at_its_own_rate() {
        let loan = loan_of(dec!(500_000), dec!(0.065), dec!(25));
        let check =
            check_tdsr_msr(&loan, dec!(0), MonthlyIncome::fixed(dec!(20_000)), false).unwrap();

        assert_eq!(check.assessment_rate, dec!(0.065));
        assert_eq!(
            check.assessed_monthly_instalment,
            round_currency(crate::payment::payment_amount(&loan))
        );
    }

    #[test]
    fn assessment_rate_takes_the_higher_of_the_floor_and_the_contract_rate() {
        assert_eq!(assessment_rate(dec!(0.015)), MEDIUM_TERM_RATE_FLOOR);
        assert_eq!(assessment_rate(dec!(0.04)), MEDIUM_TERM_RATE_FLOOR);
        assert_eq!(assessment_rate(dec!(0.072)), dec!(0.072));
    }

    #[test]
    fn a_stepping_package_is_assessed_on_the_rate_that_lasts() {
        // The failure this prevents: a package quoted at 3.5% promotional
        // and 4.8% thereafter. Assessed on the teaser the borrower is tested
        // at the 4% floor and passes; assessed correctly they are tested at
        // 4.8% and may not. Invisible while both rates sit under 4%, which
        // is exactly where Singapore rates are today -- so it would have
        // surfaced only once rates rose, on the borrowers least able to
        // absorb it.
        let stepping = Loan::builder()
            .principal(dec!(800_000))
            .annual_rate(dec!(0.035))
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Monthly)
            .reversion(crate::loan::Reversion {
                after_periods: 24,
                annual_rate: dec!(0.048),
            })
            .build()
            .unwrap();

        let check = check_tdsr_msr(
            &stepping,
            Decimal::ZERO,
            MonthlyIncome::fixed(dec!(12_000)),
            false,
        )
        .unwrap();

        assert_eq!(check.assessment_rate, dec!(0.048));
    }

    #[test]
    fn a_stepping_package_below_the_floor_is_still_assessed_at_the_floor() {
        // Both rates under 4%, which is every Singapore package right now.
        let sg_today = Loan::builder()
            .principal(dec!(400_000))
            .annual_rate(dec!(0.0142))
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Monthly)
            .reversion(crate::loan::Reversion {
                after_periods: 24,
                annual_rate: dec!(0.0172),
            })
            .build()
            .unwrap();

        let check = check_tdsr_msr(
            &sg_today,
            Decimal::ZERO,
            MonthlyIncome::fixed(dec!(12_000)),
            false,
        )
        .unwrap();

        assert_eq!(check.assessment_rate, MEDIUM_TERM_RATE_FLOOR);
    }

    #[test]
    fn bsd_matches_the_published_one_million_example() {
        // 1% * 180k + 2% * 180k + 3% * 640k = 1,800 + 3,600 + 19,200 = 24,600
        assert_eq!(buyers_stamp_duty(dec!(1_000_000)), dec!(24_600));
    }

    #[test]
    fn bsd_top_tier_applies_above_three_million() {
        // BSD(3,000,000) + 6% * 500,000 extra
        let below = buyers_stamp_duty(dec!(3_000_000));
        let above = buyers_stamp_duty(dec!(3_500_000));
        assert_eq!(above - below, dec!(30_000));
    }

    #[test]
    fn absd_is_zero_for_citizens_first_property() {
        assert_eq!(
            additional_buyers_stamp_duty(dec!(1_000_000), Residency::Citizen, PropertyCount::First),
            dec!(0)
        );
    }

    #[test]
    fn absd_foreigner_rate_is_flat_regardless_of_property_count() {
        let first = additional_buyers_stamp_duty(
            dec!(1_000_000),
            Residency::Foreigner,
            PropertyCount::First,
        );
        let third = additional_buyers_stamp_duty(
            dec!(1_000_000),
            Residency::Foreigner,
            PropertyCount::ThirdOrMore,
        );
        assert_eq!(first, dec!(600_000));
        assert_eq!(first, third);
    }

    #[test]
    fn upfront_costs_sums_bsd_and_absd() {
        let costs = upfront_costs(
            dec!(1_000_000),
            Residency::PermanentResident,
            PropertyCount::Second,
        );
        assert_eq!(costs.bsd, dec!(24_600));
        assert_eq!(costs.absd, dec!(300_000));
        assert_eq!(costs.total, dec!(324_600));
    }

    #[test]
    fn hdb_loan_requires_an_hdb_or_ec_flat() {
        assert!(hdb_loan_eligible(true));
        assert!(!hdb_loan_eligible(false));
    }
}

// ---------------------------------------------------------------------------
// Borrowing capacity
// ---------------------------------------------------------------------------

/// The LTV ceiling and minimum cash share for a purchase, from MAS Notice
/// 632 (last revised 5 July 2018), the LTV limit table.
///
/// The table keys off three things: how many housing loans the borrower
/// already has outstanding, whether the property is an HDB flat, and whether
/// the tenure is "extended" — over 30 years (25 for an HDB flat), or running
/// past the borrower's 65th birthday. Extended tenure drops the ceiling
/// sharply and raises the cash floor.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LtvLimit {
    /// Share of the property price that may be financed, e.g. `0.75`.
    pub ltv: Decimal,
    /// Share of the price that must be paid in cash rather than CPF.
    pub min_cash: Decimal,
    /// Whether the extended-tenure rows applied.
    pub extended_tenure: bool,
}

/// Longest tenure that avoids the extended-tenure LTV haircut. Notice 632
/// rows (4C)/(7A) use 30 years for private property; (4D)/(7B) use 25 for
/// an HDB flat.
fn standard_tenure_years(is_hdb_or_ec: bool) -> Decimal {
    if is_hdb_or_ec {
        dec!(25)
    } else {
        dec!(30)
    }
}

/// Age past which a loan is treated as extended-tenure, per the
/// "tenure + age of the Borrower ... less than or equal to 65 years" test.
const MAX_AGE_AT_MATURITY: Decimal = dec!(65);

/// The LTV row that applies to a purchase.
///
/// `outstanding_housing_loans` counts loans the borrower already services,
/// so `0` is a first-time buyer. `borrower_age` is optional because the app
/// doesn't always ask; when absent only the tenure test can be applied, which
/// is the borrower-friendly reading and is flagged as such by the caller.
pub fn ltv_limit(
    outstanding_housing_loans: u32,
    is_hdb_or_ec: bool,
    term_years: Decimal,
    borrower_age: Option<Decimal>,
) -> LtvLimit {
    let over_tenure = term_years > standard_tenure_years(is_hdb_or_ec);
    let past_retirement = borrower_age
        .map(|age| age + term_years > MAX_AGE_AT_MATURITY)
        .unwrap_or(false);
    let extended = over_tenure || past_retirement;

    let (ltv, min_cash) = match (outstanding_housing_loans, extended) {
        (0, false) => (dec!(0.75), dec!(0.05)), // Notice 632 (4C)/(4D)
        (0, true) => (dec!(0.55), dec!(0.10)),  // (7A)/(7B)
        (1, false) => (dec!(0.45), dec!(0.25)), // (11C)/(11D)
        (1, true) => (dec!(0.25), dec!(0.25)),  // (14A)/(14B)
        (_, false) => (dec!(0.35), dec!(0.25)), // (17A)/(17B)
        (_, true) => (dec!(0.15), dec!(0.25)),  // (20A)/(20B)
    };

    LtvLimit {
        ltv,
        min_cash,
        extended_tenure: extended,
    }
}

/// Which rule caps the *loan*. Naming it is the useful part — "you can
/// afford $X" is far less actionable than knowing whether earning more or
/// saving more is what would move it.
///
/// Note this is deliberately not "what caps the price". A more expensive
/// property always needs a larger deposit, so the buyer's funds bound the
/// price in every case; saying so would be true of everyone and tell nobody
/// anything. The question worth answering is which side of the purchase is
/// actually tight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingConstraint {
    /// TDSR: what the borrower can service, after existing debts.
    Tdsr,
    /// MSR: the tighter housing-only ceiling on an HDB flat or EC.
    Msr,
    /// The loan sits on the MAS LTV ceiling, so it is the deposit — not
    /// income — that limits it. Earning more would not raise this figure;
    /// a larger deposit would.
    Ltv,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct SgAffordabilityInput {
    pub income: MonthlyIncome,
    pub other_monthly_debts: Decimal,
    /// Cash the buyer holds. Kept apart from CPF because the two are not
    /// interchangeable at completion: the minimum cash down payment cannot
    /// be met from CPF, and both stamp duties must be paid in cash within 14
    /// days — CPF only reimburses afterwards, too late to complete on.
    pub cash_available: Decimal,
    /// CPF Ordinary Account balance usable for this purchase. Can cover the
    /// deposit above the cash floor, and nothing else here.
    pub cpf_oa_available: Decimal,
    /// The rate the loan opens at, as a fraction. What the borrower pays
    /// first, and what the instalment shown to them is built from.
    pub annual_rate: Decimal,
    /// The rate after the lock-in, for a package that steps up. `None` for a
    /// rate that holds for the term.
    ///
    /// Servicing is assessed at the higher of this and
    /// [`MEDIUM_TERM_RATE_FLOOR`] -- "thereafter" is the Notice's word, and
    /// assessing on the promotional rate would pass borrowers a bank would
    /// decline.
    pub thereafter_annual_rate: Option<Decimal>,
    pub term_years: Decimal,
    pub borrower_age: Option<Decimal>,
    pub is_hdb_or_ec: bool,
    pub residency: Residency,
    /// Properties held *after* this purchase — the ABSD basis.
    pub property_count: PropertyCount,
    /// Housing loans already outstanding — the LTV basis. Distinct from
    /// `property_count`: a buyer can own a property outright and still be
    /// taking a first housing loan.
    pub outstanding_housing_loans: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct SgAffordabilityResult {
    pub max_price: Decimal,
    pub max_loan: Decimal,
    pub binding_constraint: BindingConstraint,
    /// Largest monthly instalment the MAS ceilings allow, at the assessed rate.
    pub max_monthly_instalment: Decimal,
    pub assessment_rate: Decimal,
    pub ltv: Decimal,
    pub extended_tenure: bool,
    pub deposit: Decimal,
    /// The share of `deposit` that must be cash rather than CPF, from the
    /// Notice 632 cash column.
    pub min_cash_required: Decimal,
    pub bsd: Decimal,
    pub absd: Decimal,
    /// Cash the buyer actually has to produce: the cash floor, whatever of
    /// the remaining deposit CPF can't cover, and both stamp duties.
    pub cash_required: Decimal,
    /// CPF OA applied to the deposit.
    pub cpf_used: Decimal,
    pub cash_and_cpf_at_completion: Decimal,
    /// Income after the variable-income haircut.
    pub assessed_monthly_income: Decimal,
}

/// The largest loan whose assessed instalment is `instalment`.
///
/// The inverse of [`crate::payment::payment_amount`]: dividing by the payment
/// factor turns a monthly ceiling back into a principal.
fn loan_for_instalment(instalment: Decimal, annual_rate: Decimal, term_years: Decimal) -> Decimal {
    let periods = PaymentFrequency::Monthly.periods_in_years(term_years);
    if periods == 0 {
        return Decimal::ZERO;
    }
    let factor = crate::payment::payment_factor(
        PaymentFrequency::Monthly.periodic_rate(annual_rate),
        periods,
    );
    if factor <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    instalment / factor
}

/// How a purchase at `price` is funded, once CPF is only allowed where the
/// rules actually allow it.
#[derive(Debug, Clone, Copy)]
struct Funding {
    loan: Decimal,
    deposit: Decimal,
    min_cash: Decimal,
    duties: UpfrontCosts,
    cpf_used: Decimal,
    cash_required: Decimal,
}

/// Works out what a purchase at `price` demands in cash versus CPF.
///
/// CPF may only go towards the part of the deposit above the Notice 632 cash
/// floor. It may not cover that floor, and it may not cover stamp duty at
/// completion — both duties fall due within 14 days, faster than CPF Board
/// disburses, so they are paid in cash and reimbursed later if at all.
fn funding_at(
    price: Decimal,
    max_loan_by_income: Decimal,
    limit: LtvLimit,
    cpf_available: Decimal,
    residency: Residency,
    count: PropertyCount,
) -> Funding {
    let loan = (price * limit.ltv).min(max_loan_by_income);
    let deposit = (price - loan).max(Decimal::ZERO);
    let min_cash = price * limit.min_cash;
    let duties = upfront_costs(price, residency, count);

    let cpf_eligible = (deposit - min_cash).max(Decimal::ZERO);
    let cpf_used = cpf_available.max(Decimal::ZERO).min(cpf_eligible);

    Funding {
        loan,
        deposit,
        min_cash,
        duties,
        cpf_used,
        cash_required: deposit - cpf_used + duties.total,
    }
}

/// The most a Singapore buyer can pay for a property, given the MAS
/// servicing ceilings, the Notice 632 LTV limits, and the cash they hold.
///
/// Every one of those can bind, and which one does is reported rather than
/// left implicit. Solved by bisection on price because BSD and ABSD are
/// progressive in the price they're charged on, so the funds a purchase needs
/// aren't a linear function of it — but they are monotonic, which is all
/// bisection requires.
pub fn max_affordable_sg(input: &SgAffordabilityInput) -> MortgageResult<SgAffordabilityResult> {
    let assessed_income = input.income.assessed();
    if assessed_income <= Decimal::ZERO {
        return Err(MortgageError::InvalidIncome(assessed_income.to_string()));
    }

    // 1. What the MAS ceilings allow to be spent on this loan each month,
    //    on income after the variable-income haircut.
    let tdsr_capacity = assessed_income * TDSR_LIMIT - input.other_monthly_debts;
    let msr_capacity = input.is_hdb_or_ec.then(|| assessed_income * MSR_LIMIT);

    let mut binding = BindingConstraint::Tdsr;
    let mut max_instalment = tdsr_capacity;
    if let Some(msr) = msr_capacity {
        if msr < max_instalment {
            max_instalment = msr;
            binding = BindingConstraint::Msr;
        }
    }
    let max_instalment = max_instalment.max(Decimal::ZERO);

    // 2. Turn that into a loan, priced at the assessed rate — the same floor
    //    a bank must apply, so capacity isn't overstated for a cheap quote.
    let assessment_rate =
        assessment_rate(input.thereafter_annual_rate.unwrap_or(input.annual_rate));
    let max_loan_by_income = loan_for_instalment(max_instalment, assessment_rate, input.term_years);

    let limit = ltv_limit(
        input.outstanding_housing_loans,
        input.is_hdb_or_ec,
        input.term_years,
        input.borrower_age,
    );

    // 3. Largest price the buyer can actually complete on. Cash is the
    //    binding resource: CPF helps only with part of the deposit.
    let cash = input.cash_available.max(Decimal::ZERO);
    let cpf = input.cpf_oa_available.max(Decimal::ZERO);
    let mut lo = Decimal::ZERO;
    // Generous ceiling: every purchase needs its deposit and duties covered,
    // so the answer is well inside this.
    let mut hi = (max_loan_by_income + cash + cpf).max(Decimal::ONE) * dec!(2);
    for _ in 0..60 {
        let mid = (lo + hi) / dec!(2);
        let funding = funding_at(
            mid,
            max_loan_by_income,
            limit,
            cpf,
            input.residency,
            input.property_count,
        );
        if funding.cash_required <= cash {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let max_price = round_currency(lo);
    let funding = funding_at(
        max_price,
        max_loan_by_income,
        limit,
        cpf,
        input.residency,
        input.property_count,
    );
    let max_loan = round_currency(funding.loan);

    // 4. Name whichever side is tight. If the loan came to rest on the LTV
    //    ceiling then income had slack and the deposit is what binds;
    //    otherwise the servicing ceiling chosen in step 1 stands.
    if max_loan < max_loan_by_income - dec!(1) {
        binding = BindingConstraint::Ltv;
    }

    Ok(SgAffordabilityResult {
        max_price,
        max_loan,
        binding_constraint: binding,
        max_monthly_instalment: round_currency(max_instalment),
        assessment_rate,
        ltv: limit.ltv,
        extended_tenure: limit.extended_tenure,
        deposit: round_currency(funding.deposit),
        min_cash_required: round_currency(funding.min_cash),
        bsd: funding.duties.bsd,
        absd: funding.duties.absd,
        cash_required: round_currency(funding.cash_required),
        cpf_used: round_currency(funding.cpf_used),
        cash_and_cpf_at_completion: round_currency(funding.deposit + funding.duties.total),
        assessed_monthly_income: round_currency(assessed_income),
    })
}

#[cfg(test)]
mod capacity_tests {
    use super::*;

    fn afford() -> SgAffordabilityInput {
        SgAffordabilityInput {
            income: MonthlyIncome::fixed(dec!(12_000)),
            thereafter_annual_rate: None,
            other_monthly_debts: dec!(0),
            cash_available: dec!(500_000),
            cpf_oa_available: dec!(0),
            annual_rate: dec!(0.04),
            term_years: dec!(25),
            borrower_age: Some(dec!(35)),
            is_hdb_or_ec: false,
            residency: Residency::Citizen,
            property_count: PropertyCount::First,
            outstanding_housing_loans: 0,
        }
    }

    #[test]
    fn first_timer_within_tenure_gets_the_seventy_five_percent_row() {
        let limit = ltv_limit(0, false, dec!(30), Some(dec!(35)));
        assert_eq!(limit.ltv, dec!(0.75));
        assert_eq!(limit.min_cash, dec!(0.05));
        assert!(!limit.extended_tenure);
    }

    #[test]
    fn tenure_past_thirty_years_drops_to_the_extended_row() {
        let limit = ltv_limit(0, false, dec!(31), Some(dec!(30)));
        assert_eq!(limit.ltv, dec!(0.55));
        assert_eq!(limit.min_cash, dec!(0.10));
        assert!(limit.extended_tenure);
    }

    #[test]
    fn an_hdb_flat_hits_the_extended_row_five_years_earlier() {
        // Notice 632 (4D)/(7B) use 25 years where private property uses 30.
        assert_eq!(ltv_limit(0, true, dec!(26), Some(dec!(30))).ltv, dec!(0.55));
        assert_eq!(
            ltv_limit(0, false, dec!(26), Some(dec!(30))).ltv,
            dec!(0.75)
        );
    }

    #[test]
    fn a_loan_running_past_sixty_five_is_extended_even_on_a_short_tenure() {
        // 20-year tenure is well inside the limit, but the borrower is 50, so
        // it matures at 70.
        let limit = ltv_limit(0, false, dec!(20), Some(dec!(50)));
        assert!(limit.extended_tenure);
        assert_eq!(limit.ltv, dec!(0.55));
    }

    #[test]
    fn second_and_third_loans_step_the_ceiling_down() {
        assert_eq!(
            ltv_limit(1, false, dec!(25), Some(dec!(35))).ltv,
            dec!(0.45)
        );
        assert_eq!(
            ltv_limit(2, false, dec!(25), Some(dec!(35))).ltv,
            dec!(0.35)
        );
        assert_eq!(
            ltv_limit(5, false, dec!(25), Some(dec!(35))).ltv,
            dec!(0.35)
        );
        // The cash floor jumps to 25% from the second loan onward.
        assert_eq!(
            ltv_limit(1, false, dec!(25), Some(dec!(35))).min_cash,
            dec!(0.25)
        );
    }

    #[test]
    fn affordability_prices_capacity_at_the_mas_floor_not_a_cheap_quote() {
        let cheap = max_affordable_sg(&SgAffordabilityInput {
            annual_rate: dec!(0.015),
            ..afford()
        })
        .unwrap();
        let at_floor = max_affordable_sg(&afford()).unwrap();

        assert_eq!(cheap.assessment_rate, MEDIUM_TERM_RATE_FLOOR);
        // A 1.5% quote must not buy more house than a 4% one, because the
        // bank assesses both at 4%.
        assert_eq!(cheap.max_loan, at_floor.max_loan);
    }

    #[test]
    fn a_thin_deposit_puts_the_loan_on_the_ltv_ceiling_not_the_income_ceiling() {
        // $12k income services far more than $80k of deposit can support, so
        // the 75% ceiling is what the loan comes to rest on. Earning more
        // would not raise this figure; saving more would.
        let result = max_affordable_sg(&SgAffordabilityInput {
            cash_available: dec!(80_000),
            ..afford()
        })
        .unwrap();
        assert_eq!(result.binding_constraint, BindingConstraint::Ltv);
        assert_eq!(
            result.max_loan,
            round_currency(result.max_price * dec!(0.75))
        );
        // Everything the buyer has goes into the purchase.
        assert!(result.cash_required <= dec!(80_000));
        assert!(result.cash_required > dec!(78_000));
    }

    #[test]
    fn a_deep_deposit_with_modest_income_lands_on_tdsr_instead() {
        // Reverse the squeeze: plenty of deposit, ordinary salary. Now it is
        // servicing capacity that stops the loan growing.
        let result = max_affordable_sg(&SgAffordabilityInput {
            cash_available: dec!(4_000_000),
            ..afford()
        })
        .unwrap();
        assert_eq!(result.binding_constraint, BindingConstraint::Tdsr);
        assert!(result.max_loan < round_currency(result.max_price * dec!(0.75)));
        // 55% of $12,000, with no other debts.
        assert_eq!(result.max_monthly_instalment, dec!(6_600));
    }

    #[test]
    fn msr_binds_before_tdsr_on_an_hdb_flat() {
        let result = max_affordable_sg(&SgAffordabilityInput {
            is_hdb_or_ec: true,
            cash_available: dec!(2_000_000),
            ..afford()
        })
        .unwrap();
        assert_eq!(result.binding_constraint, BindingConstraint::Msr);
        // 30% of $12,000, not 55%.
        assert_eq!(result.max_monthly_instalment, dec!(3_600));
    }

    #[test]
    fn other_debts_come_straight_off_the_tdsr_ceiling() {
        let clean = max_affordable_sg(&SgAffordabilityInput {
            cash_available: dec!(3_000_000),
            ..afford()
        })
        .unwrap();
        let indebted = max_affordable_sg(&SgAffordabilityInput {
            cash_available: dec!(3_000_000),
            other_monthly_debts: dec!(2_000),
            ..afford()
        })
        .unwrap();
        assert_eq!(
            clean.max_monthly_instalment - indebted.max_monthly_instalment,
            dec!(2_000)
        );
        assert!(indebted.max_loan < clean.max_loan);
    }

    #[test]
    fn absd_eats_into_what_a_foreigner_can_complete_on() {
        let citizen = max_affordable_sg(&afford()).unwrap();
        let foreigner = max_affordable_sg(&SgAffordabilityInput {
            residency: Residency::Foreigner,
            ..afford()
        })
        .unwrap();
        // 60% ABSD has to come out of the same funds, so the reachable price
        // collapses.
        assert!(foreigner.max_price < citizen.max_price);
        assert!(foreigner.absd > Decimal::ZERO);
    }

    #[test]
    fn the_answer_is_actually_completable() {
        // The headline figure is worthless if the buyer can't fund it: the
        // deposit plus duties must fit inside what they said they have.
        for funds in [dec!(80_000), dec!(300_000), dec!(900_000)] {
            let r = max_affordable_sg(&SgAffordabilityInput {
                cash_available: funds,
                ..afford()
            })
            .unwrap();
            assert!(
                r.cash_required <= funds + dec!(1),
                "needs {} cash but only {} available",
                r.cash_required,
                funds
            );
            assert!(r.max_loan <= round_currency(r.max_price * r.ltv) + dec!(1));
        }
    }

    #[test]
    fn rejects_non_positive_income_for_affordability() {
        assert!(max_affordable_sg(&SgAffordabilityInput {
            income: MonthlyIncome::fixed(dec!(0)),
            thereafter_annual_rate: None,
            ..afford()
        })
        .is_err());
    }
    // -- Notice 645 para 17: variable income haircut ------------------------

    #[test]
    fn fixed_salary_counts_in_full_and_commission_at_seventy_percent() {
        assert_eq!(MonthlyIncome::fixed(dec!(10_000)).assessed(), dec!(10_000));
        assert_eq!(
            MonthlyIncome {
                fixed: dec!(0),
                variable: dec!(10_000),
            }
            .assessed(),
            dec!(7_000)
        );
        // Para 17(c)(i): the haircut hits only the variable half.
        assert_eq!(
            MonthlyIncome {
                fixed: dec!(6_000),
                variable: dec!(4_000),
            }
            .assessed(),
            dec!(8_800)
        );
    }

    #[test]
    fn a_commission_earner_is_assessed_below_their_headline_pay() {
        // Same $12,000 a month, all commission rather than salary. The
        // borrower feels equally well paid; MAS does not.
        let loan = Loan::builder()
            .principal(dec!(1_000_000))
            .annual_rate(dec!(0.04))
            .term_years(dec!(30))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();
        let salaried =
            check_tdsr_msr(&loan, dec!(0), MonthlyIncome::fixed(dec!(12_000)), false).unwrap();
        let commissioned = check_tdsr_msr(
            &loan,
            dec!(0),
            MonthlyIncome {
                fixed: dec!(0),
                variable: dec!(12_000),
            },
            false,
        )
        .unwrap();

        assert_eq!(salaried.assessed_monthly_income, dec!(12_000));
        assert_eq!(commissioned.assessed_monthly_income, dec!(8_400));
        assert!(commissioned.tdsr.ratio > salaried.tdsr.ratio);
    }

    #[test]
    fn negative_income_components_cannot_inflate_the_assessment() {
        let income = MonthlyIncome {
            fixed: dec!(-5_000),
            variable: dec!(10_000),
        };
        // The negative fixed figure is discarded rather than netted off.
        assert_eq!(income.assessed(), dec!(7_000));
    }

    // -- CPF cannot cover the cash floor or stamp duty ----------------------

    #[test]
    fn cpf_cannot_substitute_for_the_minimum_cash_down_payment() {
        // All CPF, no cash: the buyer cannot complete on anything, because
        // the Notice 632 cash floor and both stamp duties are cash-only.
        let r = max_affordable_sg(&SgAffordabilityInput {
            cash_available: dec!(0),
            cpf_oa_available: dec!(500_000),
            ..afford()
        })
        .unwrap();
        assert_eq!(r.max_price, Decimal::ZERO);
    }

    #[test]
    fn cpf_covers_the_deposit_above_the_cash_floor() {
        let cash_only = max_affordable_sg(&afford()).unwrap();
        let with_cpf = max_affordable_sg(&SgAffordabilityInput {
            cpf_oa_available: dec!(300_000),
            ..afford()
        })
        .unwrap();

        // CPF can't buy the cash floor or the duties, but it can take over
        // the rest of the deposit, freeing cash to reach a higher price.
        assert!(with_cpf.max_price > cash_only.max_price);
        assert!(with_cpf.cpf_used > Decimal::ZERO);
        assert!(with_cpf.cpf_used <= dec!(300_000));
    }

    #[test]
    fn stamp_duty_always_falls_on_cash_even_with_cpf_to_spare() {
        // Both duties are payable within 14 days, which CPF Board cannot
        // disburse against — so they sit in the cash column regardless.
        let r = max_affordable_sg(&SgAffordabilityInput {
            residency: Residency::Foreigner,
            cash_available: dec!(600_000),
            cpf_oa_available: dec!(2_000_000),
            ..afford()
        })
        .unwrap();
        assert!(r.absd > Decimal::ZERO);
        assert!(
            r.cash_required >= r.bsd + r.absd,
            "cash {} should cover at least the duties {}",
            r.cash_required,
            r.bsd + r.absd
        );
    }

    #[test]
    fn the_cash_floor_is_never_met_from_cpf() {
        for cpf in [dec!(0), dec!(200_000), dec!(1_000_000)] {
            let r = max_affordable_sg(&SgAffordabilityInput {
                cpf_oa_available: cpf,
                ..afford()
            })
            .unwrap();
            // Whatever CPF does, cash still has to cover the floor plus duties.
            assert!(
                r.cash_required >= r.min_cash_required + r.bsd + r.absd - dec!(1),
                "cpf {cpf}: cash {} below floor {} + duties {}",
                r.cash_required,
                r.min_cash_required,
                r.bsd + r.absd
            );
            assert!(r.cpf_used <= (r.deposit - r.min_cash_required).max(Decimal::ZERO) + dec!(1));
        }
    }
    // -- FTA National Treatment --------------------------------------------

    #[test]
    fn an_fta_national_pays_citizen_absd_rates_not_the_foreigner_flat_rate() {
        let price = dec!(2_000_000);
        // A US citizen's first home: 0%, where a plain foreigner owes 60%.
        assert_eq!(
            additional_buyers_stamp_duty(price, Residency::FtaNational, PropertyCount::First),
            Decimal::ZERO
        );
        assert_eq!(
            additional_buyers_stamp_duty(price, Residency::Foreigner, PropertyCount::First),
            dec!(1_200_000)
        );
    }

    #[test]
    fn fta_treatment_tracks_citizen_rates_all_the_way_up() {
        for count in [
            PropertyCount::First,
            PropertyCount::Second,
            PropertyCount::ThirdOrMore,
        ] {
            assert_eq!(
                additional_buyers_stamp_duty(dec!(1_500_000), Residency::FtaNational, count),
                additional_buyers_stamp_duty(dec!(1_500_000), Residency::Citizen, count),
                "FTA and citizen rates diverged at {count:?}"
            );
        }
    }

    #[test]
    fn fta_status_changes_what_a_buyer_can_reach_but_not_their_ltv() {
        // Stamp duty relief only. Notice 632 keys LTV off outstanding loans,
        // never residency, so the ceiling is untouched.
        let foreigner = max_affordable_sg(&SgAffordabilityInput {
            residency: Residency::Foreigner,
            ..afford()
        })
        .unwrap();
        let fta = max_affordable_sg(&SgAffordabilityInput {
            residency: Residency::FtaNational,
            ..afford()
        })
        .unwrap();

        assert_eq!(fta.absd, Decimal::ZERO);
        assert_eq!(fta.ltv, foreigner.ltv);
        assert!(fta.max_price > foreigner.max_price);
    }
}
