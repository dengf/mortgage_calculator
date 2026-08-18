//! Maximum affordable home price given income, debts, and loan terms.
//!
//! This is the calculator-shaped equivalent of `convex`'s "Hedge Advisor":
//! not a bond/loan analysis of a *given* instrument, but a decision-support
//! layer that works backwards from a constraint (debt-to-income ratio) to a
//! recommended input (home price).

use mortgage_core::{round_currency, MortgageError, MortgageResult, PaymentFrequency};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::payment::payment_factor;

/// Inputs to the affordability calculation. Property tax is expressed as an
/// annual rate of home price (e.g. `0.012` for 1.2%/yr) since it scales
/// with the price being solved for; insurance and HOA are flat annual/
/// monthly dollar estimates since they don't.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AffordabilityInput {
    pub gross_monthly_income: Decimal,
    pub monthly_debts: Decimal,
    pub down_payment: Decimal,
    pub annual_rate: Decimal,
    pub term_years: Decimal,
    /// Back-end debt-to-income ceiling, e.g. `0.36` for the classic 36% rule.
    pub max_dti: Decimal,
    pub annual_property_tax_rate: Decimal,
    pub annual_insurance: Decimal,
    pub monthly_hoa: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AffordabilityResult {
    pub max_monthly_housing_payment: Decimal,
    pub max_principal_and_interest: Decimal,
    pub max_loan_amount: Decimal,
    pub max_home_price: Decimal,
    pub front_end_dti: Decimal,
    pub back_end_dti: Decimal,
}

/// Solves for the maximum home price a borrower can afford under `input`.
///
/// The housing budget (income * max_dti - existing debts) has to cover
/// principal & interest *and* property tax *and* insurance/HOA, and
/// property tax itself scales with the home price being solved for. That
/// makes this a linear equation in `loan_amount` rather than a simple
/// division — see the derivation in the loan_amount computation below.
pub fn max_affordable(input: &AffordabilityInput) -> MortgageResult<AffordabilityResult> {
    if input.gross_monthly_income <= Decimal::ZERO {
        return Err(MortgageError::InvalidIncome(
            input.gross_monthly_income.to_string(),
        ));
    }
    if input.max_dti <= Decimal::ZERO || input.max_dti >= Decimal::ONE {
        return Err(MortgageError::InvalidDtiRatio(input.max_dti.to_string()));
    }
    if input.annual_rate < Decimal::ZERO {
        return Err(MortgageError::InvalidRate(input.annual_rate.to_string()));
    }

    let frequency = PaymentFrequency::Monthly;
    let periodic_rate = frequency.periodic_rate(input.annual_rate);
    let total_periods = frequency.periods_in_years(input.term_years);
    if total_periods == 0 {
        return Err(MortgageError::InvalidTerm(total_periods));
    }
    let k = payment_factor(periodic_rate, total_periods);

    let max_total_debt_payment = input.gross_monthly_income * input.max_dti;
    let max_monthly_housing_payment =
        (max_total_debt_payment - input.monthly_debts).max(Decimal::ZERO);

    let monthly_insurance = input.annual_insurance / Decimal::from(12);
    let monthly_tax_rate = input.annual_property_tax_rate / Decimal::from(12);

    // max_housing_payment = loan*k + (loan + down_payment)*monthly_tax_rate
    //                        + monthly_insurance + monthly_hoa
    // => loan * (k + monthly_tax_rate) =
    //      max_housing_payment - monthly_insurance - monthly_hoa - down_payment*monthly_tax_rate
    let budget_for_loan_and_tax = max_monthly_housing_payment
        - monthly_insurance
        - input.monthly_hoa
        - input.down_payment * monthly_tax_rate;

    let max_loan_amount = round_currency((budget_for_loan_and_tax / (k + monthly_tax_rate)).max(Decimal::ZERO));
    let max_home_price = round_currency(max_loan_amount + input.down_payment);
    let max_principal_and_interest = round_currency(max_loan_amount * k);

    let monthly_property_tax = round_currency(max_home_price * monthly_tax_rate);
    let total_housing_cost = round_currency(
        max_principal_and_interest + monthly_property_tax + monthly_insurance + input.monthly_hoa,
    );

    let front_end_dti = total_housing_cost / input.gross_monthly_income;
    let back_end_dti = (total_housing_cost + input.monthly_debts) / input.gross_monthly_income;

    Ok(AffordabilityResult {
        max_monthly_housing_payment: round_currency(max_monthly_housing_payment),
        max_principal_and_interest,
        max_loan_amount,
        max_home_price,
        front_end_dti,
        back_end_dti,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base_input() -> AffordabilityInput {
        AffordabilityInput {
            gross_monthly_income: dec!(10000),
            monthly_debts: dec!(500),
            down_payment: dec!(60000),
            annual_rate: dec!(0.065),
            term_years: dec!(30),
            max_dti: dec!(0.36),
            annual_property_tax_rate: dec!(0.012),
            annual_insurance: dec!(1500),
            monthly_hoa: dec!(0),
        }
    }

    #[test]
    fn back_end_dti_never_exceeds_the_cap() {
        let result = max_affordable(&base_input()).unwrap();
        assert!(result.back_end_dti <= dec!(0.36) + dec!(0.001));
        assert!(result.max_home_price > result.max_loan_amount);
    }

    #[test]
    fn higher_income_affords_more_home() {
        let mut richer = base_input();
        richer.gross_monthly_income = dec!(20000);

        let base_result = max_affordable(&base_input()).unwrap();
        let richer_result = max_affordable(&richer).unwrap();
        assert!(richer_result.max_home_price > base_result.max_home_price);
    }

    #[test]
    fn rejects_dti_ratio_out_of_range() {
        let mut input = base_input();
        input.max_dti = dec!(1.2);
        assert!(max_affordable(&input).is_err());
    }
}
