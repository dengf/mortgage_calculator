import React from 'react';
import { makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';

/**
 * The printable illustration.
 *
 * A document, not the calculator in print clothing: its own layout, its own
 * order, and only the figures a reader who was not sitting at the screen
 * needs. What it says comes from `mortgage-calc`'s `report` module — which
 * bands the loan divides into, how it is stressed, and which authorities it
 * cites are all decided there. See CLAUDE.md.
 *
 * Two things are structural rather than decorative and must not be styled
 * away: the watermark across every page, and the footer naming this as an
 * illustration with a link back to the working. The shape of the page is
 * borrowed from disclosures a *lender* issues about an offer it is making
 * — the CFPB's Loan Estimate, MAS's Residential Property Loan Fact Sheet —
 * and the closer it gets to those, the more it has to say what it is not.
 * `ReportDocument.test.jsx` holds it to that.
 */
export default function ReportDocument({ report, region, scenario, sourceUrl }) {
  const { t, locale } = useI18n();
  const money = makeFormatMoney(region);
  const pct = (n) => (n == null ? '—' : `${n.toFixed(3)}%`);
  const preparedOn = new Date().toLocaleDateString(locale, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });

  if (!report || report.error) return null;

  const steps = report.payment_after_reversion != null;
  // Rust decided whether anything was held still to produce these figures,
  // and wrote the sentence saying so. See `mortgage_calc::report::Report`.
  const floats = report.floating_base_percent != null;
  // The cadence the instalments are on. Every column that used to say
  // "Monthly" said it over whatever the borrower actually pays -- the
  // frequency never crossed the boundary, so the document could not know.
  const cadence = t(`freq.${report.frequency ?? 'monthly'}`);

  return (
    <article className="report" lang={locale}>
      {/* Repeated per page by the print stylesheet, not once at the top:
          a page that gets separated from the others still carries it. */}
      <div className="report-watermark" aria-hidden="true">
        <span>{t('report.watermark')}</span>
      </div>

      <header className="report-head">
        <div>
          <h1>{t('report.title')}</h1>
          <p className="report-subtitle">{t('report.subtitle')}</p>
        </div>
        <dl className="report-meta">
          <div>
            <dt>{t('report.prepared')}</dt>
            <dd>{preparedOn}</dd>
          </div>
          <div>
            <dt>{t('report.market')}</dt>
            <dd>{t(`region.${region}`)}</dd>
          </div>
        </dl>
      </header>

      {/* The CFPB's "Can this amount increase after closing?" column. On a
          package it is the only honest way to state a rate or an
          instalment, and it is the question a borrower does not know to
          ask. */}
      <section className="report-section">
        <h2>{t('report.terms')}</h2>
        <div className="report-table-wrap">
          <table className="report-table report-terms">
            <thead>
              <tr>
                <th>{t('report.item')}</th>
                <th>{t('report.value')}</th>
                <th>{t('report.canChange')}</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <th scope="row">{t('field.homePrice')}</th>
                <td>{money(scenario.homePrice)}</td>
                <td>{t('report.no')}</td>
              </tr>
              <tr>
                <th scope="row">{t('field.loanAmount')}</th>
                <td>{money(report.principal)}</td>
                <td>{t('report.no')}</td>
              </tr>
              <tr>
                <th scope="row">{t('field.loanTerm')}</th>
                <td>{t('duration.years', { years: report.term_years })}</td>
                <td>{t('report.no')}</td>
              </tr>
              <tr>
                <th scope="row">{t('field.interestRate')}</th>
                <td>{pct(report.initial_rate_percent)}</td>
                <td>
                  {steps
                    ? t('report.ratePlan', {
                        years: report.lock_in_years,
                        rate: pct(report.final_rate_percent),
                      })
                    : t('report.no')}
                  {/* A rate that steps up on a schedule and a rate that moves
                    with a benchmark are two different answers to this
                    column, and a package can give both. Printing only the
                    schedule would let "then 1.720%" read as the last word. */}
                  {floats && (
                    <span className="report-can-change">{t('report.andWithBenchmark')}</span>
                  )}
                </td>
              </tr>
              <tr>
                <th scope="row">{t('payment.payment')}</th>
                <td>{money(report.initial_payment)}</td>
                <td>
                  {steps
                    ? t('report.paymentPlan', {
                        years: report.lock_in_years,
                        payment: money(report.payment_after_reversion),
                      })
                    : t('report.no')}
                  {/* The instalment moves for the same reason the rate does.
                    Saying it of the rate alone reads as though the payment
                    settles at the thereafter figure and stays there. */}
                  {floats && (
                    <span className="report-can-change">{t('report.andWithBenchmark')}</span>
                  )}
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        {report.rate_note && (
          <p className="report-note report-assumption">
            {t(report.rate_note.code, report.rate_note.params) || report.rate_note.text}
          </p>
        )}
      </section>

      {/* The CFPB's year bands. A loan whose payment moves is described
          over time or not at all. */}
      <section className="report-section">
        <h2>{t('report.overTime')}</h2>
        <div className="report-table-wrap">
          <table className="report-table">
            <thead>
              <tr>
                <th>{t('report.period')}</th>
                <th>{t('rate.rate')}</th>
                <th>{t('report.instalment', { cadence })}</th>
              </tr>
            </thead>
            <tbody>
              {report.bands.map((band) => (
                <tr key={band.from_year}>
                  <th scope="row">
                    {t('report.yearRange', { from: band.from_year, to: band.to_year })}
                  </th>
                  <td>{pct(band.annual_rate_percent)}</td>
                  <td>{money(band.payment)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <dl className="report-totals">
          <div>
            <dt>{t('report.totalPaid')}</dt>
            <dd>{money(report.total_paid)}</dd>
          </div>
          <div>
            <dt>{t('payment.totalInterest')}</dt>
            <dd>{money(report.total_interest)}</dd>
          </div>
          <div>
            <dt>{t('report.interestShare')}</dt>
            <dd>{pct(report.interest_share_percent)}</dd>
          </div>
        </dl>
      </section>

      {/* Required of a Singapore fact sheet, and the most useful thing on
          the page in either market: what a rise actually costs per month. */}
      <section className="report-section">
        <h2>{t('report.ifRatesRise')}</h2>
        <p className="report-note">{t('report.ifRatesRiseNote')}</p>
        <div className="report-table-wrap">
          <table className="report-table">
            <thead>
              <tr>
                <th>{t('report.increase')}</th>
                <th>{t('rate.rate')}</th>
                <th>{t('report.instalment', { cadence })}</th>
                <th>{t('report.paymentIncrease')}</th>
              </tr>
            </thead>
            <tbody>
              {report.rate_rise.map((row) => (
                <tr key={row.increase_percent}>
                  <th scope="row">
                    {t('report.plusPoints', { points: row.increase_percent.toFixed(2) })}
                  </th>
                  <td>{pct(row.annual_rate_percent)}</td>
                  <td>{money(row.payment)}</td>
                  <td>{money(row.payment_increase)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className="report-section report-schedule">
        <h2>{t('report.schedule')}</h2>
        <div className="report-table-wrap">
          <table className="report-table">
            <thead>
              <tr>
                <th>{t('report.paymentNo')}</th>
                <th>{t('amort.paid')}</th>
                <th>{t('amort.principal')}</th>
                <th>{t('amort.interest')}</th>
                <th>{t('amort.balance')}</th>
              </tr>
            </thead>
            <tbody>
              {report.schedule.map((row) => (
                <tr key={row.period}>
                  <th scope="row">{row.period}</th>
                  <td>{money(row.paid)}</td>
                  <td>{money(row.principal)}</td>
                  <td>{money(row.interest)}</td>
                  <td>{money(row.remaining_balance)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <footer className="report-foot">
        <p className="report-disclaimer">{t('report.referenceOnly')}</p>
        <p className="report-disclaimer">{t(`about.disclaimer.${region}`)}</p>

        <h3>{t('report.sources')}</h3>
        <ul className="report-sources">
          {report.references.map((reference) => (
            <li key={reference.code}>
              {t(reference.code)} — <span className="report-url">{reference.url}</span>
            </li>
          ))}
        </ul>

        <p className="report-origin">
          {t('report.workedAt')} <span className="report-url">{sourceUrl}</span>
        </p>
      </footer>
    </article>
  );
}
