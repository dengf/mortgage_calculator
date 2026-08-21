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

use mortgage_core::{round_currency, MortgageError, MortgageResult};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

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
}

/// Checks a loan's monthly payment against the MAS TDSR (always) and MSR
/// (HDB/EC only) ceilings.
pub fn check_tdsr_msr(
    monthly_housing_payment: Decimal,
    other_monthly_debts: Decimal,
    gross_monthly_income: Decimal,
    is_hdb_or_ec: bool,
) -> MortgageResult<TdsrMsrCheck> {
    if gross_monthly_income <= Decimal::ZERO {
        return Err(MortgageError::InvalidIncome(
            gross_monthly_income.to_string(),
        ));
    }

    let tdsr_ratio = (monthly_housing_payment + other_monthly_debts) / gross_monthly_income;
    let msr = is_hdb_or_ec
        .then(|| limit_status(monthly_housing_payment / gross_monthly_income, MSR_LIMIT));

    Ok(TdsrMsrCheck {
        tdsr: limit_status(tdsr_ratio, TDSR_LIMIT),
        msr,
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

    #[test]
    fn tdsr_flags_when_total_debt_exceeds_55_percent() {
        let check = check_tdsr_msr(dec!(5000), dec!(1000), dec!(10000), false).unwrap();
        assert!(check.tdsr.exceeded);
        assert!(check.msr.is_none());
    }

    #[test]
    fn msr_only_applies_to_hdb_and_is_tighter_than_tdsr() {
        let check = check_tdsr_msr(dec!(3200), dec!(0), dec!(10000), true).unwrap();
        assert!(!check.tdsr.exceeded);
        assert!(check.msr.unwrap().exceeded);
    }

    #[test]
    fn rejects_non_positive_income() {
        assert!(check_tdsr_msr(dec!(1000), dec!(0), dec!(0), false).is_err());
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
