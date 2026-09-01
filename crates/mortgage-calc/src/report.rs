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

use mortgage_core::{round_currency, MortgageResult, PaymentFrequency, Region};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::amortization::{AmortizationRow, AmortizationYear};
use crate::payment::{payment_factor, summarize};
use crate::Loan;

/// A published authority the figures in a report rest on.
///
/// Carried by the report rather than written into the template, because
/// which authorities apply is a fact about the market the loan is in. A
/// document that states a 55% servicing ceiling and a 75% LTV cap without
/// saying who set them is asking to be taken as the lender's own word.
///
/// The doc comments through this crate cite these same authorities against
/// the individual rules. This list is the machine-readable half, and it is
/// the one the reader sees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Authority {
    /// LTV ceilings, the cash component of the deposit, and the tenure
    /// thresholds that tighten them.
    MasNotice632,
    /// The Residential Property Loan Fact Sheet a bank must give a
    /// borrower. This illustration follows its shape and is not one.
    MasNotice632a,
    /// TDSR and MSR, and the medium-term rate floor servicing is assessed
    /// at.
    MasNotice645,
    /// SORA, the benchmark Singapore packages are quoted over.
    MasSora,
    /// Buyer's Stamp Duty and Additional Buyer's Stamp Duty.
    Iras,
    /// CPF Ordinary Account interest, and the HDB concessionary rate pegged
    /// above it.
    CpfBoard,
    /// The Loan Estimate this illustration follows the shape of, and is
    /// not.
    Cfpb,
    /// The conforming loan limit that separates conforming from jumbo.
    Fhfa,
    /// The published Prime and SOFR benchmarks.
    FederalReserveH15,
}

impl Authority {
    /// Where a reader checks it.
    ///
    /// A citation nobody can follow is decoration. These are the landing
    /// pages rather than deep links to a particular PDF, which move.
    pub fn url(self) -> &'static str {
        match self {
            Authority::MasNotice632 => "https://www.mas.gov.sg/regulation/notices/notice-632",
            Authority::MasNotice632a => "https://www.mas.gov.sg/regulation/notices/notice-632a",
            Authority::MasNotice645 => "https://www.mas.gov.sg/regulation/notices/notice-645",
            Authority::MasSora => "https://www.mas.gov.sg/monetary-policy/sora",
            Authority::Iras => "https://www.iras.gov.sg/taxes/stamp-duty/for-property",
            Authority::CpfBoard => {
                "https://www.cpf.gov.sg/member/growing-your-savings/earning-attractive-interest"
            }
            Authority::Cfpb => "https://www.consumerfinance.gov/owning-a-home/loan-estimate/",
            Authority::Fhfa => "https://www.fhfa.gov/policy/conforming-loan-limit",
            Authority::FederalReserveH15 => "https://www.federalreserve.gov/releases/h15/",
        }
    }
}

/// The authorities a report for `region` rests on, in the order a reader
/// meets them: the loan first, then what it is assessed against, then the
/// duties and benchmarks around it.
pub fn references(region: Region) -> Vec<Authority> {
    match region {
        Region::SG => vec![
            Authority::MasNotice632a,
            Authority::MasNotice645,
            Authority::MasNotice632,
            Authority::Iras,
            Authority::CpfBoard,
            Authority::MasSora,
        ],
        Region::US => vec![
            Authority::Cfpb,
            Authority::Fhfa,
            Authority::FederalReserveH15,
        ],
    }
}

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

/// One scheduled payment, as a document states it.
///
/// Carries its own year rather than leaving the reader to divide. On a
/// weekly loan, payment 847 is in year 17 and nobody works that out while
/// reading; on a bi-weekly one the divisor is 26, which nobody guesses. The
/// arithmetic is one line and it belongs on this side of the boundary with
/// the cadence that determines it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SchedulePayment {
    /// Payment number, counting from 1.
    pub period: u32,
    /// Which year of the loan it falls in, counting from 1.
    pub year: u32,
    pub paid: Decimal,
    pub principal: Decimal,
    pub interest: Decimal,
    pub remaining_balance: Decimal,
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
    /// The benchmark every figure above was computed at, when the quote
    /// rests on one that moves. `None` when the loan's rates are
    /// contractual for the term.
    ///
    /// The document is required to print this. Not as a hedge: the bands,
    /// the totals and the schedule are all exact *given* this number, and a
    /// reader who is not told it was held still has been shown a
    /// projection dressed as a quotation. MAS makes a bank say the same
    /// thing on a Notice 632A fact sheet, whose rate-change illustration is
    /// published alongside the admission that the reference rate may move
    /// further than the example shows.
    pub floating_base: Option<Decimal>,
    pub rate_rise: Vec<RateRiseRow>,
    /// Every scheduled payment, one row each.
    ///
    /// Not a yearly summary. The document is read by someone deciding
    /// whether to sign, and the question they ask of a schedule -- what do
    /// I owe after the lock-in, what does the first payment after the
    /// step-up look like -- is a question about a payment, not about a
    /// calendar year. A yearly roll-up also hides the step-up entirely: on
    /// a package reverting at 24 months, the year-2 row averages the two
    /// instalments and shows a figure the borrower never pays.
    ///
    /// Length follows the cadence: 300 rows for a 25-year monthly loan,
    /// 1,300 for a weekly one.
    pub schedule: Vec<SchedulePayment>,
    /// The same payments rolled up by year.
    ///
    /// Both are built, because which one belongs in the document is the
    /// reader's call rather than this module's: a borrower checking the
    /// payment their step-up lands on needs every row, and one sending a
    /// twenty-five line summary to a spouse does not need thirty pages.
    /// Deriving them separately in the front end would be two answers to
    /// one question -- they are cut here, from one schedule.
    pub yearly: Vec<AmortizationYear>,
    /// Who set the rules the figures follow, so the document cites its
    /// sources rather than asserting them.
    pub references: Vec<Authority>,
}

/// Increases the illustration is run at, in percentage points.
///
/// Chosen to bracket the ceiling a Singapore bank underwrites against: MAS
/// Notice 645's medium-term rate floor is 4%, and on a package quoted near
/// 1.7% thereafter, +2% lands either side of it. A borrower who can service
/// the +2% line is inside the assessment their own bank will run.
const RATE_RISES: [Decimal; 3] = [dec!(0.005), dec!(0.01), dec!(0.02)];

/// Builds the illustration for `loan` as it would be read in `region`.
pub fn report(loan: &Loan, region: Region) -> MortgageResult<Report> {
    let summary = summarize(loan);
    // One schedule, read several ways. The document prints every row and
    // the totals beside them, and stresses the balance left at the step-up
    // -- all of it has to come from the same calculation or the page can
    // contradict itself.
    let rows = crate::amortization::schedule(loan, Decimal::ZERO)?;
    let periods_per_year = loan.frequency().periods_per_year();

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
        floating_base: loan.floating_base(),
        rate_rise: rate_rise(loan, &rows),
        yearly: crate::amortization::summarize_by_year(&rows, periods_per_year),
        schedule: payments(&rows, periods_per_year),
        references: references(region),
    })
}

/// Every payment, each told which year it falls in.
fn payments(rows: &[AmortizationRow], periods_per_year: u32) -> Vec<SchedulePayment> {
    rows.iter()
        .map(|row| SchedulePayment {
            period: row.period,
            // Payments 1..=periods_per_year are year 1, so the division is
            // on `period - 1`. Off by one here would put the first payment
            // of every year in the year before it.
            year: (row.period - 1).checked_div(periods_per_year).unwrap_or(0) + 1,
            paid: row.payment,
            principal: row.principal_portion,
            interest: row.interest_portion,
            remaining_balance: row.remaining_balance,
        })
        .collect()
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

    // `after_periods` payments have been made by the time the reversion
    // takes effect. For `after_periods == 0` that's none at all -- the
    // balance is still the untouched principal -- and indexing
    // `rows[after_periods - 1]` would underflow, so that case is handled
    // separately rather than folded into the lookup below.
    let balance = if reversion.after_periods == 0 {
        loan.principal()
    } else {
        rows.get(reversion.after_periods as usize - 1)
            .map(|row| row.remaining_balance)
            .unwrap_or(loan.principal())
    };

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

    #[test]
    fn a_package_report_states_the_benchmark_it_was_computed_at() {
        let report = report(&package(), Region::SG).unwrap();
        assert_eq!(report.floating_base, Some(dec!(0.0112)));
    }

    #[test]
    fn a_report_on_contractual_rates_states_no_assumption() {
        // Nothing was held still, so there is nothing to disclose, and a
        // caveat printed anyway would teach the reader to skip the ones
        // that mean something.
        assert_eq!(report(&flat(), Region::US).unwrap().floating_base, None);
    }

    #[test]
    fn disclosing_the_benchmark_changes_no_figure_on_the_page() {
        let contractual = Loan::builder()
            .principal(dec!(400000))
            .rate_type(RateType::Reverting {
                base_rate: dec!(0.0112),
                base_floats: false,
                initial_spread: dec!(0.003),
                initial_years: dec!(2),
                thereafter_spread: dec!(0.006),
            })
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Monthly)
            .build()
            .unwrap();

        let quoted = report(&package(), Region::SG).unwrap();
        let agreed = report(&contractual, Region::SG).unwrap();

        assert_eq!(quoted.bands, agreed.bands);
        assert_eq!(quoted.rate_rise, agreed.rate_rise);
        assert_eq!(quoted.total_paid, agreed.total_paid);
        assert_ne!(quoted.floating_base, agreed.floating_base);
    }

    /// The shape every Singapore home loan takes.
    fn package() -> Loan {
        Loan::builder()
            .principal(dec!(400000))
            .rate_type(RateType::Reverting {
                base_rate: dec!(0.0112),
                base_floats: true,
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
        let bands = report(&flat(), Region::US).unwrap().bands;
        assert_eq!(bands.len(), 1);
        assert_eq!(bands[0].from_year, dec!(1));
        assert_eq!(bands[0].to_year, dec!(30));
    }

    #[test]
    fn a_package_is_split_at_the_lock_in_with_no_year_lost_between_them() {
        // The bands are read as "Years 1-2" and "Years 3-25". An off-by-one
        // here silently drops or double-counts a year of the loan.
        let bands = report(&package(), Region::SG).unwrap().bands;
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
        let report = report(&package(), Region::SG).unwrap();
        let first = report.rate_rise.first().unwrap();

        assert_eq!(first.annual_rate, report.final_annual_rate + dec!(0.005));
        assert!(first.annual_rate > report.initial_annual_rate + dec!(0.005));
    }

    #[test]
    fn each_rise_costs_more_than_the_one_below_it() {
        let rows = report(&package(), Region::SG).unwrap().rate_rise;
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
        let report = report(&package(), Region::SG).unwrap();
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
        let rows = report(&flat(), Region::US).unwrap().rate_rise;
        assert_eq!(rows.len(), 3);
        assert!(rows[0].payment > crate::payment::payment_amount(&flat()));
    }

    #[test]
    fn the_schedule_has_a_row_for_every_payment() {
        let report = report(&package(), Region::SG).unwrap();
        assert_eq!(report.schedule.len(), 300);
        assert_eq!(report.schedule.first().unwrap().period, 1);
        assert_eq!(report.schedule.last().unwrap().period, 300);
        assert_eq!(
            report.schedule.last().unwrap().remaining_balance,
            Decimal::ZERO
        );
    }

    #[test]
    fn the_schedule_follows_the_cadence_the_loan_is_paid_on() {
        // The row count is the number of payments, not the number of years.
        // A weekly loan is not a monthly loan with smaller numbers.
        let weekly = Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.02))
            .term_years(dec!(25))
            .frequency(PaymentFrequency::Weekly)
            .build()
            .unwrap();

        assert_eq!(report(&weekly, Region::SG).unwrap().schedule.len(), 1300);
    }

    #[test]
    fn every_payment_knows_which_year_it_falls_in() {
        let report = report(&package(), Region::SG).unwrap();

        // Payments 1..=12 are year 1 and 13 opens year 2. Dividing without
        // the -1 would put every January in the year before it.
        assert_eq!(report.schedule[0].year, 1);
        assert_eq!(report.schedule[11].year, 1);
        assert_eq!(report.schedule[12].year, 2);
        assert_eq!(report.schedule.last().unwrap().year, 25);
    }

    #[test]
    fn the_year_marker_counts_in_the_cadence_the_loan_is_paid_on() {
        // 26 payments to a year, not 12. A bi-weekly schedule numbered as
        // though it were monthly would claim year 25 by payment 300 and run
        // to year 54.
        let biweekly = Loan::builder()
            .principal(dec!(400000))
            .annual_rate(dec!(0.02))
            .term_years(dec!(25))
            .frequency(PaymentFrequency::BiWeekly)
            .build()
            .unwrap();
        let schedule = report(&biweekly, Region::SG).unwrap().schedule;

        assert_eq!(schedule[25].year, 1);
        assert_eq!(schedule[26].year, 2);
        assert_eq!(schedule.last().unwrap().year, 25);
    }

    #[test]
    fn the_two_cuts_of_the_schedule_are_the_same_schedule() {
        // The reader picks which one the document prints. They are cut from
        // one calculation, so picking cannot change what the loan costs.
        let report = report(&package(), Region::SG).unwrap();

        assert_eq!(report.yearly.len(), 25);
        for year in &report.yearly {
            let payments: Vec<_> = report
                .schedule
                .iter()
                .filter(|p| p.year == year.year)
                .collect();
            assert_eq!(payments.len(), 12, "year {}", year.year);
            assert_eq!(
                payments.iter().map(|p| p.interest).sum::<Decimal>(),
                year.interest
            );
            assert_eq!(
                payments.last().unwrap().remaining_balance,
                year.remaining_balance
            );
        }
    }

    #[test]
    fn the_schedule_shows_the_step_up_on_the_payment_it_happens() {
        // The reason this is not a yearly roll-up. A package reverting at
        // 24 months steps between payment 24 and payment 25; a year-2 row
        // would average the two and print an instalment the borrower never
        // pays on a day they never pay it.
        let report = report(&package(), Region::SG).unwrap();
        let before = report.schedule[23].paid;
        let after = report.schedule[24].paid;

        assert_eq!(before, report.initial_payment);
        assert_eq!(after, report.payment_after_reversion.unwrap());
        assert_ne!(before, after);
    }

    #[test]
    fn the_totals_match_the_schedule_they_are_shown_beside() {
        // The document prints both. Two routes to one figure is how a
        // summary ends up disagreeing with the rows under it.
        let report = report(&package(), Region::SG).unwrap();
        let from_rows: Decimal = report.schedule.iter().map(|r| r.interest).sum();

        assert_eq!(round_currency(from_rows), report.total_interest);
    }

    #[test]
    fn a_report_cites_the_market_it_is_read_in() {
        let sg = report(&package(), Region::SG).unwrap().references;
        let us = report(&flat(), Region::US).unwrap().references;

        assert!(sg.contains(&Authority::MasNotice645));
        assert!(sg.contains(&Authority::Iras));
        assert!(!sg.contains(&Authority::Fhfa));

        assert!(us.contains(&Authority::Fhfa));
        assert!(!us.contains(&Authority::MasNotice645));
    }

    #[test]
    fn no_report_is_published_without_citing_anything() {
        // A document that states a servicing ceiling and an LTV cap without
        // naming who set them reads as the issuer's own word for it.
        for region in [Region::SG, Region::US] {
            assert!(!references(region).is_empty(), "{region:?}");
        }
    }

    #[test]
    fn every_authority_can_actually_be_looked_up() {
        // A citation nobody can follow is decoration.
        for region in [Region::SG, Region::US] {
            for authority in references(region) {
                let url = authority.url();
                assert!(url.starts_with("https://"), "{authority:?}: {url}");
            }
        }
    }

    #[test]
    fn both_markets_name_the_disclosure_this_document_is_not() {
        // The shape is borrowed from a regulated disclosure a lender issues
        // about an offer. Naming the real thing, with a link, is what keeps
        // a borrower from mistaking this for it.
        assert!(references(Region::SG).contains(&Authority::MasNotice632a));
        assert!(references(Region::US).contains(&Authority::Cfpb));
    }
}
