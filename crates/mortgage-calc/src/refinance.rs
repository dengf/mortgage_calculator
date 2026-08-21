//! Refinance break-even and lifetime interest comparison.

use mortgage_core::{round_currency, MortgageError, MortgageResult, PaymentFrequency};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::payment::payment_factor;
use crate::Loan;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RefinanceInput {
    pub current_balance: Decimal,
    pub current_annual_rate: Decimal,
    /// Payments remaining on the current loan at its existing frequency.
    pub remaining_periods: u32,
    pub new_annual_rate: Decimal,
    pub new_term_years: Decimal,
    pub closing_costs: Decimal,
    pub frequency: PaymentFrequency,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RefinanceResult {
    pub current_payment: Decimal,
    pub new_payment: Decimal,
    pub payment_savings: Decimal,
    /// `None` when the new payment is not lower, so closing costs are
    /// never recouped.
    pub break_even_periods: Option<u32>,
    pub remaining_interest_on_current_loan: Decimal,
    pub total_interest_on_new_loan: Decimal,
    /// Interest saved by switching, net of closing costs. Compares total
    /// remaining cost of the current loan against total cost of the new
    /// loan; a shorter new term is credited for the periods it no longer
    /// needs to pay interest in at all.
    pub lifetime_savings: Decimal,
}

/// Compares staying on the current loan against refinancing into a new one.
pub fn analyze_refinance(input: &RefinanceInput) -> MortgageResult<RefinanceResult> {
    if input.remaining_periods == 0 {
        return Err(MortgageError::InvalidTerm(0));
    }

    let current_periodic_rate = input.frequency.periodic_rate(input.current_annual_rate);
    let current_factor = payment_factor(current_periodic_rate, input.remaining_periods);
    let current_payment = round_currency(input.current_balance * current_factor);

    let new_loan = Loan::builder()
        .principal(input.current_balance)
        .annual_rate(input.new_annual_rate)
        .term_years(input.new_term_years)
        .frequency(input.frequency)
        .build()?;
    let new_payment = crate::payment::payment_amount(&new_loan);
    let new_total_periods = new_loan.total_periods();

    let payment_savings = round_currency(current_payment - new_payment);

    let break_even_periods = if payment_savings > Decimal::ZERO {
        (input.closing_costs / payment_savings)
            .ceil()
            .to_u32()
            .filter(|periods| *periods > 0)
            .or(Some(1))
    } else {
        None
    };

    let remaining_interest_on_current_loan = round_currency(
        current_payment * Decimal::from(input.remaining_periods) - input.current_balance,
    );
    let total_interest_on_new_loan =
        round_currency(new_payment * Decimal::from(new_total_periods) - input.current_balance);

    let lifetime_savings = round_currency(
        remaining_interest_on_current_loan - total_interest_on_new_loan - input.closing_costs,
    );

    Ok(RefinanceResult {
        current_payment,
        new_payment,
        payment_savings,
        break_even_periods,
        remaining_interest_on_current_loan,
        total_interest_on_new_loan,
        lifetime_savings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn lower_rate_produces_positive_savings_and_break_even() {
        let input = RefinanceInput {
            current_balance: dec!(300000),
            current_annual_rate: dec!(0.075),
            remaining_periods: 300,
            new_annual_rate: dec!(0.055),
            new_term_years: dec!(30),
            closing_costs: dec!(6000),
            frequency: PaymentFrequency::Monthly,
        };

        let result = analyze_refinance(&input).unwrap();
        assert!(result.payment_savings > Decimal::ZERO);
        assert!(result.break_even_periods.is_some());
        assert!(result.break_even_periods.unwrap() > 0);
    }

    #[test]
    fn higher_rate_has_no_break_even() {
        let input = RefinanceInput {
            current_balance: dec!(300000),
            current_annual_rate: dec!(0.045),
            remaining_periods: 300,
            new_annual_rate: dec!(0.07),
            new_term_years: dec!(30),
            closing_costs: dec!(6000),
            frequency: PaymentFrequency::Monthly,
        };

        let result = analyze_refinance(&input).unwrap();
        assert!(result.payment_savings < Decimal::ZERO);
        assert_eq!(result.break_even_periods, None);
    }

    #[test]
    fn rejects_zero_remaining_periods_instead_of_panicking() {
        let input = RefinanceInput {
            current_balance: dec!(300000),
            current_annual_rate: dec!(0.06),
            remaining_periods: 0,
            new_annual_rate: dec!(0.05),
            new_term_years: dec!(30),
            closing_costs: dec!(6000),
            frequency: PaymentFrequency::Monthly,
        };

        assert!(matches!(
            analyze_refinance(&input),
            Err(MortgageError::InvalidTerm(0))
        ));
    }
}
