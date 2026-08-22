import React from 'react';
import NumberField from './NumberField';
import DownPaymentField from './DownPaymentField';
import { principalOf } from '../scenario';
import { useI18n } from '../i18n';

/**
 * The shared loan inputs, rendered identically wherever the same loan is
 * being described.
 *
 * `fields` selects which of them a tab needs — Compare varies rate and term
 * per scenario row, so it takes only price, deposit and cadence.
 */
export default function ScenarioFields({
  scenario,
  onChange,
  money,
  formatMoney,
  fields = ['price', 'downPayment', 'rate', 'term', 'frequency'],
}) {
  const { t } = useI18n();
  const set = (key) => (value) => onChange({ ...scenario, [key]: value });
  const has = (f) => fields.includes(f);

  return (
    <div className="panel-form">
      {has('price') && (
        <NumberField
          label={t('field.homePrice')}
          value={scenario.homePrice}
          onChange={set('homePrice')}
          suffix={money}
          min={0}
        />
      )}

      {has('downPayment') && (
        <DownPaymentField
          label={t('field.downPayment')}
          scenario={scenario}
          onChange={set('downPayment')}
          money={money}
        />
      )}

      {has('rate') && (
        <NumberField
          label={t('field.interestRate')}
          value={scenario.rate}
          onChange={set('rate')}
          suffix="%"
          min={0}
        />
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
          <span className="field-derived-value">{formatMoney(principalOf(scenario))}</span>
        </div>
      )}
    </div>
  );
}
