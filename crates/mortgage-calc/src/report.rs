//! The content of a client-facing loan illustration.
//!
//! This module decides *what a report says*: which figures appear, how the
//! life of the loan is divided, and which stress cases have to be shown. It
//! decides nothing about how any of that looks. Labels are the front end's
//! job, and so is page layout -- what leaves here is figures and structure.
//!
//! # Why the structure is the domain's and not the renderer's
//!
//! Both regulators that matter here reached the same conclusion about how a
//! loan should be presented, and neither reached it by accident.
//!
//! The CFPB's Loan Estimate (TILA-RESPA) puts a column headed "Can this
//! amount increase after closing?" against the interest rate and the monthly
//! payment, and then breaks projected payments into *year bands* -- an
//! adjustable sample reads "Years 1-5 | Years 6-8 | Years 9-11 | Years
//! 12-30", each band carrying its own instalment. A single headline payment
//! is not considered an adequate description of a loan whose payment moves.
//!
//! MAS Notice 632A requires a bank to hand over a Residential Property Loan
//! Fact Sheet, and MoneySense -- MAS's own public programme -- lists what it
//! must carry: loan amount and tenure, total repayment amount, lock-in
//! period, interest rate and repayment schedule, a **rate change
//! illustration** showing how possible increases in interest rates will
//! affect the monthly instalment, the effective interest rate, and penalty
//! fees.
//!
//! Both, in other words, insist that a loan is described over time and
//! stressed against a rise. That is a statement about mortgages, not about
//! documents, which is why it is encoded here rather than in a template.
//!
//! # What this is not
//!
//! It is not a Loan Estimate and not a Fact Sheet. Those are regulated
//! disclosures a *lender* issues about an offer it is making. This is a
//! planning illustration produced by a calculator, and the front end is
//! responsible for saying so on the document.

use mortgage_core::{round_currency, MortgageResult, PaymentFrequency};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::amortization::AmortizationYear;
use crate::payment::{payment_factor, summarize};
use crate::Loan;

/// A stretch of the loan over which the instalment does not change.
///
/// A flat loan has one. A package that steps up has two: the lock-in, and
/// everything after it. This is the CFPB's year-band idea, and it exists
/// because the alternative -- one headline figure -- describes the first
/// two years of a twenty-five year loan and nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PaymentBand {
    /// First and last year of the band, counting from 1 and inclusive.
    pub from_year: Decimal,
    pub to_year: Decimal,
    pub annual_rate: Decimal,
    pub payment: Decimal,
}

/// One line of the rate-change illustration MAS requires.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RateRiseRow {
    /// Percentage points added to the rate that lasts, e.g. `0.01` for +1%.
    pub increase: Decimal,
    pub annual_rate: Decimal,
    pub payment: Decimal,
    /// How much more per period than the loan's own final instalment.
    pub payment_increase: Decimal,
}

/// Everything a loan illustration states, as figures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub principal: Decimal,
    pub term_years: Decimal,
    pub frequency: PaymentFrequency,
    /// The rate and instalment the loan opens on.
    pub initial_annual_rate: Decimal,
    pub initial_payment: Decimal,
    /// The rate the loan spends most of its life at -- equal to
    /// `initial_annual_rate` unless it steps up.
    pub final_annual_rate: Decimal,
    /// `None` when the instalment never changes.
    pub payment_after_reversion: Option<Decimal>,
    /// Years the opening instalment holds. `None` when it holds throughout.
    pub lock_in_years: Option<Decimal>,
    pub total_paid: Decimal,
    pub total_interest: Decimal,
    /// Interest as a percentage of everything paid. `None` when nothing is
    /// paid -- a share of zero is undefined, not zero percent.
    pub interest_share: Option<Decimal>,
    pub bands: Vec<PaymentBand>,
    pub rate_rise: Vec<RateRiseRow>,
    pub yearly: Vec<AmortizationYear>,
}

/// Increases the illustration is run at, in percentage points.
///
/// Chosen to bracket the ceiling a Singapore bank underwrites against: MAS
/// Notice 645's medium-term rate floor is 4%, and on a package quoted near
/// 1.7% thereafter, +2% lands either side of it. A borrower who can service
/// the +2% line is inside the assessment their own bank will run.
const RATE_RISES: [Decimal; 3] = [dec!(0.005), dec!(0.01), dec!(0.02)];

/// Builds the illustration for `loan`.
pub fn report(loan: &Loan) -> MortgageResult<Report> {
    let summary = summarize(loan);
    // One schedule, read several ways. The document prints the yearly rows
    // and the totals beside them, and stresses the balance left at the
    // step-up -- all three have to come from the same calculation or the
    // page can contradict itself.
    let rows = crate::amortization::schedule(loan, Decimal::ZERO)?;
    let yearly = crate::amortization::summarize_by_year(&rows, loan.frequency().periods_per_year());

    Ok(Report {
        principal: loan.principal(),
        term_years: loan.term_years(),
        frequency: loan.frequency(),
        initial_annual_rate: loan.annual_rate(),
        initial_payment: summary.payment,
        final_annual_rate: loan.final_annual_rate(),
        payment_after_reversion: summary.payment_after_reversion,
        lock_in_years: loan.reversion().map(|_| reversion_years(loan)),
        total_paid: summary.total_paid,
        total_interest: summary.total_interest,
        interest_share: summary.interest_share(),
        bands: bands(loan, &summary),
        rate_rise: rate_rise(loan, &rows),
        yearly,
    })
}

/// How long the opening instalment holds, in years.
fn reversion_years(loan: &Loan) -> Decimal {
    loan.reversion()
        .map(|r| {
            Decimal::from(r.after_periods) / Decimal::from(loan.frequency().periods_per_year())
        })
        .unwrap_or(loan.term_years())
}

/// The stretches the instalment is constant over.
fn bands(loan: &Loan, summary: &crate::payment::PaymentSummary) -> Vec<PaymentBand> {
    let whole_term = PaymentBand {
        from_year: Decimal::ONE,
        to_year: loan.term_years(),
        annual_rate: loan.annual_rate(),
        payment: summary.payment,
    };

    match (summary.payment_after_reversion, loan.reversion()) {
        (Some(after), Some(reversion)) => {
            let lock_in = reversion_years(loan);
            vec![
                PaymentBand {
                    to_year: lock_in,
                    ..whole_term
                },
                PaymentBand {
                    from_year: lock_in + Decimal::ONE,
                    to_year: loan.term_years(),
                    annual_rate: reversion.annual_rate,
                    payment: after,
                },
            ]
        }
        _ => vec![whole_term],
    }
}

/// What the instalment becomes if rates rise.
///
/// Run against the rate the loan *lasts* at, not the one it opens on: a
/// promotional spread is contractual for its lock-in, so a rise during those
/// two years does not touch the instalment. The question the illustration
/// answers is what happens when the loan is sitting on its thereafter rate
/// and the benchmark moves -- which is where it spends most of its life.
///
/// Each line re-amortizes the balance outstanding when the step-up happens
/// over the term that is left, the way a bank reprices at reset.
fn rate_rise(loan: &Loan, rows: &[crate::amortization::AmortizationRow]) -> Vec<RateRiseRow> {
    let (balance, remaining_periods) = at_reversion(loan, rows);
    if remaining_periods == 0 {
        return Vec::new();
    }

    let base = loan.final_annual_rate();
    let base_payment = instalment(loan, balance, base, remaining_periods);

    RATE_RISES
        .iter()
        .map(|increase| {
            let annual_rate = base + *increase;
            let payment = instalment(loan, balance, annual_rate, remaining_periods);
            RateRiseRow {
                increase: *increase,
                annual_rate,
                payment,
                payment_increase: round_currency(payment - base_payment),
            }
        })
        .collect()
}

/// The balance still owed when the loan reaches its final rate, and the
/// number of payments left at that point. For a flat loan that is the whole
/// loan from period one.
fn at_reversion(loan: &Loan, rows: &[crate::amortization::AmortizationRow]) -> (Decimal, u32) {
    let total = loan.total_periods();
    let Some(reversion) = loan.reversion() else {
        return (loan.principal(), total);
    };

    let balance = rows
        .get(reversion.after_periods as usize - 1)
        .map(|row| row.remaining_balance)
        .unwrap_or(loan.principal());

    (balance, total.saturating_sub(reversion.after_periods))
}

fn instalment(loan: &Loan, balance: Decimal, annual_rate: Decimal, periods: u32) -> Decimal {
    let periodic = loan.frequency().periodic_rate(annual_rate);
    round_currency(balance * payment_factor(periodic, periods))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RateType;

    fn flat() -> Loan {
        Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.065))
            .term_years(dec!(30))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap()
    }

    /// The shape every Singapore home loan takes.
    fn package() -> Loan {
        Loan::builder()
            .principal(dec!(400000))
            .rate_type(RateType::Reverting {
                base_rate: dec!(0.0112),
                initial_spread: dec!(0.003),
                initial_years: dec!(2),
                thereafter_spread: dec!(0.006),
            })
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap()
    }

    #[test]
    fn a_flat_loan_is_one_band_over_the_whole_term() {
        let bands = report(&flat()).unwrap().bands;
        assert_eq!(bands.len(), 1);
        assert_eq!(bands[0].from_year, dec!(1));
        assert_eq!(bands[0].to_year, dec!(30));
    }

    #[test]
    fn a_package_is_split_at_the_lock_in_with_no_year_lost_between_them() {
        // The bands are read as "Years 1-2" and "Years 3-25". An off-by-one
        // here silently drops or double-counts a year of the loan.
        let bands = report(&package()).unwrap().bands;
        assert_eq!(bands.len(), 2);
        assert_eq!((bands[0].from_year, bands[0].to_year), (dec!(1), dec!(2)));
        assert_eq!((bands[1].from_year, bands[1].to_year), (dec!(3), dec!(25)));
        assert!(bands[1].payment > bands[0].payment);
        assert!(bands[1].annual_rate > bands[0].annual_rate);
    }

    #[test]
    fn a_rise_is_measured_from_the_rate_that_lasts_not_the_teaser() {
        // A promotional spread is contractual for its lock-in: a rise in the
        // benchmark during those years does not move the instalment. Running
        // the illustration off the opening rate would understate every line.
        let report = report(&package()).unwrap();
        let first = report.rate_rise.first().unwrap();

        assert_eq!(first.annual_rate, report.final_annual_rate + dec!(0.005));
        assert!(first.annual_rate > report.initial_annual_rate + dec!(0.005));
    }

    #[test]
    fn each_rise_costs_more_than_the_one_below_it() {
        let rows = report(&package()).unwrap().rate_rise;
        assert_eq!(rows.len(), 3);
        for pair in rows.windows(2) {
            assert!(
                pair[1].payment > pair[0].payment,
                "{:?} should cost more than {:?}",
                pair[1],
                pair[0]
            );
        }
        assert!(rows[0].payment_increase > Decimal::ZERO);
    }

    #[test]
    fn a_rise_reprices_the_balance_left_at_the_step_up_not_the_whole_loan() {
        // Two years of principal have been paid off by the time the rate
        // moves. Repricing the original amount over the original term would
        // overstate what a rise actually costs.
        let report = report(&package()).unwrap();
        let unchanged = report
            .rate_rise
            .iter()
            .map(|row| row.payment - row.payment_increase)
            .next()
            .unwrap();

        assert_eq!(unchanged, report.payment_after_reversion.unwrap());
    }

    #[test]
    fn a_flat_loan_is_still_stressed() {
        // MoneySense lists the rate-change illustration as a fact sheet
        // requirement without qualification, and a US ARM is quoted flat
        // here too. "Fixed today" is not "fixed forever".
        let rows = report(&flat()).unwrap().rate_rise;
        assert_eq!(rows.len(), 3);
        assert!(rows[0].payment > crate::payment::payment_amount(&flat()));
    }

    #[test]
    fn the_yearly_schedule_covers_the_whole_term() {
        let report = report(&package()).unwrap();
        assert_eq!(report.yearly.len(), 25);
        assert_eq!(
            report.yearly.last().unwrap().remaining_balance,
            Decimal::ZERO
        );
    }

    #[test]
    fn the_totals_match_the_schedule_they_are_shown_beside() {
        // The document prints both. Two routes to one figure is how a
        // summary ends up disagreeing with the rows under it.
        let report = report(&package()).unwrap();
        let from_rows: Decimal = report.yearly.iter().map(|y| y.interest).sum();

        assert_eq!(round_currency(from_rows), report.total_interest);
    }
}
