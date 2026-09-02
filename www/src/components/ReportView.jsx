import React, { useId, useMemo, useState } from 'react';
import ScenarioFields from './ScenarioFields';
import CalcError from './CalcError';
import ReportDocument from './ReportDocument';
import { currencySymbol, makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';
import { allFilled } from '../inputs';
import { rateValues, toRateTypeDto } from '../rate';
import { looksLikeAddress, mailtoUrl, parseRecipients } from '../mailto';
import { DEFAULT_SCENARIO, useScenarioSummary } from '../scenario';

// Quoted only when a value needs it -- a sentence like the rate's "can this
// change" column routinely carries a comma of its own; every plain number
// here passes through untouched.
function csvField(value) {
  const text = String(value);
  return /[",\r\n]/.test(text) ? `"${text.replace(/"/g, '""')}"` : text;
}

function rowsToCsv(rows) {
  return rows.map((row) => row.map(csvField).join(',')).join('\r\n');
}

function downloadCsv(filename, csv) {
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

/**
 * Every section of `ReportDocument`, translated into one CSV rather than one
 * printed page -- same numbers, same order, same reasoning about what can
 * change and why (see mortgage-calc's `report` module and CLAUDE.md's "carry
 * what a figure assumed, next to the figure"). Plain decimals throughout
 * rather than the document's locale-formatted currency strings: the point of
 * a CSV is to be summed in a spreadsheet, not read on a page.
 */
function buildReportCsv({ report, scenario, region, granularity, cadence, t, locale }) {
  const pct = (n) => (n == null ? '—' : `${n.toFixed(3)}%`);
  const num = (n) => Number(n).toFixed(2);
  const steps = report.payment_after_reversion != null;
  const floats = report.floating_base_percent != null;
  const benchmarkNote = floats ? ` ${t('report.andWithBenchmark')}` : '';
  const byPayment = granularity !== 'year';
  const preparedOn = new Date().toLocaleDateString(locale, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });

  const sections = [
    rowsToCsv([
      [t('report.prepared'), preparedOn],
      [t('report.market'), t(`region.${region}`)],
    ]),
    rowsToCsv([
      [t('report.terms')],
      [t('report.item'), t('report.value'), t('report.canChange')],
      [t('field.homePrice'), num(scenario.homePrice), t('report.no')],
      [t('field.loanAmount'), num(report.principal), t('report.no')],
      [t('field.loanTerm'), t('duration.years', { years: report.term_years }), t('report.no')],
      [
        t('field.interestRate'),
        pct(report.initial_rate_percent),
        (steps
          ? t('report.ratePlan', {
              years: report.lock_in_years,
              rate: pct(report.final_rate_percent),
            })
          : t('report.no')) + benchmarkNote,
      ],
      [
        t('payment.payment'),
        num(report.initial_payment),
        (steps
          ? t('report.paymentPlan', {
              years: report.lock_in_years,
              payment: num(report.payment_after_reversion),
            })
          : t('report.no')) + benchmarkNote,
      ],
    ]),
  ];

  // The disclosure travels with the figures it qualifies, same as it does
  // on the printed page -- see CLAUDE.md's "carry what a figure assumed".
  if (report.rate_note) {
    sections.push(
      rowsToCsv([[t(report.rate_note.code, report.rate_note.params) || report.rate_note.text]]),
    );
  }

  sections.push(
    rowsToCsv([
      [t('report.overTime')],
      [t('report.period'), t('rate.rate'), t('report.instalment', { cadence })],
      ...report.bands.map((band) => [
        t('report.yearRange', { from: band.from_year, to: band.to_year }),
        pct(band.annual_rate_percent),
        num(band.payment),
      ]),
    ]),
  );

  sections.push(
    rowsToCsv([
      [t('report.totalPaid'), num(report.total_paid)],
      [t('payment.totalInterest'), num(report.total_interest)],
      [t('report.interestShare'), pct(report.interest_share_percent)],
    ]),
  );

  sections.push(
    rowsToCsv([
      [t('report.ifRatesRise')],
      [
        t('report.increase'),
        t('rate.rate'),
        t('report.instalment', { cadence }),
        t('report.paymentIncrease'),
      ],
      ...report.rate_rise.map((row) => [
        t('report.plusPoints', { points: row.increase_percent.toFixed(2) }),
        pct(row.annual_rate_percent),
        num(row.payment),
        num(row.payment_increase),
      ]),
    ]),
  );

  const scheduleHeader = byPayment
    ? [
        t('report.paymentNo'),
        t('amort.year'),
        t('amort.paid'),
        t('amort.principal'),
        t('amort.interest'),
        t('amort.balance'),
      ]
    : [
        t('amort.year'),
        t('amort.paid'),
        t('amort.principal'),
        t('amort.interest'),
        t('amort.balance'),
      ];
  const scheduleRows = (byPayment ? report.schedule : report.yearly).map((row) =>
    byPayment
      ? [
          row.period,
          row.year,
          num(row.paid),
          num(row.principal),
          num(row.interest),
          num(row.remaining_balance),
        ]
      : [
          row.year,
          num(row.paid),
          num(row.principal),
          num(row.interest),
          num(row.remaining_balance),
        ],
  );
  sections.push(
    rowsToCsv([
      [t(byPayment ? 'report.schedule' : 'report.scheduleYearly')],
      scheduleHeader,
      ...scheduleRows,
    ]),
  );

  // Same footer the printed page ends on -- the disclaimer and its sources
  // read as caveats on figures already shown, not as a warning up front.
  sections.push(
    rowsToCsv([
      [t('report.referenceOnly')],
      [t(`about.disclaimer.${region}`)],
      [t('report.sources')],
      ...report.references.map((reference) => [t(reference.code), reference.url]),
    ]),
  );

  return sections.join('\r\n\r\n');
}

/**
 * The document tab: the same loan as everywhere else, laid out to be handed
 * to somebody who was not sitting at the screen.
 *
 * Printing is the browser's own — its "Save as PDF" destination is how the
 * file gets made, so Print and Save are one action rather than two engines.
 * Nothing is uploaded to produce it, which is the same promise the rest of
 * the app makes.
 */
export default function ReportView({
  wasmModule,
  region = 'US',
  scenario = DEFAULT_SCENARIO,
  onScenarioChange,
}) {
  const { t, locale } = useI18n();
  const [recipients, setRecipients] = useState('');
  // Handing a message to a mail client is the one thing this page does that
  // points outward. It gets a confirmation naming exactly who it is
  // addressed to, because a mistyped or half-edited recipient list is only
  // obvious once it is spelled back.
  const [confirming, setConfirming] = useState(false);
  // How much of the schedule goes on the document. A view choice, not a
  // loan input: both cuts arrive from one `build_report` call, so flipping
  // this re-renders and never recalculates.
  const [granularity, setGranularity] = useState('payment');
  // A group of buttons, not a labelled control -- see RateFields.
  const scheduleViewId = useId();
  const formatMoney = makeFormatMoney(region);
  const money = currencySymbol(region);
  const { rate, termYears, frequency } = scenario;
  const { principal } = useScenarioSummary(wasmModule, scenario);
  const rateType = toRateTypeDto(rate);
  const rateKey = JSON.stringify(rateType);

  const report = useMemo(() => {
    if (!wasmModule?.build_report) return null;
    if (!allFilled(scenario.homePrice, scenario.downPayment, ...rateValues(rate), termYears))
      return null;
    return wasmModule.build_report({
      loan: { principal, rate: rateType, term_years: termYears, frequency },
      region,
    });
  }, [wasmModule, region, principal, rateKey, termYears, frequency]);

  const sourceUrl = window.location.origin + window.location.pathname;
  const addresses = parseRecipients(recipients);
  const rejected = addresses.filter((address) => !looksLikeAddress(address));
  const cadence = t(`freq.${report?.frequency ?? frequency}`);

  function downloadReportCsv() {
    const csv = buildReportCsv({ report, scenario, region, granularity, cadence, t, locale });
    const kind = granularity === 'year' ? 'yearly' : 'payment';
    downloadCsv(`mortgage-report-${kind}-${new Date().toISOString().slice(0, 10)}.csv`, csv);
  }

  /**
   * Hands the message to the reader's own mail client. Nothing is sent from
   * here, and the body says where the attachment has to come from — a
   * `mailto:` link cannot carry one.
   */
  function email() {
    setConfirming(false);
    const summary = [
      t('report.mailBody', {
        payment: formatMoney(report.initial_payment),
        principal: formatMoney(report.principal),
        years: report.term_years,
      }),
      report.payment_after_reversion != null
        ? t('report.mailSteps', {
            years: report.lock_in_years,
            payment: formatMoney(report.payment_after_reversion),
          })
        : null,
      '',
      t('report.referenceOnly'),
      '',
      `${t('report.workedAt')} ${sourceUrl}`,
      '',
      t('report.mailAttach'),
    ]
      .filter((line) => line !== null)
      .join('\n');

    window.location.href = mailtoUrl({
      recipients,
      subject: t('report.mailSubject'),
      body: summary,
    });
  }

  return (
    <>
      {/* The document is a sibling of the panel, not a child of it. The
          print stylesheet hides the calculator's panels, and a report
          nested inside one printed as a blank page. */}
      <section className="panel">
        <ScenarioFields
          wasmModule={wasmModule}
          scenario={scenario}
          onChange={onScenarioChange}
          money={money}
          formatMoney={formatMoney}
          collapsible
        />

        <CalcError result={report} />

        {report && !report.error && (
          <div className="report-actions">
            {/* What the document contains, before what to do with it. The
                two used to sit on one wrapping row, so a setting and a
                call to action read as one control group -- and each note
                was squeezed into whatever width was left beside its
                button. Three stacked rows, each with its explanation
                underneath it at full width. */}
            <div className="report-option" role="group" aria-labelledby={scheduleViewId}>
              <span className="field-label" id={scheduleViewId}>
                {t('report.scheduleView')}
              </span>
              <div className="rate-kind">
                {['payment', 'year'].map((option) => (
                  <button
                    key={option}
                    type="button"
                    className={granularity === option ? 'kind-toggle active' : 'kind-toggle'}
                    aria-pressed={granularity === option}
                    onClick={() => setGranularity(option)}
                  >
                    {t(option === 'payment' ? 'report.byPayment' : 'report.byYear')}
                  </button>
                ))}
              </div>
            </div>

            <div className="report-action">
              <div className="report-action-row">
                <button className="primary-button" onClick={() => window.print()}>
                  {t('report.print')}
                </button>
              </div>
              <p className="report-actions-note">{t('report.printNote')}</p>
            </div>

            <div className="report-action">
              <div className="report-action-row">
                <button className="secondary-button" onClick={downloadReportCsv}>
                  {t('report.downloadCsv')}
                </button>
              </div>
              <p className="report-actions-note">{t('report.downloadCsvNote')}</p>
            </div>

            <div className="report-action">
              <div className="report-action-row">
                <label className="field report-recipients">
                  <span className="field-label">{t('report.recipients')}</span>
                  <div className="field-input">
                    <input
                      type="text"
                      value={recipients}
                      onChange={(e) => setRecipients(e.target.value)}
                      placeholder={t('report.recipientsPlaceholder')}
                    />
                  </div>
                </label>
                <button
                  className="secondary-button"
                  onClick={() => setConfirming(true)}
                  disabled={addresses.length === 0 || rejected.length > 0}
                >
                  {t('report.email', { count: addresses.length })}
                </button>
              </div>
              {rejected.length > 0 && (
                <p className="report-actions-warning">
                  {t('report.recipientsBad', { addresses: rejected.join(', ') })}
                </p>
              )}
              <p className="report-actions-note">{t('report.emailNote')}</p>
            </div>

            {confirming && (
              <div className="report-confirm" role="dialog" aria-label={t('report.confirmTitle')}>
                <h3>{t('report.confirmTitle')}</h3>
                {/* Spelled out one per line rather than summarised as a
                    count: "2 recipients" is exactly the phrasing that lets a
                    stale address through. */}
                <ul className="report-confirm-list">
                  {addresses.map((address) => (
                    <li key={address}>{address}</li>
                  ))}
                </ul>
                <p>{t('report.confirmBody')}</p>
                <div className="report-confirm-actions">
                  <button className="primary-button" onClick={email}>
                    {t('report.confirmSend')}
                  </button>
                  <button className="secondary-button" onClick={() => setConfirming(false)}>
                    {t('report.confirmCancel')}
                  </button>
                </div>
              </div>
            )}
          </div>
        )}
      </section>

      {report && !report.error && (
        <ReportDocument
          report={report}
          region={region}
          scenario={scenario}
          sourceUrl={sourceUrl}
          granularity={granularity}
        />
      )}
    </>
  );
}
