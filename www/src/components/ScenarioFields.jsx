import React, { useState } from 'react';
import NumberField from './NumberField';
import DownPaymentField from './DownPaymentField';
import RateFields from './RateFields';
import { useScenarioSummary } from '../scenario';
import { useI18n } from '../i18n';

/**
 * The shared loan inputs, rendered identically wherever the same loan is
 * being described.
 *
 * `fields` selects which of them a tab needs — Compare varies rate and term
 * per scenario row, so it takes only price, deposit and cadence.
 *
 * `collapsible` lets Amortization and Report start with this section folded
 * away, since both restate the same loan Payment already shows in full and
 * lead with their own headline (the schedule, the document) rather than the
 * inputs that produced it. Payment doesn't pass it, so it renders exactly as
 * before -- always open, no toggle chrome at all.
 */
export default function ScenarioFields({
  wasmModule,
  scenario,
  onChange,
  money,
  formatMoney,
  fields = ['price', 'downPayment', 'rate', 'term', 'frequency'],
  collapsible = false,
}) {
  const { t } = useI18n();
  const summary = useScenarioSummary(wasmModule, scenario);
  const set = (key) => (value) => onChange({ ...scenario, [key]: value });
  const has = (f) => fields.includes(f);
  // Collapsed by default when collapsible: this is a restatement of a loan
  // already dialled in on Payment, not the first thing to fill in here.
  const [expanded, setExpanded] = useState(!collapsible);

  const fieldsBlock = (
    <div className="panel-form">
      {has('price') && (
        <NumberField
          label={t('field.homePrice')}
          value={scenario.homePrice}
          onChange={set('homePrice')}
          suffix={money}
          min={0}
          grouped
        />
      )}

      {has('downPayment') && (
        <DownPaymentField
          label={t('field.downPayment')}
          wasmModule={wasmModule}
          scenario={scenario}
          percent={summary.downPaymentPercent}
          onChange={set('downPayment')}
          money={money}
        />
      )}

      {/* A rate, not a rate figure: a Singapore package is quoted as an
          index, a promotional spread, how long it lasts, and the spread it
          steps up to. A single "Interest rate" box could only ever describe
          the first few years of one. */}
      {has('rate') && (
        <RateFields rate={scenario.rate} onChange={set('rate')} wasmModule={wasmModule} />
      )}

      {has('term') && (
        <NumberField
          label={t('field.loanTerm')}
          value={scenario.termYears}
          onChange={set('termYears')}
          suffix={t('field.years')}
          min={1}
        />
      )}

      {has('frequency') && (
        <label className="field">
          <span className="field-label">{t('field.paymentFrequency')}</span>
          <select
            className="field-select"
            value={scenario.frequency}
            onChange={(e) => set('frequency')(e.target.value)}
          >
            <option value="monthly">{t('freq.monthly')}</option>
            <option value="biweekly">{t('freq.biweekly')}</option>
            <option value="weekly">{t('freq.weekly')}</option>
          </select>
        </label>
      )}

      {/* The loan is derived, not entered. Showing it keeps the number the
          rest of the app talks about visible, and makes it structurally
          impossible for price and loan to contradict each other — a $300k
          loan on a $250k home used to be accepted without comment. */}
      {has('price') && has('downPayment') && (
        <div className="field field-derived">
          <span className="field-label">{t('field.loanAmount')}</span>
          <span className="field-derived-value">{formatMoney(summary.principal)}</span>
        </div>
      )}
    </div>
  );

  if (!collapsible) return fieldsBlock;

  return (
    <div className="scenario-fields-collapsible">
      <button
        type="button"
        className="scenario-fields-toggle"
        aria-expanded={expanded}
        onClick={() => setExpanded((v) => !v)}
      >
        <span className="scenario-fields-toggle-label">{t('field.loanDetails')}</span>
        {!expanded && (
          <span className="scenario-fields-summary">
            {[
              has('price') ? formatMoney(scenario.homePrice) : null,
              has('downPayment') && summary.downPaymentPercent != null
                ? t('field.percentOfPrice', { percent: summary.downPaymentPercent.toFixed(1) })
                : null,
              has('term') ? `${scenario.termYears} ${t('field.years')}` : null,
            ]
              .filter(Boolean)
              .join(' · ')}
          </span>
        )}
        <svg
          className="scenario-fields-caret"
          width="10"
          height="10"
          viewBox="0 0 10 10"
          aria-hidden="true"
        >
          <path
            d="M1.5 3.5L5 7L8.5 3.5"
            stroke="currentColor"
            strokeWidth="1.5"
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      </button>
      {expanded && fieldsBlock}
    </div>
  );
}
