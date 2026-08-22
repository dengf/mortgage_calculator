import React from 'react';
import NumberField from './NumberField';
import { makeFormatMoney } from '../currency';
import { useI18n } from '../i18n';

// This panel only ever renders under the US region, so its currency is
// fixed rather than passed in.
const formatUsd = makeFormatMoney('US');

/**
 * US panel: conforming/jumbo classification, ZIP-derived property tax, the
 * PMI trigger below 20% down, the resulting PITI, and an optional
 * mortgage-interest deduction estimate.
 *
 * Every figure comes from `mortgage_calc::united_states` through the
 * `calculate_united_states` binding — no US rules are reimplemented here.
 */
export default function UnitedStatesPanel({ inputs, onChange, result, homePrice }) {
  const { t } = useI18n();
  const set = (key) => (value) => onChange({ ...inputs, [key]: value });

  return (
    <div className="sg-panel">
      <h3 className="sg-panel-title">{t('us.title')}</h3>

      <div className="panel-form sg-panel-form">
        <label className="field">
          <span className="field-label">{t('us.zip')}</span>
          <div className="field-input">
            <input
              type="text"
              inputMode="numeric"
              maxLength={5}
              value={inputs.zip}
              onChange={(e) => set('zip')(e.target.value.replace(/\D/g, ''))}
            />
          </div>
        </label>
        <NumberField
          label={t('us.pmiRate')}
          value={inputs.pmi_rate_percent}
          onChange={set('pmi_rate_percent')}
          suffix="%"
          min={0}
        />
        <label className="field">
          <span className="field-label">{t('us.useTaxDeduction')}</span>
          <select
            className="field-select"
            value={inputs.use_tax_deduction ? 'yes' : 'no'}
            onChange={(e) => set('use_tax_deduction')(e.target.value === 'yes')}
          >
            <option value="no">{t('us.no')}</option>
            <option value="yes">{t('us.yes')}</option>
          </select>
        </label>
        {inputs.use_tax_deduction && (
          <NumberField
            label={t('us.marginalRate')}
            value={inputs.marginal_tax_rate_percent}
            onChange={set('marginal_tax_rate_percent')}
            suffix="%"
            min={0}
          />
        )}
      </div>

      {result?.error && (
        <div className="error">
          {result.error_message ? t(result.error_message.code, result.error_message.params) : result.error}
        </div>
      )}

      {result?.property_tax_rate_percent == null && inputs.zip.length > 0 && (
        <div className="error">{t('us.unknownZip', { zip: inputs.zip })}</div>
      )}

      {result && !result.error && (
        <>
          <div className="stat-grid">
            <div className="stat">
              <span className="stat-label">{t('us.loanType')}</span>
              <span className="stat-value">{result.loan_type === 'Jumbo' ? t('us.jumbo') : t('us.conforming')}</span>
            </div>
            <div className="stat">
              <span className="stat-label">{t('us.downPayment')}</span>
              <span className="stat-value">
                {formatUsd(result.down_payment)}{' '}
                <small>({result.down_payment_percent.toFixed(1)}%)</small>
              </span>
            </div>
            <div className="stat">
              <span className="stat-label">
                {result.property_tax_rate_percent == null
                  ? t('us.propertyTax')
                  : t('us.propertyTaxWithRate', {
                      rate: result.property_tax_rate_percent.toFixed(2),
                    })}
              </span>
              <span className="stat-value">{formatUsd(result.monthly_property_tax)}</span>
            </div>
            <div className="stat">
              <span className="stat-label">
                {result.pmi_required ? t('us.pmiRequired') : t('us.pmiNotRequired')}
              </span>
              <span className="stat-value">{formatUsd(result.monthly_pmi)}</span>
            </div>
          </div>

          <div className="stat-grid">
            <div className="stat stat-primary">
              <span className="stat-label">{t('us.piti')}</span>
              <span className="stat-value">{formatUsd(result.monthly_piti)}</span>
            </div>
            {result.monthly_tax_savings != null && (
              <div className="stat">
                <span className="stat-label">{t('us.taxSavings')}</span>
                <span className="stat-value">{formatUsd(result.monthly_tax_savings)}</span>
              </div>
            )}
            {result.net_monthly_cost != null && (
              <div className="stat">
                <span className="stat-label">{t('us.netCost')}</span>
                <span className="stat-value">{formatUsd(result.net_monthly_cost)}</span>
              </div>
            )}
          </div>

          {result.pmi_required && (
            <p className="chart-note">
              {t('us.pmiHint', { amount: formatUsd((Number(homePrice) || 0) * 0.2) })}
            </p>
          )}
        </>
      )}
    </div>
  );
}
