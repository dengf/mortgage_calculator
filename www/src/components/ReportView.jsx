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
  const { t } = useI18n();
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
        />

        <CalcError result={report} />

        {report && !report.error && (
          <div className="report-actions">
            <div className="field" role="group" aria-labelledby={scheduleViewId}>
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

            <button className="primary-button" onClick={() => window.print()}>
              {t('report.print')}
            </button>
            <p className="report-actions-note">{t('report.printNote')}</p>

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
            {rejected.length > 0 && (
              <p className="report-actions-warning">
                {t('report.recipientsBad', { addresses: rejected.join(', ') })}
              </p>
            )}
            <p className="report-actions-note">{t('report.emailNote')}</p>

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
