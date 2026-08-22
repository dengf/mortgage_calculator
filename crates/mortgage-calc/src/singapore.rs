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

/// The rate a bank must assess a residential property loan at: the
/// borrower's own rate, or the MAS floor, whichever is higher.
pub fn assessment_rate(contract_annual_rate: Decimal) -> Decimal {
    contract_annual_rate.max(MEDIUM_TERM_RATE_FLOOR)
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
}

/// Checks a loan against the MAS TDSR (always) and MSR (HDB/EC only)
/// ceilings.
///
/// Takes the loan rather than a payment on purpose. The ratios must be
/// computed on the instalment at [`MEDIUM_TERM_RATE_FLOOR`], not the one the
/// borrower is quoted, and a caller handed a payment has no way to tell
/// which it holds — so the repricing happens here, where it can't be
/// skipped.
pub fn check_tdsr_msr(
    loan: &Loan,
    other_monthly_debts: Decimal,
    gross_monthly_income: Decimal,
    is_hdb_or_ec: bool,
) -> MortgageResult<TdsrMsrCheck> {
    if gross_monthly_income <= Decimal::ZERO {
        return Err(MortgageError::InvalidIncome(
            gross_monthly_income.to_string(),
        ));
    }

    let assessment_rate = assessment_rate(loan.annual_rate());
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
    })
}

/// Buyer's residency status, which sets the ABSD rate alongside property count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Residency {
    Citizen,
    PermanentResident,
    Foreigner,
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
            dec!(10_000),
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
            dec!(10_000),
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
            dec!(0),
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
        let check = check_tdsr_msr(&loan, dec!(0), dec!(10_000), false).unwrap();

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
        let check = check_tdsr_msr(&loan, dec!(0), dec!(20_000), false).unwrap();

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
